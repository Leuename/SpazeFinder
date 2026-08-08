# Third-Party Notices

SpazeFinder is licensed under the MIT License (see [LICENSE](LICENSE)). That
license covers this project's own code only. Bundled and statically linked
third-party components remain under their own licenses, reproduced or referenced
below.

Generated for SpazeFinder 0.1.0 from `Cargo.lock`, resolved for the
`x86_64-pc-windows-msvc` target.

## Frontend

The user interface is vanilla HTML, CSS, and JavaScript with no dependencies, no
build step, and no bundled web framework. No web fonts are bundled; the interface
uses system font stacks only.

## Rust dependencies

Direct dependencies declared in `src-tauri/Cargo.toml`:

| Crate | License |
|---|---|
| `tauri` 2 | MIT OR Apache-2.0 |
| `tauri-build` 2 | MIT OR Apache-2.0 |
| `tauri-plugin-dialog` 2 | MIT OR Apache-2.0 |
| `serde`, `serde_json` 1 | MIT OR Apache-2.0 |
| `rayon` 1 | MIT OR Apache-2.0 |
| `trash` 5 | MIT |
| `windows-sys` 0.59 | MIT OR Apache-2.0 |
| `tempfile` 3 (dev only, not shipped) | MIT OR Apache-2.0 |

These pull in 267 crates in total for the Windows build. License breakdown of the
full resolved set:

| License | Crates |
|---|---|
| MIT OR Apache-2.0 (all spellings) | majority |
| MIT | large minority |
| Unicode-3.0 | 18 |
| Zlib OR Apache-2.0 OR MIT | 17 |
| Apache-2.0 WITH LLVM-exception (dual with MIT/Apache) | 5 |
| MPL-2.0 | 5 |
| BSD-3-Clause (alone or dual) | 3 |
| ISC, Zlib, 0BSD, CC0-1.0, Unlicense (dual options) | remainder |

No GPL, LGPL, AGPL, or license-unknown crate is present in the Windows dependency
set.

### MPL-2.0 components

The following crates are licensed under the Mozilla Public License 2.0. MPL-2.0 is
file-level copyleft: linking them into an MIT-licensed binary is permitted, and
their source is available unmodified from crates.io. SpazeFinder does not modify
these crates. If a future version does modify them, the modified files must be
published under MPL-2.0.

- `cssparser` 0.36.0
- `cssparser-macros` 0.6.1
- `dtoa-short` 0.3.5
- `option-ext` 0.2.0
- `selectors` 0.36.1

MPL-2.0 text: <https://www.mozilla.org/MPL/2.0/>

### Unicode-3.0 components

18 crates (the `icu_*` / `unicode-*` family used by URL and text handling) are
licensed under the Unicode License v3, which requires the Unicode copyright and
permission notice to accompany distribution.

Unicode-3.0 text: <https://www.unicode.org/license.txt>

### Full per-crate list

Regenerate the complete, authoritative list before each release:

```powershell
cd src-tauri
cargo install cargo-about
cargo about generate --format json > ..\third-party-licenses.json
```

or verify the license and advisory policy with:

```powershell
cargo install cargo-deny
cargo deny check
```

The policy lives in `src-tauri/deny.toml`, which pins the check to the
`x86_64-pc-windows-msvc` target and lists the allowed licenses. As of
SpazeFinder 0.1.0 it reports `advisories ok, bans ok, licenses ok, sources ok`,
and `cargo audit` reports no vulnerabilities.

## Runtime components (not bundled)

SpazeFinder uses Microsoft Edge WebView2 to render its interface. WebView2 is a
Microsoft component installed with or alongside Windows and is not redistributed
by this project. It is governed by the Microsoft Edge WebView2 Runtime license
terms.

## Assets

| Asset | Source | License |
|---|---|---|
| `src-tauri/icons/icon.ico` | Original project artwork, generated from `icons/icons.png` | MIT (part of this project) |

No fonts, sounds, or third-party images are bundled.
