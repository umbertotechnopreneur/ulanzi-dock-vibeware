// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

//! Run Wayland keyboard effects and portal authorization outside the dock loop.
use super::{dispatch, hotkey_keys, ActionPlan, Output};
use crate::tray::Control;
use anyhow::{Context, Result};
use enigo::{Direction, Key};
use futures_lite::{future, StreamExt};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use zbus::{blocking::Connection, proxy::SignalStream, zvariant::{OwnedObjectPath, OwnedValue, Value}};

const SERVICE: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.portal.RemoteDesktop";
const SESSION: &str = "org.freedesktop.portal.Session";
type Results = HashMap<String, OwnedValue>;
static SERIAL: AtomicU32 = AtomicU32::new(0);

struct Job {
    action: String,
    label: String,
    generation: u64,
    expires: Instant,
}

#[derive(Default)]
struct State {
    generation: AtomicU64,
    busy: AtomicBool,
}

pub(super) struct Executor {
    sender: Option<SyncSender<Job>>,
    state: Arc<State>,
    worker: Option<JoinHandle<()>>,
}

impl Executor {
    pub(super) fn start(control: Arc<Control>) -> Result<Self> {
        let state = Arc::new(State::default());
        let shared = Arc::clone(&state);
        let (sender, jobs) = mpsc::sync_channel::<Job>(1);
        let worker = thread::Builder::new().name("wayland-shortcuts".into()).spawn(move || {
            let mut keyboard = None;
            while let Ok(job) = jobs.recv() {
                let cancelled = || {
                    !control.is_running() || shared.generation.load(Ordering::SeqCst) != job.generation
                };
                if !cancelled() && Instant::now() < job.expires {
                    if let Err(error) = execute(&mut keyboard, &job.action, job.expires, &cancelled) {
                        if !cancelled() {
                            eprintln!("Action '{}' failed: {error:#}", job.label);
                        }
                    }
                }
                shared.busy.store(false, Ordering::SeqCst);
            }
        }).context("starting the Wayland shortcut worker")?;

        Ok(Self { sender: Some(sender), state, worker: Some(worker) })
    }

    /// One gesture can be in progress. A burst never queues delayed keyboard input.
    pub(super) fn submit(&self, action: &str, label: &str) {
        if self.state.busy.swap(true, Ordering::SeqCst) {
            return;
        }
        let job = Job {
            action: action.to_owned(),
            label: label.to_owned(),
            generation: self.state.generation.load(Ordering::SeqCst),
            expires: Instant::now() + Duration::from_secs(1),
        };
        let sender = self.sender.as_ref().expect("Wayland worker has a sender");
        if sender.try_send(job).is_err() {
            self.state.busy.store(false, Ordering::SeqCst);
            eprintln!("Action '{label}' failed: Wayland shortcut worker is unavailable");
        }
    }

    pub(super) fn cancel_pending(&self) {
        self.state.generation.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.cancel_pending();
        drop(self.sender.take());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn execute(
    keyboard: &mut Option<Keyboard>, action: &str, expires: Instant, cancelled: &impl Fn() -> bool,
) -> Result<()> {
    let chord = match action.strip_prefix("hotkey:") {
        Some(chord) => Cow::Borrowed(chord),
        None => Cow::Owned(super::desktop::clipboard_chord()?),
    };
    let keys = hotkey_keys(&chord)?;
    if cancelled() || Instant::now() >= expires {
        return Ok(());
    }
    if keyboard.as_mut().is_some_and(Keyboard::is_closed) {
        *keyboard = None;
    }
    let Some(input) = keyboard.as_ref() else {
        // The gesture that opens a permission dialog never becomes delayed input.
        let authorized = Keyboard::new(cancelled)?;
        if !cancelled() {
            *keyboard = Some(authorized);
            eprintln!("Keyboard access granted. Press the dock button again to send the shortcut.");
        }
        return Ok(());
    };

    let mut send_failed = false;
    let result = dispatch(ActionPlan::Hotkey(keys), |event| {
        let Output::Key(key, direction) = event else { unreachable!() };
        if matches!(direction, Direction::Press) {
            anyhow::ensure!(!cancelled(), "dock context changed");
        }
        input.key(key, direction).inspect_err(|_| send_failed = true)
    });
    // A closed session fails a send. Release held keys first, then discard it.
    if send_failed {
        *keyboard = None;
    }
    result
}

fn token() -> String {
    format!("ulanzi{}_{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed))
}

async fn request<B: serde::Serialize + zbus::zvariant::DynamicType>(
    bus: &zbus::Connection, method: &str, token: &str, body: &B,
    deadline: Instant, cancelled: &impl Fn() -> bool,
) -> Result<Results> {
    anyhow::ensure!(!cancelled(), "keyboard authorization cancelled");
    anyhow::ensure!(Instant::now() < deadline, "keyboard authorization timed out");
    let sender = bus.unique_name().context("portal connection has no unique name")?
        .as_str().trim_start_matches(':').replace('.', "_");
    let path = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");
    let proxy = zbus::Proxy::new(bus, SERVICE, path.as_str(), "org.freedesktop.portal.Request").await?;
    let mut replies = proxy.receive_signal("Response").await?;
    anyhow::ensure!(!cancelled(), "keyboard authorization cancelled");
    let reply = bus.call_method(Some(SERVICE), PATH, Some(INTERFACE), method, body).await?;
    let returned: OwnedObjectPath = reply.body().deserialize()
        .with_context(|| format!("decoding the {method} request handle"))?;
    anyhow::ensure!(returned.as_str() == path, "portal returned an unexpected request handle");
    let response = future::or(
        async { replies.next().await.context("portal response stream closed") },
        async {
            while Instant::now() < deadline {
                anyhow::ensure!(!cancelled(), "keyboard authorization cancelled");
                async_io::Timer::after(Duration::from_millis(50)).await;
            }
            anyhow::bail!("keyboard authorization timed out")
        },
    ).await;
    let message = match response {
        Ok(message) => message,
        Err(error) => { let _ = proxy.call_method("Close", &()).await; return Err(error); }
    };
    let (code, results): (u32, Results) = message.body().deserialize()
        .with_context(|| format!("decoding the {method} response"))?;
    anyhow::ensure!(code == 0, "keyboard authorization cancelled or denied (response {code})");
    Ok(results)
}

struct Keyboard {
    bus: Connection,
    session: OwnedObjectPath,
    closed: SignalStream<'static>,
}
impl Keyboard {
    fn new(cancelled: &impl Fn() -> bool) -> Result<Self> {
        let deadline = Instant::now() + Duration::from_secs(90);
        future::block_on(async {
            let bus = zbus::connection::Builder::session()?
                .method_timeout(Duration::from_secs(3)).build().await?;
            let handle = token();
            let session_token = token();
            let options = HashMap::from([
                ("handle_token", Value::from(handle.as_str())),
                ("session_handle_token", Value::from(session_token.as_str())),
            ]);
            let mut results = request(&bus, "CreateSession", &handle, &(options,), deadline, cancelled).await?;
            // The portal returns session_handle as a D-Bus string in its result
            // dictionary, even though following calls require an object path.
            let handle_value = results.remove("session_handle")
                .context("portal returned no keyboard session")?;
            let session_string = String::try_from(handle_value)
                .context("decoding the portal session handle string")?;
            let session = OwnedObjectPath::try_from(session_string)
                .context("validating the portal session object path")?;
            let session_proxy = zbus::Proxy::new(&bus, SERVICE, session.as_str(), SESSION).await?;
            let closed = session_proxy.receive_signal("Closed").await?;
            drop(session_proxy);
            let handle = token();
            let options = HashMap::from([
                ("handle_token", Value::from(handle.as_str())),
                ("types", Value::from(1u32)), // Keyboard only.
            ]);
            request(&bus, "SelectDevices", &handle, &(&session, options), deadline, cancelled).await?;
            let handle = token();
            let options = HashMap::from([("handle_token", Value::from(handle.as_str()))]);
            eprintln!("Allow keyboard access in your desktop’s permission dialog to use dock shortcuts.");
            let results = request(&bus, "Start", &handle, &(&session, "", options), deadline, cancelled).await?;
            let devices = results.get("devices").and_then(|value| u32::try_from(value).ok()).unwrap_or(0);
            anyhow::ensure!(devices & 1 != 0, "The desktop did not grant keyboard access");
            Ok(Self { bus: Connection::from(bus), session, closed })
        })
    }

    fn is_closed(&mut self) -> bool {
        self.bus.is_closed() || future::block_on(future::poll_once(self.closed.next())).is_some()
    }

    fn key(&self, key: Key, direction: Direction) -> Result<()> {
        let symbol = keysym(key)?;
        let states: &[u32] = match direction { Direction::Press => &[1], Direction::Release => &[0], Direction::Click => &[1, 0] };
        for state in states {
            self.bus.call_method(Some(SERVICE), PATH, Some(INTERFACE), "NotifyKeyboardKeysym",
                &(&self.session, HashMap::<&str, Value<'_>>::new(), symbol as i32, *state))
                .context("sending the shortcut through the authorized Wayland session")?;
        }
        Ok(())
    }
}
impl Drop for Keyboard {
    fn drop(&mut self) {
        let _ = self.bus.call_method(Some(SERVICE), self.session.as_str(),
            Some(SESSION), "Close", &());
    }
}

fn keysym(key: Key) -> Result<u32> {
    Ok(match key {
        Key::Control => 0xffe3, Key::Shift => 0xffe1, Key::Alt => 0xffe9, Key::Meta => 0xffeb,
        Key::Tab => 0xff09, Key::Return => 0xff0d, Key::Escape => 0xff1b, Key::Space => 0x20,
        Key::F2 => 0xffbf, Key::F5 => 0xffc2, Key::F7 => 0xffc4, Key::F9 => 0xffc6, Key::F12 => 0xffc9,
        Key::Unicode(c) if u32::from(c) <= 0xff => u32::from(c),
        Key::Unicode(c) => 0x01000000 | u32::from(c),
        _ => anyhow::bail!("unsupported Wayland shortcut key: {key:?}"),
    })
}
