// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Change the default PipeWire playback device without keyboard injection.

use anyhow::{Context, Result};
use std::{
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub(super) fn toggle_mute() -> Result<()> {
    run(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
}

pub(super) fn volume(up: bool) -> Result<()> {
    run(&["set-volume", "@DEFAULT_AUDIO_SINK@", if up { "5%+" } else { "5%-" }, "--limit", "1.0"])
}

fn run(args: &[&str]) -> Result<()> {
    let mut child = Command::new("wpctl")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .context("starting wpctl; install WirePlumber for Linux audio control")?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                anyhow::ensure!(status.success(), "PipeWire audio command failed ({status})");
                return Ok(());
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                result.context("waiting for the PipeWire audio command")?;
                anyhow::bail!("PipeWire audio command timed out");
            }
        }
    }
}
