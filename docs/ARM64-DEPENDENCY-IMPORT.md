<!-- Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT. -->

# ARM64 dependency import — 27 October 2026

The local ARM64 build needed three already locked versions of Microsoft's
`windows_aarch64_msvc` crate. No dependency version or `Cargo.lock` entry changed.

| Version | Official download | SHA-256 verified against Cargo.lock |
| --- | --- | --- |
| 0.48.5 | https://static.crates.io/crates/windows_aarch64_msvc/windows_aarch64_msvc-0.48.5.crate | `dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc` |
| 0.52.6 | https://static.crates.io/crates/windows_aarch64_msvc/windows_aarch64_msvc-0.52.6.crate | `09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469` |
| 0.53.1 | https://static.crates.io/crates/windows_aarch64_msvc/windows_aarch64_msvc-0.53.1.crate | `b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006` |

All three archives and their extracted files were staged in
`tmp/arm64-first-import` before being added to the Cargo cache. The targeted scan
used Windows Defender platform 4.18.26100.4-0:

```powershell
& 'C:\ProgramData\Microsoft\Windows Defender\Platform\4.18.26100.4-0\MpCmdRun.exe' -Scan -ScanType 3 -File 'E:\UlanziDock VibeWare version\tmp\arm64-first-import'
```

The scan completed with exit code 0 and reported:
`Scanning E:\UlanziDock VibeWare version\tmp\arm64-first-import found no threats.`

The original log is `tmp/arm64-first-import-scan.log`; source URLs and checksums
are also recorded in `tmp/arm64-first-import/sources.json`. This evidence applies
only to these unchanged imports. Subsequent compilation uses Cargo offline.
