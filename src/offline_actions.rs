// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.
// Physical order and expected effects for the shared Linux reference profile.
// The oracle specifies output events, independently of the configured action strings.

use crate::{
    actions::{simulate, RecordedOutput},
    controller::{route_button, ButtonRoute},
    dock::ButtonEvent,
    model::{Config, CLOCK_KEY, PAGE_INDEX_NAME},
};
use anyhow::Result;
use enigo::{Direction, Key};
use std::{collections::BTreeSet, path::PathBuf};

const PAGES: [(&str, &str); 10] = [
    ("Windows / media", "PlayPause Previous Next ToggleMute > VolumeDown VolumeUp Files ShowDesktop Settings Screenshot TaskView https://www.google.com #"),
    ("Utility", "brighter dimmer Emoji Screenshot > Files ShowDesktop TaskView Settings Clipboard Control+c Control+v Search #"),
    ("Codex", "Control+n Control+Alt+Shift+o Control+Alt+a Control+Alt+s > Control+Shift+g Control+` Control+p Control+Shift+p Control+Alt+l Control+c Control+v Control+f #"),
    ("VS Code", "Control+p Control+Shift+p Control+` Control+Shift+f > Control+Shift+e Control+Shift+g Control+Shift+d Control+Shift+x Control+s Control+n Shift+Alt+f Control+Shift+m #"),
    ("Spotify", "Space Previous Next Control+s > Control+r Control+k Alt+Shift+0 Alt+Shift+b Alt+Shift+q Alt+Shift+j Alt+Shift+s Alt+Shift+h #"),
    ("Word", "Control+s Control+o Control+n Control+p > Control+z Control+y Control+f Control+b Control+i Control+u F7 F12 #"),
    ("PowerPoint", "Control+s Control+o Control+n Control+p > Control+z Control+y Control+f F5 Shift+F5 Control+m Control+Shift+d Escape #"),
    ("Excel", "Control+s Control+o Control+n Control+p > Control+z Control+y Control+f Alt+= Control+1 Control+; F2 Control+Shift+l #"),
    ("OBS Creator", "Control+Shift+Alt+1 Control+Shift+Alt+2 Control+Shift+Alt+3 Control+Shift+Alt+4 > Control+Shift+Alt+5 Control+Shift+Alt+6 Control+Shift+Alt+7 Control+Shift+Alt+8 Control+Shift+Alt+9 Control+Shift+Alt+0 Control+Shift+Alt+F5 Control+Shift+Alt+F7 #"),
    ("Themes", "dark-abstract light-classic light-abstract manga-ink > steampunk-brass cyberpunk-neon moire cubism art-deco ukiyo-e solarpunk memphis #"),
];

fn key_name(key: &Key) -> String {
    match key {
        Key::Unicode(c) => c.to_string(),
        other => format!("{other:?}"),
    }
}

// Verify that a chord is released in reverse order and that there are no extra events.
fn desktop_effect(action: &str) -> Result<String> {
    use RecordedOutput::*;
    let events = simulate(action)?;
    Ok(match events.as_slice() {
        [OpenUrl(url)] => url.clone(),
        [Mpris(method)] => method.to_string(),
        [ToggleMute] => "ToggleMute".into(),
        [Volume(up)] => if *up { "VolumeUp" } else { "VolumeDown" }.into(),
        [Desktop(action)] => format!("{action:?}"),
        [Key(key, Direction::Click)] => key_name(key),
        chord => {
            let (down, up) = chord.split_at(chord.len() / 2);
            assert!(!down.is_empty(), "{action}: no output");
            assert_eq!(down.len(), up.len(), "{action}: unbalanced chord");
            let mut tokens = Vec::new();
            for (press, release) in down.iter().zip(up.iter().rev()) {
                let Key(key, Direction::Press) = press else { panic!("{action}: expected press"); };
                assert_eq!(release, &Key(*key, Direction::Release), "{action}: release order");
                tokens.push(key_name(key));
            }
            tokens.join("+")
        }
    })
}

#[test]
fn all_87_actions_from_physical_press_to_recorded_effect() -> Result<()> {
    let mut config = match std::env::var_os("ULANZI_OFFLINE_PROFILE") {
        Some(path) => {
            let path = PathBuf::from(path);
            anyhow::ensure!(path.is_file(), "test profile not found: {}", path.display());
            Config::load(&path)?
        }
        None => {
            let mut config = Config::default();
            let catalog: crate::applications::Catalog = serde_yaml_ng::from_str(include_str!("../applications.yaml"))?;
            catalog.apply(&mut config)?;
            config
        }
    };
    assert_eq!(config.pages.len(), PAGES.len() + 1);
    // Enable both brightness directions in memory, regardless of the saved value.
    config.brightness = 50;
    config.refresh_key_availability();
    let mut actions = BTreeSet::new();
    for (page, (name, expected)) in PAGES.iter().enumerate() {
        assert_eq!(config.pages[page].name, *name);
        assert!(config.pages[page].enabled, "{name} is disabled in this profile");
        let positions: Vec<_> = expected.split_whitespace().collect();
        assert_eq!(positions.len(), config.pages[page].keys.len());
        for (index, effect) in positions.into_iter().enumerate() {
            let key = &config.pages[page].keys[index];
            let where_ = format!("{name}[{index}] {} ({})", key.label, key.action);
            let mut held = None;
            let mut event = |pressed| route_button(&config, page, true, true, &mut held, ButtonEvent { index, pressed });
            assert_eq!(event(false)?, ButtonRoute::Ignore, "{where_}: stray release");
            assert_eq!(event(true)?, if index == CLOCK_KEY { ButtonRoute::Ignore } else { ButtonRoute::Pressed }, "{where_}: press");
            assert_eq!(event(true)?, ButtonRoute::Ignore, "{where_}: repeated press");
            let route = event(false)?;
            match route {
                ButtonRoute::Page(target) if effect == "#" => assert_eq!(config.pages[target].name, PAGE_INDEX_NAME, "{where_}"),
                ButtonRoute::Page(target) if effect == ">" => assert_eq!(target, (page + 1) % PAGES.len(), "{where_}"),
                ButtonRoute::Brightness(up) => assert_eq!(if up { "brighter" } else { "dimmer" }, effect, "{where_}"),
                ButtonRoute::Theme(theme) => assert_eq!(theme.slug(), effect, "{where_}"),
                ButtonRoute::Desktop(action) => assert_eq!(desktop_effect(action)?, effect, "{where_}"),
                other => panic!("{where_}: unexpected {other:?}, expected {effect}"),
            }
            assert_eq!(event(false)?, ButtonRoute::Ignore, "{where_}: repeated release");
            if index != CLOCK_KEY { actions.insert(key.action.as_str()); }
        }
    }
    assert_eq!(actions.len(), 87);
    println!("87 distinct actions verified at 140 positions on 10 pages with simulated input and output.");
    Ok(())
}
