/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent. AI implementation. Accountable human review.
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 * AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
 * AI Versions: Tools and models may vary by contributor and execution environment.
 * AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 * VBWR E */

use crate::{
    art,
    model::{PageConfig, Theme, CLOCK_KEY, KEY_COUNT, NEXT_KEY},
};
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use serde_json::{json, Map, Value};
use std::{
    io::{Cursor, Write},
    time::Duration,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

const VENDOR_ID: u16 = 0x2207;
const PRODUCT_ID: u16 = 0x0019;
const PACKET: usize = 1024;

pub struct Dock {
    device: HidDevice,
}

pub struct ButtonEvent {
    pub index: usize,
    pub pressed: bool,
}

// Errors: USB enumeration failure or missing D200H consumer interface.
pub fn discover() -> Result<(HidApi, bool)> {
    let api = HidApi::new().context("enumerating HID devices")?;
    let found = api.device_list().any(|d| {
        d.vendor_id() == VENDOR_ID
            && d.product_id() == PRODUCT_ID
            && d.usage_page() == 0x000c
            && d.usage() == 0x0001
    });
    Ok((api, found))
}

impl Dock {
    // api: enumerated HID context.
    // Errors: missing consumer interface or failed exclusive/open access.
    pub fn open(api: &HidApi) -> Result<Self> {
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VENDOR_ID
                    && d.product_id() == PRODUCT_ID
                    && d.usage_page() == 0x000c
                    && d.usage() == 0x0001
            })
            .context("D200H consumer HID interface (usage page 0x0c, usage 1) not found")?;
        let device = info
            .open_device(api)
            .context("opening D200H consumer HID interface")?;
        Ok(Self { device })
    }

    // page: page to show on all 14 display keys.
    // theme: selected artwork family.
    // held: pressed key to draw; all other keys use up artwork.
    // Errors: image, archive, or USB write failure.
    pub fn show(
        &self,
        page: &PageConfig,
        theme: Theme,
        held: Option<usize>,
        enabled: bool,
        status_display: bool,
    ) -> Result<()> {
        let archive = make_archive(page, theme, held, enabled, status_display)?;
        if status_display {
            self.background()?;
        }
        self.send_file(0x0001, &archive)?;
        if status_display {
            self.background()?;
        }
        Ok(())
    }

    // Update only the pressed or released display key. Page changes still use show().
    // Errors: invalid key, image/archive generation, or USB write failure.
    pub fn show_key(
        &self,
        page: &PageConfig,
        theme: Theme,
        index: usize,
        pressed: bool,
        enabled: bool,
    ) -> Result<()> {
        let archive = make_key_archive(page, theme, index, pressed, enabled)?;
        self.send_file(0x000d, &archive)
    }

    // A focus change replaces only the twelve application keys, preserving
    // NEXT PAGE and the clock. No updates are sent while availability is stable.
    pub fn refresh_availability(
        &self,
        page: &PageConfig,
        theme: Theme,
        held: Option<usize>,
        enabled: bool,
        status_display: bool,
    ) -> Result<()> {
        let mut keys = (0..CLOCK_KEY)
            .filter(|&i| i != NEXT_KEY)
            .collect::<Vec<_>>();
        if status_display {
            keys.push(CLOCK_KEY);
        }
        let archive = make_archive_for_keys(page, theme, held, &keys, enabled, status_display)?;
        self.send_file(0x000d, &archive)?;
        if status_display {
            self.background()?;
        }
        Ok(())
    }

    // Keep the wide panel in custom-background mode without clock overlays.
    // Use the same ASCII field encoding as clock mode. A binary mode byte can
    // leave native statistic overlays visible on the owner's firmware.
    pub fn background(&self) -> Result<()> {
        let payload = b"2|0|0|00:00:00|0";
        self.send_packet(0x0006, payload, payload.len() as u32)
    }

    // Errors: clock encoding or USB write failure.
    pub fn clock(&self) -> Result<()> {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        let payload = format!("1|0|0|{}|0", now);
        self.send_packet(0x0006, payload.as_bytes(), payload.len() as u32)
    }

    // timeout: bounded wait so the clock and Ctrl+C remain responsive.
    // Errors: USB input failure.
    pub fn read_button(&self, timeout: Duration) -> Result<Option<ButtonEvent>> {
        let mut buffer = [0u8; PACKET + 1];
        let count = self
            .device
            .read_timeout(&mut buffer, timeout.as_millis() as i32)?;
        let bytes = &buffer[..count];
        for offset in [0, 1] {
            if bytes.len() < offset + 12 || bytes[offset..offset + 2] != [0x7c, 0x7c] {
                continue;
            }
            let command = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
            if command != 0x0101 && command != 0x0102 {
                continue;
            }
            let index = bytes[offset + 9] as usize;
            if index >= KEY_COUNT {
                return Ok(None);
            }
            return Ok(Some(ButtonEvent {
                index,
                pressed: bytes[offset + 11] == 1,
            }));
        }
        Ok(None)
    }

    // command: full layout (0x0001) or one-key additive update (0x000d).
    // archive: ZIP containing the intended manifest positions.
    // Errors: oversized ZIP or USB write failure.
    fn send_file(&self, command: u16, archive: &[u8]) -> Result<()> {
        anyhow::ensure!(
            archive.len() <= u32::MAX as usize,
            "layout ZIP exceeds protocol length"
        );
        self.send_packet(
            command,
            &archive[..archive.len().min(1016)],
            archive.len() as u32,
        )?;
        for chunk in archive[archive.len().min(1016)..].chunks(PACKET) {
            let mut report = [0u8; PACKET + 1];
            report[1..1 + chunk.len()].copy_from_slice(chunk);
            self.write(&report)?;
        }
        Ok(())
    }

    // command: D200H command number.
    // payload: up to 1016 bytes of command data.
    // length: declared total payload length.
    // Errors: oversized payload or USB write failure.
    fn send_packet(&self, command: u16, payload: &[u8], length: u32) -> Result<()> {
        anyhow::ensure!(payload.len() <= 1016, "command payload exceeds one packet");
        let mut report = [0u8; PACKET + 1];
        report[1..3].copy_from_slice(&[0x7c, 0x7c]);
        report[3..5].copy_from_slice(&command.to_be_bytes());
        report[5..9].copy_from_slice(&length.to_le_bytes());
        report[9..9 + payload.len()].copy_from_slice(payload);
        self.write(&report)
    }

    // report: Report ID 0 followed by exactly 1024 protocol bytes.
    // Errors: failed or short USB write.
    fn write(&self, report: &[u8; PACKET + 1]) -> Result<()> {
        let count = self
            .device
            .write(report)
            .context("writing D200H HID report")?;
        anyhow::ensure!(
            count == report.len(),
            "short HID write: {count} of {} bytes",
            report.len()
        );
        Ok(())
    }
}

// page: source labels for all physical positions.
// theme: selected artwork family.
// held: currently pressed key, if any.
// Errors: PNG or ZIP generation failure; archive boundary retry exhaustion.
pub fn make_archive(
    page: &PageConfig,
    theme: Theme,
    held: Option<usize>,
    enabled: bool,
    status_display: bool,
) -> Result<Vec<u8>> {
    make_archive_for_keys(
        page,
        theme,
        held,
        &(0..KEY_COUNT).collect::<Vec<_>>(),
        enabled,
        status_display,
    )
}

// Build an additive archive containing exactly one ordinary display key.
pub fn make_key_archive(
    page: &PageConfig,
    theme: Theme,
    index: usize,
    pressed: bool,
    enabled: bool,
) -> Result<Vec<u8>> {
    anyhow::ensure!(
        index < CLOCK_KEY,
        "key {index} is not an ordinary display key"
    );
    make_archive_for_keys(
        page,
        theme,
        pressed.then_some(index),
        &[index],
        enabled,
        false,
    )
}

fn make_archive_for_keys(
    page: &PageConfig,
    theme: Theme,
    held: Option<usize>,
    keys: &[usize],
    enabled: bool,
    status_display: bool,
) -> Result<Vec<u8>> {
    let archive_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let pictures = keys
        .iter()
        .copied()
        .map(|i| {
            if i == CLOCK_KEY && status_display {
                art::render_status(theme, page, enabled)
            } else {
                art::render_available(theme, page, i, held == Some(i), enabled)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut base = ZipWriter::new(Cursor::new(Vec::new()));
    let mut manifest = Map::new();
    for (&index, picture) in keys.iter().zip(pictures.iter()) {
        // New names make the firmware load each replacement icon.
        let name = if index == CLOCK_KEY && !status_display {
            String::new()
        } else {
            let name = format!("Images/key-{index:02}-{archive_id:x}.png");
            base.start_file(&name, options)?;
            base.write_all(picture)?;
            name
        };
        let position = format!("{}_{}", index % 5, index / 5);
        let mut entry = json!({"State":0,"ViewParam":[{"Text":"","Icon":name}]});
        if index == CLOCK_KEY && status_display {
            entry["SmallViewMode"] = json!(2);
        }
        manifest.insert(position, entry);
    }
    base.start_file("manifest.json", options)?;
    base.write_all(&serde_json::to_vec(&Value::Object(manifest))?)?;
    base.start_file("sentinel.txt", options)?;
    let base_bytes = base.finish()?.into_inner();
    let mut source = zip::ZipArchive::new(Cursor::new(base_bytes))?;
    let dummy_options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for attempt in 0..1024u32 {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        // A stored dummy shifts every later ZIP byte by exactly one per attempt.
        let filler = vec![b'A'; attempt as usize];
        zip.start_file("dummy.txt", dummy_options)?;
        zip.write_all(&filler)?;
        for index in 0..source.len() {
            zip.raw_copy_file(source.by_index(index)?)?;
        }
        let data = zip.finish()?.into_inner();
        if (1016..data.len())
            .step_by(PACKET)
            .all(|offset| data[offset] != 0 && data[offset] != 0x7c)
        {
            return Ok(data);
        }
    }
    anyhow::bail!("could not satisfy D200H ZIP continuation boundary constraint")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Config;
    use std::io::Read;

    #[test]
    fn full_layout_contains_thirteen_icons_and_safe_boundaries() -> Result<()> {
        let page = &Config::default().pages[0];
        let bytes = make_archive(page, Theme::DarkClassic, Some(4), true, false)?;
        assert!((1016..bytes.len())
            .step_by(PACKET)
            .all(|n| bytes[n] != 0 && bytes[n] != 0x7c));
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
        let mut manifest = String::new();
        zip.by_name("manifest.json")?
            .read_to_string(&mut manifest)?;
        let positions: Value = serde_json::from_str(&manifest)?;
        assert_eq!(positions.as_object().expect("object").len(), KEY_COUNT);
        assert!(positions["4_0"]["ViewParam"][0]["Icon"]
            .as_str()
            .expect("icon name")
            .starts_with("Images/key-04-"));
        assert_eq!(positions["3_2"]["ViewParam"][0]["Icon"], "");
        for index in 0..CLOCK_KEY {
            let mut png = Vec::new();
            let position = format!("{}_{}", index % 5, index / 5);
            let name = positions[&position]["ViewParam"][0]["Icon"]
                .as_str()
                .expect("icon name");
            zip.by_name(name)?.read_to_end(&mut png)?;
            let image = image::load_from_memory(&png)?;
            assert_eq!((image.width(), image.height()), (196, 196));
        }
        Ok(())
    }

    #[test]
    fn replacement_layout_uses_fresh_icon_names() -> Result<()> {
        let page = &Config::default().pages[0];
        let mut names = Vec::new();
        for _ in 0..2 {
            let data = make_archive(page, Theme::LightAbstract, None, true, false)?;
            let mut zip = zip::ZipArchive::new(Cursor::new(data))?;
            let mut manifest = String::new();
            zip.by_name("manifest.json")?
                .read_to_string(&mut manifest)?;
            let positions: Value = serde_json::from_str(&manifest)?;
            names.push(positions["0_0"]["ViewParam"][0]["Icon"].clone());
        }
        assert_ne!(names[0], names[1]);
        Ok(())
    }

    #[test]
    fn additive_update_contains_only_the_selected_key_and_both_states() -> Result<()> {
        let page = &Config::default().pages[0];
        let mut images = Vec::new();
        for pressed in [false, true] {
            let bytes = make_key_archive(page, Theme::LightAbstract, 4, pressed, true)?;
            assert!((1016..bytes.len())
                .step_by(PACKET)
                .all(|n| bytes[n] != 0 && bytes[n] != 0x7c));
            let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
            let mut manifest = String::new();
            zip.by_name("manifest.json")?
                .read_to_string(&mut manifest)?;
            let positions: Value = serde_json::from_str(&manifest)?;
            let entries = positions.as_object().expect("manifest object");
            assert_eq!(entries.len(), 1);
            let name = positions["4_0"]["ViewParam"][0]["Icon"]
                .as_str()
                .expect("selected icon name");
            let mut png = Vec::new();
            zip.by_name(name)?.read_to_end(&mut png)?;
            let image = image::load_from_memory(&png)?;
            assert_eq!((image.width(), image.height()), (196, 196));
            images.push(png);
        }
        assert_ne!(images[0], images[1]);
        assert!(make_key_archive(page, Theme::LightAbstract, CLOCK_KEY, false, true).is_err());
        Ok(())
    }

    #[test]
    fn every_default_page_and_theme_can_be_packaged() -> Result<()> {
        for page in &Config::default().pages {
            for theme in Theme::ALL {
                let data = make_archive(page, theme, None, true, false)?;
                assert!(data.starts_with(b"PK"));
                assert!((1016..data.len())
                    .step_by(PACKET)
                    .all(|offset| data[offset] != 0 && data[offset] != 0x7c));
            }
        }
        Ok(())
    }

    #[test]
    fn inactive_focus_update_is_grayscale_and_preserves_navigation_and_clock() -> Result<()> {
        let page = &Config::default().pages[2];
        let keys = (0..CLOCK_KEY)
            .filter(|&i| i != NEXT_KEY)
            .collect::<Vec<_>>();
        let bytes = make_archive_for_keys(page, Theme::CyberpunkNeon, None, &keys, false, false)?;
        assert!((1016..bytes.len())
            .step_by(PACKET)
            .all(|n| bytes[n] != 0 && bytes[n] != 0x7c));
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
        let mut manifest = String::new();
        zip.by_name("manifest.json")?
            .read_to_string(&mut manifest)?;
        let positions: Value = serde_json::from_str(&manifest)?;
        assert_eq!(positions.as_object().expect("manifest object").len(), 12);
        assert!(positions.get("4_0").is_none());
        assert!(positions.get("3_2").is_none());
        for entry in positions.as_object().expect("manifest object").values() {
            let mut png = Vec::new();
            zip.by_name(entry["ViewParam"][0]["Icon"].as_str().expect("icon"))?
                .read_to_end(&mut png)?;
            let image = image::load_from_memory(&png)?.to_rgba8();
            assert_eq!(image.dimensions(), (196, 196));
            assert!(image
                .pixels()
                .all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
        }
        assert_eq!(
            art::render_available(Theme::CyberpunkNeon, page, NEXT_KEY, false, false)?,
            art::render_available(Theme::CyberpunkNeon, page, NEXT_KEY, false, true)?
        );
        assert_eq!(
            art::render_available(Theme::CyberpunkNeon, page, 0, true, false)?,
            art::render_available(Theme::CyberpunkNeon, page, 0, false, false)?
        );
        Ok(())
    }

    #[test]
    fn custom_status_panel_uses_wide_geometry_and_background_mode() -> Result<()> {
        let page = &Config::default().pages[3];
        let mut panels = Vec::new();
        for enabled in [false, true] {
            let bytes = make_archive(page, Theme::LightAbstract, None, enabled, true)?;
            let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
            let mut manifest = String::new();
            zip.by_name("manifest.json")?
                .read_to_string(&mut manifest)?;
            let positions: Value = serde_json::from_str(&manifest)?;
            assert_eq!(positions["3_2"]["SmallViewMode"], 2);
            let name = positions["3_2"]["ViewParam"][0]["Icon"]
                .as_str()
                .expect("wide panel icon");
            let mut png = Vec::new();
            zip.by_name(name)?.read_to_end(&mut png)?;
            let image = image::load_from_memory(&png)?.to_rgba8();
            assert_eq!(image.dimensions(), (400, 200));
            assert!(image.pixels().all(|p| p[3] == 255));
            panels.push(png);
        }
        assert_ne!(panels[0], panels[1]);
        Ok(())
    }
}
