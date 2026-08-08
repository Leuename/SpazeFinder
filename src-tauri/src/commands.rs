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
        *app.state::<AppState>().tree.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(ScanResult { root_path, root });
        let _ = app.emit("scan-done", payload);
    });
}

fn lock_tree<'a>(state: &'a State<'a, AppState>) -> std::sync::MutexGuard<'a, Option<ScanResult>> {
    state.tree.lock().unwrap_or_else(|e| e.into_inner())
}

fn rel_components(root: &Path, path: &str) -> Result<Vec<String>, String> {
    Path::new(path)
        .strip_prefix(root)
        .map_err(|_| "path outside scanned drive".to_string())
        .map(|p| p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect())
}

#[derive(Serialize)]
pub struct ChildInfo {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

#[tauri::command]
pub fn get_children(state: State<AppState>, path: String) -> Result<Vec<ChildInfo>, String> {
    let guard = lock_tree(&state);
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
    // explorer.exe brokers the launch so the child runs unelevated
    std::process::Command::new("explorer").arg(&path).spawn().map(|_| ()).map_err(|e| e.to_string())
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
    let mut guard = lock_tree(&state);
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    trash::delete(&path).map_err(|e| e.to_string())?;
    if scan::remove_node(&mut sr.root, &rel).is_none() {
        return Err("deleted from disk but not found in scan tree — rescan recommended".into());
    }
    Ok(())
}

#[tauri::command]
pub fn rename(state: State<AppState>, path: String, new_name: String) -> Result<(), String> {
    if new_name.is_empty() || new_name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("invalid file name".into());
    }
    let mut guard = lock_tree(&state);
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    let p = Path::new(&path);
    let dest = p.with_file_name(&new_name);
    if dest.exists() {
        return Err("a file with that name already exists".into());
    }
    std::fs::rename(p, &dest).map_err(|e| e.to_string())?;
    scan::rename_node(&mut sr.root, &rel, &new_name);
    Ok(())
}

#[tauri::command]
pub fn move_item(state: State<AppState>, path: String, dest_dir: String) -> Result<(), String> {
    let p = Path::new(&path);
    let name = p.file_name().ok_or("bad path")?;
    let target = Path::new(&dest_dir).join(name);
    let mut guard = lock_tree(&state);
    let sr = guard.as_mut().ok_or("no scan loaded")?;
    let rel = rel_components(&sr.root_path, &path)?;
    let dest_rel = rel_components(&sr.root_path, &dest_dir)?;
    if scan::find_node(&sr.root, &dest_rel).is_none() {
        return Err("destination not in scanned tree".into());
    }
    if target.exists() {
        return Err("target already exists".into());
    }
    std::fs::rename(p, &target).map_err(|e| e.to_string())?;
    if let Some(node) = scan::remove_node(&mut sr.root, &rel) {
        scan::insert_node(&mut sr.root, &dest_rel, node);
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
