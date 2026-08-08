# SpazeFinder

The fastest path from "disk full" to "space freed" — scan, see, act, done.

SpazeFinder is a Windows desktop app that scans a whole drive and shows what takes up the space as a biggest-first tree, with Explorer-like actions so cleanup happens in place. Built with Tauri 2: a parallel Rust scanner behind a dependency-free vanilla JS interface.

## Features

- **Full-drive scan with admin rights** — the release build elevates via UAC at launch so protected folders are counted; scans millions of files in parallel (rayon)
- **Biggest-first tree** — every level sorted by size, with gradient bars showing each item's share of its parent
- **Act in place, like Explorer**
  - Double-click a file to open it (de-elevated through the shell, so nothing inherits admin rights)
  - Right-click: Open, Reveal in Explorer, Rename (inline, extension preserved), Move to…, Delete
  - Delete goes to the **Recycle Bin only** — nothing in the app deletes permanently
- **Live scan readout** — odometer-style byte and file counters while scanning
- **Dual theme** — light (`#eff0d1`) and dark (`#262730`), toggle persisted, monochrome base with a single teal→lime gradient carrying the data
- **Instant navigation** — the scan tree lives in memory; expanding folders, deleting, renaming, and moving update sizes without a rescan

## Building

Requires the [Rust toolchain](https://rustup.rs) (stable, MSVC) on Windows.

```powershell
cd src-tauri
cargo build --release
```

The executable lands at `src-tauri/target/release/spaze-finder.exe`. It requests admin elevation at launch (release builds only — debug builds skip the manifest so `cargo test` can run).

### Tests

```powershell
cd src-tauri
cargo test
```

### Note for contributors

Frontend assets (`src/`) are embedded into the binary at compile time. After JS/CSS-only changes, touch `src-tauri/src/main.rs` (update its modified time) before `cargo build`, or the binary ships stale assets.

## Tech

- [Tauri 2](https://tauri.app) — Rust backend, WebView2 frontend
- Rust: `rayon` (parallel scan), `trash` (Recycle Bin), `windows-sys` (drive enumeration)
- Frontend: vanilla HTML/CSS/JS, no build step, no dependencies

## Privacy

SpazeFinder processes filesystem information locally on your device. It makes no
network requests. Official releases include no telemetry, analytics, advertising,
crash reporting, accounts, or cloud scan storage, and write no scan history to
disk. The only thing stored between sessions is your theme preference.

The scanner reads directory listings and file sizes — it does not open or read
file contents.

See [PRIVACY.md](PRIVACY.md).

## Responsible Use

Use SpazeFinder only on devices and filesystems that you own or are authorized to
access.

Review files carefully before moving, renaming, or deleting them, and keep backups
of important data. Delete sends items to the Recycle Bin; Recycle Bin recovery is
not guaranteed.

Release builds run elevated — see [DISCLAIMER.md](DISCLAIMER.md).

## Security

Report vulnerabilities privately per [SECURITY.md](SECURITY.md). Do not include
real personal files or credentials in a report.

## Third-Party Licenses

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Official Releases

Official builds are published only at
<https://github.com/Leuename/SpazeFinder/releases>. Verify downloads against the
published SHA-256 checksums.

## License

Copyright © 2026 Emmanuel Millave.

SpazeFinder is free and open-source software licensed under the MIT License.
See [LICENSE](LICENSE) for details.
