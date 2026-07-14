# SpaceScanner — Design Spec

Date: 2026-07-15
Status: Approved pending user review

## Purpose

Windows desktop app that scans a whole drive and shows what takes up space as a folder tree sorted biggest→smallest, with Explorer-like file interaction.

## Stack

- **Tauri 2** — Rust backend, plain HTML/CSS/JS frontend (no framework)
- Rust crates: `jwalk` (parallel directory walk), `trash` (Recycle Bin delete)
- Windows app manifest: `requireAdministrator` — UAC elevation prompt at launch so scan reaches protected folders

## Architecture

- **Rust backend** owns the scan tree and all file operations.
  - Scan: parallel walk of the selected drive, builds an in-memory tree of `{name, path, size, is_dir, children}` where every folder's size = recursive sum. Progress events (files counted, bytes so far) streamed to UI during scan.
  - UI never receives the whole tree. Frontend requests one directory level at a time; backend returns that level's children sorted by size descending.
- **Frontend**: single-page tree list UI. Lazy-expands folders by asking the backend per level.

### Tauri commands

| Command | Does |
|---|---|
| `list_drives` | fixed drives with total/free space |
| `start_scan(drive)` | kick off scan; emits `scan-progress` events, `scan-done` |
| `get_children(path)` | children of path from scan tree, sorted size desc |
| `open_file(path)` | shell open with default app |
| `reveal(path)` | open Explorer with file selected |
| `delete(path)` | send to Recycle Bin (`trash` crate); update tree + ancestor sizes |
| `rename(path, new_name)` | rename; update tree |
| `move_item(path, dest_dir)` | move; update tree + ancestor sizes on both branches |

## UI

- **Launch**: UAC prompt (manifest). Then: one fixed drive → scan starts immediately; multiple → drive buttons (letter, used/total), click to scan.
- **Scanning**: progress bar with file count + bytes scanned.
- **Main view**:
  - Tree list rows: name, size (human units), % of parent, horizontal bar proportional to %.
  - Folders expand/collapse on click; children fetched lazily, sorted biggest→smallest.
  - Breadcrumb path at top; Rescan button; drive switcher if multiple drives.
- **Context menu** (right-click row):
  - Open
  - Reveal in Explorer
  - Delete (Recycle Bin) — one confirm dialog
  - Rename — inline edit
  - Move — folder picker dialog
- After delete/rename/move: backend updates tree and ancestor sizes; UI refreshes affected rows in place. No full rescan.

## Errors

- Access-denied folders even under admin (locked system files): skip, tally into an "unscanned" counter shown after scan.
- File op failures (locked file, path gone): error toast with OS message; tree untouched on failure.
- Delete is Recycle-Bin only — recoverable by design. No permanent delete in app.

## Testing

- Rust unit tests: scan tree building + size aggregation + sorted-children against a temp-dir fixture; delete/rename/move tree-update math against fixture.
- File ops against real shell (open/reveal/recycle) smoke-tested manually.

## Out of scope

- Treemap view, multi-drive simultaneous scan, scheduled scans, dark/light theme toggle, non-Windows platforms.
