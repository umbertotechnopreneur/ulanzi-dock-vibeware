// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Native session-bus indicators for an active Linux dock session.

use anyhow::{Context, Result};
use super::{ConnectionState, Control};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::Command,
    sync::{mpsc::{self, Receiver, Sender}, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};
use zbus::{
    blocking::{connection::Builder, fdo::DBusProxy, Connection},
    zvariant::{ObjectPath, Value},
};

pub(crate) mod connection;

const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const ITEM: &str = "/StatusNotifierItem";
const NOTIFICATIONS: &str = "org.freedesktop.Notifications";
const NOTIFICATION_PATH: &str = "/org/freedesktop/Notifications";

pub(crate) struct DesktopStatus {
    stop: Sender<()>,
    worker: Option<JoinHandle<()>>,
}

impl DesktopStatus {
    pub(crate) fn start(profile: PathBuf, control: Arc<Control>) -> Self {
        let (stop, receiver) = mpsc::channel();
        let worker = thread::Builder::new().name("desktop-status".into()).spawn(move || {
            if let Err(error) = serve(profile, control, receiver) {
                eprintln!("Desktop status unavailable: {error:#}");
            }
        }).map_err(|error| eprintln!("Could not start desktop status: {error}")).ok();
        Self { stop, worker }
    }
}

impl Drop for DesktopStatus {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

type Pixmaps = Vec<(i32, i32, Vec<u8>)>;

struct Indicator {
    profile: PathBuf,
    icon: Pixmaps,
    control: Arc<Control>,
}

impl Indicator {
    fn new(profile: PathBuf, control: Arc<Control>) -> Result<Self> {
        let image = image::load_from_memory(include_bytes!("../../assets/icons/ulanzi-dock-64.png"))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let mut pixels = image.into_raw();
        // SNI specifies network-byte-order ARGB for each pixel.
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.rotate_right(1);
        }
        Ok(Self { profile, icon: vec![(width as i32, height as i32, pixels)], control })
    }
}

#[zbus::interface(name = "org.kde.StatusNotifierItem")]
impl Indicator {
    #[zbus(property)]
    fn id(&self) -> &str { "ulanzideck" }
    #[zbus(property)]
    fn category(&self) -> &str { "Hardware" }
    #[zbus(property)]
    fn title(&self) -> &str { "UlanziDeck" }
    #[zbus(property)]
    fn status(&self) -> &str { "Active" }
    #[zbus(property)]
    fn icon_name(&self) -> &str { "" }
    #[zbus(property)]
    fn icon_pixmap(&self) -> Pixmaps { self.icon.clone() }
    #[zbus(property, name = "ToolTip")]
    fn tool_tip(&self) -> (String, Pixmaps, String, String) {
        (String::new(), self.icon.clone(), "UlanziDeck".into(), connection_text(self.control.connection()).into())
    }
    #[zbus(property)]
    fn item_is_menu(&self) -> bool { false }
    #[zbus(property)]
    fn menu(&self) -> ObjectPath<'static> {
        ObjectPath::from_static_str("/Menu").expect("valid static path")
    }

    fn activate(&self, _x: i32, _y: i32) -> zbus::fdo::Result<()> {
        open_settings(&self.profile)
    }
}

fn open_settings(profile: &std::path::Path) -> zbus::fdo::Result<()> {
    let executable = std::env::current_exe()
        .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))?;
    let mut command = Command::new(executable);
    command.arg("configure").arg("--config").arg(profile);
    super::linux_openers::spawn(&mut command)
        .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
}

// GNOME AppIndicator requires a real DBusMenu before showing an item.
// This static tree has a root (0) and one action (1).
type Properties = HashMap<String, Value<'static>>;
type Layout = (i32, Properties, Vec<Value<'static>>);

struct Menu(PathBuf);

fn menu_properties(id: i32, requested: &[String]) -> Properties {
    let mut props = Properties::new();
    let (key, value) = match id {
        0 => ("children-display", "submenu"),
        1 => ("label", "Open settings"),
        _ => return props,
    };
    if requested.is_empty() || requested.iter().any(|name| name == key) {
        props.insert(key.into(), Value::from(value));
    }
    props
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl Menu {
    #[zbus(property)]
    fn version(&self) -> u32 { 3 }

    #[zbus(out_args("revision", "layout"))]
    fn get_layout(&self, parent_id: i32, recursion_depth: i32, property_names: Vec<String>)
        -> zbus::fdo::Result<(u32, Layout)>
    {
        if parent_id != 0 && parent_id != 1 {
            return Err(zbus::fdo::Error::InvalidArgs("Unknown menu item".into()));
        }
        let mut children = Vec::new();
        if parent_id == 0 && recursion_depth != 0 {
            let leaf: Layout = (1, menu_properties(1, &property_names), Vec::new());
            children.push(Value::Structure(zbus::zvariant::Structure::from(leaf)));
        }
        Ok((1, (parent_id, menu_properties(parent_id, &property_names), children)))
    }

    fn get_group_properties(&self, ids: Vec<i32>, property_names: Vec<String>) -> Vec<(i32, Properties)> {
        [0, 1].into_iter().filter(|id| ids.is_empty() || ids.contains(id))
            .map(|id| (id, menu_properties(id, &property_names))).collect()
    }

    fn about_to_show(&self, _id: i32) -> bool { false }

    fn event(&self, id: i32, event_id: &str, _data: zbus::zvariant::OwnedValue, _timestamp: u32)
        -> zbus::fdo::Result<()>
    {
        if id == 1 && event_id == "clicked" {
            open_settings(&self.0)?;
        }
        Ok(())
    }
}

fn connection_text(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Waiting => "Waiting for D200H. Connection will be retried automatically.",
        ConnectionState::Connected => "D200H connected and ready.",
        ConnectionState::Disconnected => "D200H disconnected. Reconnecting automatically.",
    }
}

fn notify(bus: &Connection, title: &str, body: &str) -> Result<u32> {
    let message = bus.call_method(Some(NOTIFICATIONS), NOTIFICATION_PATH, Some(NOTIFICATIONS),
        "Notify", &("UlanziDeck", 0_u32, "ulanzideck", title,
        body, Vec::<&str>::new(), HashMap::<&str, Value<'_>>::new(), 5000_i32))?;
    Ok(message.body().deserialize()?)
}

pub(crate) fn notify_message(title: &str, body: &str) -> Result<()> {
    let bus = Builder::session()?.method_timeout(Duration::from_secs(1)).build()?;
    notify(&bus, title, body)?;
    Ok(())
}

fn serve(profile: PathBuf, control: Arc<Control>, stop: Receiver<()>) -> Result<()> {
    // Keep the indicator on an unnamed connection. GNOME can discover it as a
    // second item if it also shares the controller's well-known bus name.
    let _status = connection::publish(&profile, Arc::clone(&control))
        .map_err(|error| eprintln!("Could not publish the controller connection state: {error:#}"))
        .ok();
    let bus = Builder::session()?.method_timeout(Duration::from_secs(1))
        .serve_at("/Menu", Menu(profile.clone()))?
        .serve_at(ITEM, Indicator::new(profile, Arc::clone(&control))?)?.build().context("opening the desktop session bus")?;
    if stop.try_recv().is_ok() { return Ok(()); }

    // A notification service and an indicator host are independent and optional.
    let notification = notify(&bus, "UlanziDeck is running", connection_text(control.connection()))
        .map_err(|error| eprintln!("Desktop notification unavailable: {error:#}")).ok();
    if notification.is_some() {
        println!("Desktop notification accepted.");
    }
    let dbus = DBusProxy::new(&bus)?;
    let mut announced = false;
    let mut registered = None;
    loop {
        // Re-register if the desktop shell starts later or replaces its watcher.
        let owner = dbus.get_name_owner(WATCHER.try_into()?).ok();
        if owner != registered {
            registered = None;
            if let Some(owner) = owner {
                if bus.call_method(Some(owner.as_str()), "/StatusNotifierWatcher", Some(WATCHER),
                    "RegisterStatusNotifierItem", &(ITEM,)).is_ok() {
                    registered = Some(owner);
                    if !announced {
                        println!("Desktop status icon registered.");
                        announced = true;
                    }
                }
            }
        }
        if stop.recv_timeout(Duration::from_secs(10)) != Err(mpsc::RecvTimeoutError::Timeout) {
            break;
        }
    }
    if let Some(id) = notification {
        let _ = bus.call_method(Some(NOTIFICATIONS), NOTIFICATION_PATH, Some(NOTIFICATIONS), "CloseNotification", &(id,));
    }
    Ok(())
}
