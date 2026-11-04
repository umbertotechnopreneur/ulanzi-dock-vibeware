// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

use crate::{
    dock::ButtonEvent,
    model::{Config, Theme, CLOCK_KEY, NEXT_KEY},
};
use anyhow::{Context, Result};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ButtonRoute<'a> {
    Ignore,
    Pressed,
    Released,
    Page(usize),
    Theme(Theme),
    Brightness(bool),
    Desktop(&'a str),
}

/// Resolve a validated dock report before drawing, persisting state or sending OS input.
pub(crate) fn route_button<'a>(
    config: &'a Config,
    current_page: usize,
    active: bool,
    actions_enabled: bool,
    held: &mut Option<usize>,
    event: ButtonEvent,
) -> Result<ButtonRoute<'a>> {
    let index = event.index;
    let page = &config.pages[current_page];
    if index >= page.keys.len() {
        return Ok(ButtonRoute::Ignore);
    }
    let available =
        index == CLOCK_KEY || ((active || index == NEXT_KEY) && page.key_available(index));
    if event.pressed {
        if !available || *held == Some(index) {
            return Ok(ButtonRoute::Ignore);
        }
        *held = Some(index);
        return Ok(if index == CLOCK_KEY {
            ButtonRoute::Ignore
        } else {
            ButtonRoute::Pressed
        });
    }

    if *held != Some(index) {
        return Ok(ButtonRoute::Ignore);
    }
    *held = None;
    if !available {
        return Ok(ButtonRoute::Ignore);
    }
    if index == CLOCK_KEY {
        return Ok(ButtonRoute::Page(config.page_index_position()));
    }
    if index == NEXT_KEY {
        return Ok(ButtonRoute::Page(config.next_enabled_page(current_page)));
    }

    let action = &page.keys[index].action;
    if let Some(name) = action.strip_prefix("page:go:") {
        let destination = config
            .pages
            .iter()
            .position(|target| target.enabled && target.name == name);
        return Ok(destination.map_or(ButtonRoute::Released, ButtonRoute::Page));
    }
    if let Some(slug) = action.strip_prefix("theme:") {
        let theme = Theme::from_slug(slug)
            .with_context(|| format!("unknown selected theme '{slug}'"))?;
        return Ok(ButtonRoute::Theme(theme));
    }
    if !actions_enabled {
        return Ok(ButtonRoute::Released);
    }
    Ok(match action.as_str() {
        "brightness:up" => ButtonRoute::Brightness(true),
        "brightness:down" => ButtonRoute::Brightness(false),
        _ => ButtonRoute::Desktop(action),
    })
}

#[cfg(test)]
mod offline_actions {
    use super::*;
    use crate::model::THEME_KEYS;

    fn event(index: usize, pressed: bool) -> ButtonEvent {
        ButtonEvent { index, pressed }
    }

    fn click(config: &Config, page: usize, index: usize, active: bool) -> ButtonRoute<'_> {
        let mut held = None;
        route_button(config, page, active, true, &mut held, event(index, true)).unwrap();
        route_button(config, page, active, true, &mut held, event(index, false)).unwrap()
    }

    #[test]
    fn one_matching_release_per_accepted_press() {
        use ButtonRoute::*;
        let config = Config::default();
        let mut held = None;
        let mut report = |index, pressed| {
            route_button(&config, 0, true, true, &mut held, event(index, pressed)).unwrap()
        };
        assert_eq!(report(0, false), Ignore);
        assert_eq!(report(0, true), Pressed);
        assert_eq!(report(0, true), Ignore);
        assert_eq!(report(2, false), Ignore);
        assert_eq!(report(0, false), Desktop("media:play-pause"));
        assert_eq!(report(0, false), Ignore);
        assert_eq!(report(0, true), Pressed);
        assert_eq!(report(2, true), Pressed);
        assert_eq!(report(0, false), Ignore);
        assert_eq!(report(2, false), Desktop("media:next"));
    }

    #[test]
    fn inactive_pages_keep_navigation_and_skip_disabled_targets() {
        let mut config = Config::default();
        config.pages[1].enabled = false;
        config.refresh_key_availability();
        let index = config.page_index_position();
        for page in 0..config.pages.len() {
            assert_eq!(
                click(&config, page, CLOCK_KEY, false),
                ButtonRoute::Page(index)
            );
        }
        assert_eq!(click(&config, 0, 0, false), ButtonRoute::Ignore);
        assert_eq!(click(&config, 0, NEXT_KEY, false), ButtonRoute::Page(2));
        // Themes is last; NEXT wraps past the generated index.
        assert_eq!(click(&config, 9, NEXT_KEY, true), ButtonRoute::Page(0));
        for (page, key) in THEME_KEYS.into_iter().take(10).enumerate() {
            let expected = if page == 1 {
                ButtonRoute::Ignore
            } else {
                ButtonRoute::Page(page)
            };
            assert_eq!(click(&config, index, key, true), expected);
        }
    }

    #[test]
    fn unavailable_brightness_keys_never_dispatch() {
        let mut config = Config::default();
        for (level, plus, minus) in [
            (0, ButtonRoute::Brightness(true), ButtonRoute::Ignore),
            (50, ButtonRoute::Brightness(true), ButtonRoute::Brightness(false)),
            (100, ButtonRoute::Ignore, ButtonRoute::Brightness(false)),
        ] {
            config.brightness = level;
            config.refresh_key_availability();
            assert_eq!(click(&config, 1, 0, true), plus);
            assert_eq!(click(&config, 1, 1, true), minus);
        }
    }

    #[test]
    fn no_actions_and_lost_focus_suppress_desktop_dispatch() {
        let config = Config::default();
        for (active, enabled, expected) in [
            (true, false, ButtonRoute::Released),
            (false, true, ButtonRoute::Ignore),
        ] {
            let mut held = None;
            route_button(&config, 0, true, enabled, &mut held, event(0, true)).unwrap();
            let released = route_button(&config, 0, active, enabled, &mut held, event(0, false));
            assert_eq!(released.unwrap(), expected);
            assert_eq!(held, None);
        }
    }

    #[test]
    fn each_theme_offers_twelve_other_themes_in_physical_order() {
        let mut config = Config::default();
        for current in Theme::ALL {
            config.refresh_theme_selector(current).unwrap();
            let alternatives = Theme::ALL.into_iter().filter(|theme| *theme != current);
            for (key, theme) in THEME_KEYS.into_iter().zip(alternatives) {
                assert_eq!(click(&config, 9, key, true), ButtonRoute::Theme(theme));
            }
        }
    }
}
