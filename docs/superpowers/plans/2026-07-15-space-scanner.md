# SpaceScanner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows desktop app that scans a drive and shows a biggest-first folder tree with Explorer-like file actions (open, reveal, recycle-bin delete, rename, move).

**Architecture:** Tauri 2 app. Rust backend owns an in-memory scan tree (built with rayon parallel walk) and all file operations; plain HTML/JS frontend lazily requests one directory level at a time via Tauri commands. App manifest requests `requireAdministrator` so the scan reaches protected folders.

**Tech Stack:** Rust, Tauri 2, rayon, trash, opener, windows-sys, tauri-plugin-dialog. Frontend: vanilla HTML/CSS/JS via `withGlobalTauri` (no Node/npm anywhere).

## Global Constraints

- Windows-only. App must request admin elevation at launch (manifest `requireAdministrator`).
- Delete is Recycle Bin ONLY (`trash` crate). No permanent delete anywhere.
- Frontend has no build step: static files in `src/`, `withGlobalTauri: true`, use `window.__TAURI__` globals.
- All directory listings returned to the UI are sorted size descending.
- Tauri command args are snake_case in Rust, camelCase in JS (`new_name` → `newName`, `dest_dir` → `destDir`).
- Repo root: `C:\Users\ASUS\space-scanner`. Run all cargo commands from `src-tauri/`.
- Commit messages end with:
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`
  `Claude-Session: https://claude.ai/code/session_016QxevbH1DjNjPmG2x3BprR`

---

### Task 1: Scaffold Tauri project with admin manifest

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src/index.html` (placeholder)
- Create: `.gitignore`

**Interfaces:**
- Consumes: nothing (first task)
- Produces: compiling Tauri app skeleton. Later tasks add modules `scan`, `state`, `commands` to `src-tauri/src/` and register commands in `main.rs`.

- [ ] **Step 1: Verify Rust toolchain**

Run: `cargo --version && rustc --version`
Expected: version output. If missing, install via `winget install Rustlang.Rustup` then `rustup default stable-msvc`.

- [ ] **Step 2: Create `.gitignore`**

```gitignore
src-tauri/target/
src-tauri/gen/
```

- [ ] **Step 3: Create `src-tauri/Cargo.toml`**

```toml
[package]
name = "space-scanner"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rayon = "1"
trash = "5"
opener = "0.7"
windows-sys = { version = "0.59", features = ["Win32_Storage_FileSystem", "Win32_Foundation"] }

[dev-dependencies]
tempfile = "3"

[profile.release]
lto = true
```

- [ ] **Step 4: Create `src-tauri/build.rs` with admin manifest**

```rust
fn main() {
    let windows = tauri_build::WindowsAttributes::new().app_manifest(
        r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
    );
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri-build");
}
```

- [ ] **Step 5: Create `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SpaceScanner",
  "version": "0.1.0",
  "identifier": "club.formura.spacescanner",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      { "title": "SpaceScanner", "width": 1000, "height": 700, "label": "main" }
    ],
    "security": { "csp": null }
  },
  "bundle": { "active": false }
}
```

- [ ] **Step 6: Create `src-tauri/capabilities/default.json`**

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default", "dialog:default"]
}
```

- [ ] **Step 7: Create `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 8: Create placeholder `src/index.html`**

```html
<!doctype html>
<html>
<head><meta charset="utf-8"><title>SpaceScanner</title></head>
<body><h1>SpaceScanner</h1></body>
</html>
```

- [ ] **Step 9: Build**

Run (from `src-tauri/`): `cargo build`
Expected: compiles clean (first build downloads crates, several minutes).

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri app with requireAdministrator manifest"
```

---

### Task 2: Scan engine (`scan.rs` — Node, parallel scan)

**Files:**
- Create: `src-tauri/src/scan.rs`
- Modify: `src-tauri/src/main.rs` (add `mod scan;`)

**Interfaces:**
- Consumes: nothing
- Produces:
  - `scan::Node { name: String, size: u64, is_dir: bool, children: Vec<Node> }` — children sorted size desc
  - `scan::Progress { files: AtomicU64, bytes: AtomicU64, denied: AtomicU64 }` (derives `Default`)
  - `scan::scan(path: &Path, prog: &Progress) -> Node`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/scan.rs` with the struct stubs and tests (implementation comes in Step 3 — start with `todo!()` in `scan`):

```rust
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    #[serde(skip)]
    pub children: Vec<Node>,
}

#[derive(Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    pub denied: AtomicU64,
}

pub fn scan(path: &Path, prog: &Progress) -> Node {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("big.bin"), vec![0u8; 1000]).unwrap();
        fs::write(dir.path().join("small.txt"), b"hi").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("mid.dat"), vec![0u8; 500]).unwrap();
        dir
    }

    #[test]
    fn scan_sums_sizes_recursively() {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        assert_eq!(root.size, 1502);
        assert!(root.is_dir);
        let sub = root.children.iter().find(|c| c.name == "sub").unwrap();
        assert_eq!(sub.size, 500);
        assert!(sub.is_dir);
    }

    #[test]
    fn scan_sorts_children_size_desc() {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        let names: Vec<&str> = root.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["big.bin", "sub", "small.txt"]);
    }

    #[test]
    fn scan_counts_progress() {
        let dir = fixture();
        let prog = Progress::default();
        scan(dir.path(), &prog);
        assert_eq!(prog.files.load(Ordering::Relaxed), 3);
        assert_eq!(prog.bytes.load(Ordering::Relaxed), 1502);
    }
}
```

Add `mod scan;` as the first line after the `#![cfg_attr...]` in `main.rs`.

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test`
Expected: 3 failures, all panicking at `not yet implemented`.

- [ ] **Step 3: Implement `scan`**

Replace the `todo!()` body:

```rust
pub fn scan(path: &Path, prog: &Progress) -> Node {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let entries = match fs::read_dir(path) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => {
            prog.denied.fetch_add(1, Ordering::Relaxed);
            return Node { name, size: 0, is_dir: true, children: vec![] };
        }
    };
    let mut children: Vec<Node> = entries
        .par_iter()
        .filter_map(|entry| {
            let ft = entry.file_type().ok()?;
            if ft.is_symlink() {
                return None; // avoid cycles and double-counting
            }
            if ft.is_dir() {
                Some(scan(&entry.path(), prog))
            } else {
                let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
                prog.files.fetch_add(1, Ordering::Relaxed);
                prog.bytes.fetch_add(size, Ordering::Relaxed);
                Some(Node {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    size,
                    is_dir: false,
                    children: vec![],
                })
            }
        })
        .collect();
    children.sort_by(|a, b| b.size.cmp(&a.size));
    let size = children.iter().map(|c| c.size).sum();
    Node { name, size, is_dir: true, children }
}
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cargo test`
Expected: `3 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: parallel drive scan engine with size aggregation"
```

---

### Task 3: Tree navigation and mutation ops

**Files:**
- Modify: `src-tauri/src/scan.rs` (append functions + tests)

**Interfaces:**
- Consumes: `Node` from Task 2
- Produces (all in `scan`):
  - `find_node<'a>(node: &'a Node, rel: &[String]) -> Option<&'a Node>` — empty `rel` returns `node` itself
  - `remove_node(node: &mut Node, rel: &[String]) -> Option<Node>` — removes, subtracts size from every ancestor, returns removed node
  - `rename_node(node: &mut Node, rel: &[String], new_name: &str) -> bool`
  - `insert_node(node: &mut Node, dest_rel: &[String], new: Node) -> bool` — caller must verify dest exists via `find_node` first; adds size along path, re-sorts destination children

- [ ] **Step 1: Write failing tests**

Append to the `tests` module in `scan.rs`:

```rust
    fn scanned() -> (tempfile::TempDir, Node) {
        let dir = fixture();
        let prog = Progress::default();
        let root = scan(dir.path(), &prog);
        (dir, root)
    }

    fn rel(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn find_node_walks_path() {
        let (_d, root) = scanned();
        assert_eq!(find_node(&root, &[]).unwrap().size, 1502);
        assert_eq!(find_node(&root, &rel(&["sub", "mid.dat"])).unwrap().size, 500);
        assert!(find_node(&root, &rel(&["nope"])).is_none());
    }

    #[test]
    fn remove_node_updates_ancestor_sizes() {
        let (_d, mut root) = scanned();
        let removed = remove_node(&mut root, &rel(&["sub", "mid.dat"])).unwrap();
        assert_eq!(removed.size, 500);
        assert_eq!(root.size, 1002);
        assert_eq!(find_node(&root, &rel(&["sub"])).unwrap().size, 0);
    }

    #[test]
    fn rename_node_keeps_size() {
        let (_d, mut root) = scanned();
        assert!(rename_node(&mut root, &rel(&["big.bin"]), "huge.bin"));
        assert!(find_node(&root, &rel(&["huge.bin"])).is_some());
        assert_eq!(root.size, 1502);
    }

    #[test]
    fn move_via_remove_then_insert() {
        let (_d, mut root) = scanned();
        let node = remove_node(&mut root, &rel(&["big.bin"])).unwrap();
        assert!(insert_node(&mut root, &rel(&["sub"]), node));
        assert_eq!(root.size, 1502);
        let sub = find_node(&root, &rel(&["sub"])).unwrap();
        assert_eq!(sub.size, 1500);
        assert_eq!(sub.children[0].name, "big.bin"); // re-sorted, biggest first
    }
```

- [ ] **Step 2: Run tests, verify the new 4 fail to compile**

Run: `cargo test`
Expected: compile error — `find_node` etc. not found.

- [ ] **Step 3: Implement the four functions**

Append above the `tests` module:

```rust
pub fn find_node<'a>(node: &'a Node, rel: &[String]) -> Option<&'a Node> {
    match rel.split_first() {
        None => Some(node),
        Some((head, tail)) => node
            .children
            .iter()
            .find(|c| c.name == *head)
            .and_then(|c| find_node(c, tail)),
    }
}

pub fn remove_node(node: &mut Node, rel: &[String]) -> Option<Node> {
    let (head, tail) = rel.split_first()?;
    if tail.is_empty() {
        let idx = node.children.iter().position(|c| c.name == *head)?;
        let removed = node.children.remove(idx);
        node.size -= removed.size;
        Some(removed)
    } else {
        let child = node.children.iter_mut().find(|c| c.name == *head)?;
        let removed = remove_node(child, tail)?;
        node.size -= removed.size;
        Some(removed)
    }
}

pub fn rename_node(node: &mut Node, rel: &[String], new_name: &str) -> bool {
    let Some((head, tail)) = rel.split_first() else { return false };
    let Some(child) = node.children.iter_mut().find(|c| c.name == *head) else { return false };
    if tail.is_empty() {
        child.name = new_name.to_string();
        true
    } else {
        rename_node(child, tail, new_name)
    }
}

pub fn insert_node(node: &mut Node, dest_rel: &[String], new: Node) -> bool {
    node.size += new.size;
    match dest_rel.split_first() {
        None => {
            node.children.push(new);
            node.children.sort_by(|a, b| b.size.cmp(&a.size));
            true
        }
        Some((head, tail)) => {
            match node.children.iter_mut().find(|c| c.name == *head) {
                Some(child) => insert_node(child, tail, new),
                None => false, // caller guarantees dest exists via find_node
            }
        }
    }
}
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cargo test`
Expected: `7 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: tree find/remove/rename/insert with ancestor size updates"
```

---

### Task 4: App state and Tauri commands

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: everything `scan` exports (Tasks 2–3)
- Produces Tauri commands callable from JS:
  - `list_drives() -> Vec<{letter, total, free}>` (fixed drives only)
  - `start_scan(drive: String)` — emits `scan-progress` `{files, bytes}` every 300ms and `scan-done` `{files, bytes, denied}`
  - `get_children(path: String) -> Vec<{name, size, is_dir}>` (size desc)
  - `open_file(path)`, `reveal(path)`, `delete(path)`, `rename(path, new_name)`, `move_item(path, dest_dir)` — all `Result<(), String>`

- [ ] **Step 1: Create `src-tauri/src/state.rs`**

```rust
use crate::scan::Node;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct ScanResult {
    pub root_path: PathBuf,
    pub root: Node,
}

#[derive(Default)]
pub struct AppState {
    pub tree: Mutex<Option<ScanResult>>,
}
```

- [ ] **Step 2: Create `src-tauri/src/commands.rs`**

```rust
use crate::scan::{self, Progress};
use crate::state::{AppState, ScanResult};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives,
};

const DRIVE_FIXED: u32 = 3;

#[derive(Serialize)]
pub struct DriveInfo {
    pub letter: String,
    pub total: u64,
    pub free: u64,
}

#[tauri::command]
pub fn list_drives() -> Vec<DriveInfo> {
    let mut out = Vec::new();
    unsafe {
        let mask = GetLogicalDrives();
        for i in 0..26u32 {
            if mask & (1 << i) == 0 {
                continue;
            }
            let letter = (b'A' + i as u8) as char;
            let root: Vec<u16> = format!("{}:\\", letter).encode_utf16().chain([0]).collect();
            if GetDriveTypeW(root.as_ptr()) != DRIVE_FIXED {
                continue;
            }
            let mut total: u64 = 0;
            let mut free: u64 = 0;
            GetDiskFreeSpaceExW(root.as_ptr(), std::ptr::null_mut(), &mut total, &mut free);
            out.push(DriveInfo { letter: format!("{}:", letter), total, free });
        }
    }
    out
}

#[derive(Serialize, Clone)]
struct ProgressPayload {
    files: u64,
    bytes: u64,
}

#[tauri::command]
pub fn start_scan(app: AppHandle, drive: String) {
    std::thread::spawn(move || {
        let prog = Arc::new(Progress::default());
        let done = Arc::new(AtomicBool::new(false));
        {
            let (prog, done, app) = (prog.clone(), done.clone(), app.clone());
            std::thread::spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    let _ = app.emit(
                        "scan-progress",
                        ProgressPayload {
                            files: prog.files.load(Ordering::Relaxed),
                            bytes: prog.bytes.load(Ordering::Relaxed),
                        },
                    );
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
            });
        }
        let root_path = PathBuf::from(&drive);
        let root = scan::scan(&root_path, &prog);
        done.store(true, Ordering::Relaxed);
        let payload = serde_json::json!({
            "files": prog.files.load(Ordering::Relaxed),
            "bytes": prog.bytes.load(Ordering::Relaxed),
            "denied": prog.denied.load(Ordering::Relaxed),
        });
        *app.state::<AppState>().tree.lock().unwrap() = Some(ScanResult { root_path, root });
        let _ = app.emit("scan-done", payload);
    });
}

fn rel_components(root: &Path, path: &str) -> Result<Vec<String>, String> {
    Path::new(path)
        .strip_prefix(root)
        .map_err(|_| "path outside scanned drive".to_string())
        .map(|p| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
}

#[derive(Serialize)]
pub struct ChildInfo {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

#[tauri::command]
pub fn get_children(state: State<AppState>, path: String) -> Result<Vec<ChildInfo>, String> {
    let guard = state.tree.lock().unwrap();
    let sr = guard.as_ref().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    let node = scan::find_node(&sr.root, &rel).ok_or("path not found")?;
    Ok(node
        .children
        .iter()
        .map(|c| ChildInfo { name: c.name.clone(), size: c.size, is_dir: c.is_dir })
        .collect())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal(path: String) -> Result<(), String> {
    // explorer exits nonzero even on success; only spawn errors matter
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete(state: State<AppState>, path: String) -> Result<(), String> {
    trash::delete(&path).map_err(|e| e.to_string())?;
    let mut guard = state.tree.lock().unwrap();
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    scan::remove_node(&mut sr.root, &rel);
    Ok(())
}

#[tauri::command]
pub fn rename(state: State<AppState>, path: String, new_name: String) -> Result<(), String> {
    if new_name.is_empty() || new_name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("invalid file name".into());
    }
    let p = Path::new(&path);
    let dest = p.with_file_name(&new_name);
    if dest.exists() {
        return Err("a file with that name already exists".into());
    }
    std::fs::rename(p, &dest).map_err(|e| e.to_string())?;
    let mut guard = state.tree.lock().unwrap();
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    scan::rename_node(&mut sr.root, &rel, &new_name);
    Ok(())
}

#[tauri::command]
pub fn move_item(state: State<AppState>, path: String, dest_dir: String) -> Result<(), String> {
    let p = Path::new(&path);
    let name = p.file_name().ok_or("bad path")?;
    let target = Path::new(&dest_dir).join(name);
    if target.exists() {
        return Err("target already exists".into());
    }
    std::fs::rename(p, &target).map_err(|e| e.to_string())?;
    let mut guard = state.tree.lock().unwrap();
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    let dest_rel = rel_components(&sr.root_path, &dest_dir)?;
    if let Some(node) = scan::remove_node(&mut sr.root, &rel) {
        if scan::find_node(&sr.root, &dest_rel).is_some() {
            scan::insert_node(&mut sr.root, &dest_rel, node);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_drives_finds_c() {
        let drives = list_drives();
        assert!(drives.iter().any(|d| d.letter == "C:"));
        assert!(drives.iter().all(|d| d.total > 0));
    }
}
```

- [ ] **Step 3: Wire into `src-tauri/src/main.rs`**

Replace the whole file:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod scan;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_drives,
            commands::start_scan,
            commands::get_children,
            commands::open_file,
            commands::reveal,
            commands::delete,
            commands::rename,
            commands::move_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Test + build**

Run: `cargo test`
Expected: `8 passed` (7 scan + 1 drives).
Run: `cargo build`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: Tauri commands for drives, scan, listing, and file ops"
```

---

### Task 5: Frontend UI

**Files:**
- Modify: `src/index.html` (replace placeholder)
- Create: `src/style.css`
- Create: `src/main.js`

**Interfaces:**
- Consumes: all Task 4 commands via `window.__TAURI__.core.invoke` and events via `window.__TAURI__.event.listen`; dialogs via `window.__TAURI__.dialog` (`open`, `confirm`, `message`)
- Produces: complete UI — drive picker (auto-scan if one drive), progress screen, lazy tree, context menu, inline rename

- [ ] **Step 1: Replace `src/index.html`**

```html
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>SpaceScanner</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <section id="drives" hidden></section>

  <section id="progress" hidden>
    <h2>Scanning…</h2>
    <div class="spinner"></div>
    <p id="prog-text">0 files · 0 B</p>
  </section>

  <section id="main" hidden>
    <header>
      <strong id="root-label"></strong>
      <span id="scan-summary"></span>
      <button id="rescan">Rescan</button>
    </header>
    <div id="tree"></div>
  </section>

  <div id="menu" class="menu" hidden>
    <div data-act="open">Open</div>
    <div data-act="reveal">Reveal in Explorer</div>
    <div data-act="rename">Rename</div>
    <div data-act="move">Move to…</div>
    <div data-act="delete" class="danger">Delete (Recycle Bin)</div>
  </div>

  <script src="main.js"></script>
</body>
</html>
```

- [ ] **Step 2: Create `src/style.css`**

```css
* { box-sizing: border-box; margin: 0; }
body { font: 13px/1.5 "Segoe UI", sans-serif; background: #1e1e24; color: #e6e6ea; height: 100vh; overflow: hidden; }
section { padding: 16px; height: 100vh; display: flex; flex-direction: column; }
#drives, #progress { align-items: center; justify-content: center; gap: 12px; }
.drive-btn { font-size: 16px; padding: 14px 28px; border-radius: 8px; border: 1px solid #444; background: #2a2a33; color: inherit; cursor: pointer; }
.drive-btn:hover { background: #34343f; }
.spinner { width: 32px; height: 32px; border: 3px solid #444; border-top-color: #4da3ff; border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
#main header { display: flex; align-items: center; gap: 12px; padding-bottom: 10px; }
#scan-summary { color: #9a9aa5; flex: 1; }
#rescan { padding: 4px 12px; border-radius: 6px; border: 1px solid #444; background: #2a2a33; color: inherit; cursor: pointer; }
#tree { flex: 1; overflow-y: auto; }
.row { display: flex; align-items: center; gap: 8px; padding: 3px 8px; border-radius: 4px; cursor: default; }
.row.dir { cursor: pointer; }
.row:hover { background: #2a2a33; }
.arrow { width: 14px; color: #9a9aa5; flex-shrink: 0; }
.name { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.name input { width: 100%; font: inherit; background: #14141a; color: inherit; border: 1px solid #4da3ff; border-radius: 3px; padding: 0 4px; }
.pct { width: 52px; text-align: right; color: #9a9aa5; flex-shrink: 0; }
.bar { width: 120px; height: 8px; background: #2a2a33; border-radius: 4px; overflow: hidden; flex-shrink: 0; }
.bar span { display: block; height: 100%; background: #4da3ff; }
.size { width: 80px; text-align: right; font-variant-numeric: tabular-nums; flex-shrink: 0; }
.menu { position: fixed; background: #2a2a33; border: 1px solid #444; border-radius: 6px; padding: 4px 0; min-width: 180px; z-index: 10; box-shadow: 0 4px 16px rgba(0,0,0,0.5); }
.menu div { padding: 6px 14px; cursor: pointer; }
.menu div:hover { background: #34343f; }
.menu .danger { color: #ff6b6b; }
```

- [ ] **Step 3: Create `src/main.js`**

```js
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open: openDialog, confirm: confirmDialog, message } = window.__TAURI__.dialog;

let rootPath = "";
const expanded = new Set();
const $ = (id) => document.getElementById(id);
const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

function fmt(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0, v = bytes;
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return v.toFixed(i ? 1 : 0) + " " + units[i];
}

function show(id) {
  for (const s of ["drives", "progress", "main"]) $(s).hidden = s !== id;
}

async function init() {
  const drives = await invoke("list_drives");
  if (drives.length === 1) return startScan(drives[0].letter + "\\");
  const div = $("drives");
  div.innerHTML = "<h2>Select a drive to scan</h2>";
  for (const d of drives) {
    const btn = document.createElement("button");
    btn.className = "drive-btn";
    btn.textContent = `${d.letter}  —  ${fmt(d.total - d.free)} used of ${fmt(d.total)}`;
    btn.onclick = () => startScan(d.letter + "\\");
    div.appendChild(btn);
  }
  show("drives");
}

async function startScan(drive) {
  rootPath = drive;
  expanded.clear();
  show("progress");
  await invoke("start_scan", { drive });
}

listen("scan-progress", (e) => {
  $("prog-text").textContent = `${e.payload.files.toLocaleString()} files · ${fmt(e.payload.bytes)}`;
});

listen("scan-done", async (e) => {
  $("root-label").textContent = rootPath;
  $("scan-summary").textContent =
    `${e.payload.files.toLocaleString()} files · ${fmt(e.payload.bytes)}` +
    (e.payload.denied ? ` · ${e.payload.denied} folders unscanned` : "");
  show("main");
  await renderTree();
});

$("rescan").onclick = () => startScan(rootPath);

const joinPath = (parent, name) => (parent.endsWith("\\") ? parent + name : parent + "\\" + name);

async function renderTree() {
  const tree = $("tree");
  tree.innerHTML = "";
  await renderLevel(tree, rootPath, 0);
}

async function renderLevel(container, parentPath, depth) {
  let children;
  try {
    children = await invoke("get_children", { path: parentPath });
  } catch {
    return;
  }
  const total = children.reduce((s, c) => s + c.size, 0) || 1;
  for (const c of children) {
    const path = joinPath(parentPath, c.name);
    const pct = (c.size / total) * 100;
    const row = document.createElement("div");
    row.className = "row" + (c.is_dir ? " dir" : "");
    row.style.paddingLeft = depth * 20 + 8 + "px";
    row.innerHTML =
      `<span class="arrow">${c.is_dir ? (expanded.has(path) ? "▾" : "▸") : ""}</span>` +
      `<span class="name" title="${esc(path)}">${esc(c.name)}</span>` +
      `<span class="pct">${pct.toFixed(1)}%</span>` +
      `<span class="bar"><span style="width:${pct}%"></span></span>` +
      `<span class="size">${fmt(c.size)}</span>`;
    const kids = document.createElement("div");
    row.onclick = async () => {
      if (!c.is_dir) return;
      if (expanded.has(path)) {
        expanded.delete(path);
        kids.innerHTML = "";
        row.querySelector(".arrow").textContent = "▸";
      } else {
        expanded.add(path);
        row.querySelector(".arrow").textContent = "▾";
        await renderLevel(kids, path, depth + 1);
      }
    };
    row.oncontextmenu = (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      showMenu(ev, path, row);
    };
    container.appendChild(row);
    container.appendChild(kids);
    if (c.is_dir && expanded.has(path)) await renderLevel(kids, path, depth + 1);
  }
}

function showMenu(ev, path, row) {
  const menu = $("menu");
  menu.style.left = Math.min(ev.pageX, window.innerWidth - 200) + "px";
  menu.style.top = Math.min(ev.pageY, window.innerHeight - 180) + "px";
  menu.hidden = false;
  menu.onclick = async (e2) => {
    menu.hidden = true;
    const act = e2.target.dataset.act;
    if (!act) return;
    try {
      if (act === "open") {
        await invoke("open_file", { path });
      } else if (act === "reveal") {
        await invoke("reveal", { path });
      } else if (act === "delete") {
        const ok = await confirmDialog(`Move to Recycle Bin?\n\n${path}`, { title: "Delete", kind: "warning" });
        if (ok) {
          await invoke("delete", { path });
          await renderTree();
        }
      } else if (act === "rename") {
        startRename(row, path);
      } else if (act === "move") {
        const dest = await openDialog({ directory: true, title: "Move to folder" });
        if (dest) {
          await invoke("move_item", { path, destDir: dest });
          await renderTree();
        }
      }
    } catch (err) {
      await message(String(err), { title: "Error", kind: "error" });
    }
  };
}

document.addEventListener("click", () => { $("menu").hidden = true; });

function startRename(row, path) {
  const span = row.querySelector(".name");
  const input = document.createElement("input");
  input.value = path.split("\\").pop();
  span.textContent = "";
  span.appendChild(input);
  input.focus();
  input.select();
  input.onclick = (e) => e.stopPropagation();
  input.onkeydown = async (e) => {
    if (e.key === "Enter") {
      try {
        await invoke("rename", { path, newName: input.value });
        await renderTree();
      } catch (err) {
        await message(String(err), { title: "Error", kind: "error" });
        await renderTree();
      }
    } else if (e.key === "Escape") {
      renderTree();
    }
  };
}

init();
```

- [ ] **Step 4: Run the app, manual check**

Run: `cargo run`
Expected: UAC prompt → window opens → single drive auto-scans (or drive buttons) → progress counts up → tree renders biggest-first.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: tree UI with drive picker, progress, context menu, inline rename"
```

---

### Task 6: End-to-end smoke test

**Files:** none created — verification only. Fix regressions inline if found.

**Interfaces:**
- Consumes: the whole app
- Produces: verified working build

- [ ] **Step 1: Run smoke checklist against `cargo run`**

On a scratch folder you create first (`mkdir C:\scanner-smoke && echo test > C:\scanner-smoke\a.txt`), after scan completes:

1. UAC elevation prompt appeared at launch — PASS/FAIL
2. Tree sorted biggest→smallest at every level — PASS/FAIL
3. Expand/collapse folders works, children sorted — PASS/FAIL
4. Right-click → Open launches default app — PASS/FAIL
5. Reveal in Explorer selects the file — PASS/FAIL
6. Delete moves `a.txt` to Recycle Bin (verify in Recycle Bin), row disappears, ancestor sizes shrink — PASS/FAIL
7. Rename inline works, tree updates — PASS/FAIL
8. Move to another folder works, both branches update — PASS/FAIL
9. Rescan re-scans same drive — PASS/FAIL

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: `8 passed`.

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: smoke test findings"
```

(Skip commit if nothing changed.)

- [ ] **Step 4: Release build (deliverable exe)**

Run: `cargo build --release`
Expected: `src-tauri/target/release/space-scanner.exe` exists. Report path to user.
