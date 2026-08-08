use crate::scan::{self, Progress};
use crate::state::{AppState, ScanResult};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const DRIVE_FIXED: u32 = 3;

/// Whether this process holds an elevated (Administrator) token.
///
/// The app launches as `asInvoker` and asks for elevation itself, so declining the UAC
/// prompt leaves us running as a standard user. The UI calls this to tell the user that
/// protected files and folders will be skipped rather than silently under-reporting.
#[tauri::command]
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut info = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut info as *mut _ as *mut core::ffi::c_void,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        );
        CloseHandle(token);
        ok != 0 && info.TokenIsElevated != 0
    }
}

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

// The destructive actions keep their guards in plain functions over &mut ScanResult
// rather than inline in the command bodies: State<AppState> cannot be built outside a
// running Tauri app, and these guards — path-escape, silent overwrite, invalid names —
// are exactly the parts worth unit-testing.

#[tauri::command]
pub fn delete(state: State<AppState>, path: String) -> Result<(), String> {
    delete_in(lock_tree(&state).as_mut().ok_or("no scan loaded")?, &path)
}

fn delete_in(sr: &mut ScanResult, path: &str) -> Result<(), String> {
    let rel = rel_components(&sr.root_path, path)?;
    trash::delete(path).map_err(|e| e.to_string())?;
    if scan::remove_node(&mut sr.root, &rel).is_none() {
        return Err("deleted from disk but not found in scan tree — rescan recommended".into());
    }
    Ok(())
}

#[tauri::command]
pub fn rename(state: State<AppState>, path: String, new_name: String) -> Result<(), String> {
    rename_in(lock_tree(&state).as_mut().ok_or("no scan loaded")?, &path, &new_name)
}

fn rename_in(sr: &mut ScanResult, path: &str, new_name: &str) -> Result<(), String> {
    if new_name.is_empty() || new_name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("invalid file name".into());
    }
    let rel = rel_components(&sr.root_path, path)?;
    let p = Path::new(path);
    let dest = p.with_file_name(new_name);
    if dest.exists() {
        return Err("a file with that name already exists".into());
    }
    std::fs::rename(p, &dest).map_err(|e| e.to_string())?;
    scan::rename_node(&mut sr.root, &rel, new_name);
    Ok(())
}

#[tauri::command]
pub fn move_item(state: State<AppState>, path: String, dest_dir: String) -> Result<(), String> {
    move_in(lock_tree(&state).as_mut().ok_or("no scan loaded")?, &path, &dest_dir)
}

fn move_in(sr: &mut ScanResult, path: &str, dest_dir: &str) -> Result<(), String> {
    let p = Path::new(path);
    let name = p.file_name().ok_or("bad path")?;
    let target = Path::new(dest_dir).join(name);
    let rel = rel_components(&sr.root_path, path)?;
    let dest_rel = rel_components(&sr.root_path, dest_dir)?;
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
    use std::fs;

    #[test]
    fn list_drives_finds_c() {
        let drives = list_drives();
        assert!(drives.iter().any(|d| d.letter == "C:"));
        assert!(drives.iter().all(|d| d.total > 0));
    }

    /// A scanned fixture: a.txt (3 bytes), b.txt (2 bytes), and an empty sub/.
    fn scanned() -> (tempfile::TempDir, ScanResult) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"aaa").unwrap();
        fs::write(dir.path().join("b.txt"), b"bb").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let root = scan::scan(dir.path(), &scan::Progress::default());
        let root_path = dir.path().to_path_buf();
        (dir, ScanResult { root_path, root })
    }

    fn at(dir: &tempfile::TempDir, name: &str) -> String {
        dir.path().join(name).to_string_lossy().into_owned()
    }

    #[test]
    fn rename_rejects_names_that_would_change_the_target_directory() {
        let (d, mut sr) = scanned();
        for bad in ["", "..\\escaped.txt", "sub/nested.txt", "c:evil", "why?.txt", "a<b", "a|b"] {
            let err = rename_in(&mut sr, &at(&d, "a.txt"), bad).unwrap_err();
            assert_eq!(err, "invalid file name", "accepted {bad:?}");
        }
        assert!(d.path().join("a.txt").exists(), "original must survive a rejected rename");
    }

    #[test]
    fn rename_refuses_to_clobber_an_existing_file() {
        let (d, mut sr) = scanned();
        let err = rename_in(&mut sr, &at(&d, "a.txt"), "b.txt").unwrap_err();
        assert_eq!(err, "a file with that name already exists");
        // both files intact, and b.txt still holds its own bytes rather than a's
        assert_eq!(fs::read(d.path().join("a.txt")).unwrap(), b"aaa");
        assert_eq!(fs::read(d.path().join("b.txt")).unwrap(), b"bb");
    }

    #[test]
    fn rename_updates_disk_and_tree_together() {
        let (d, mut sr) = scanned();
        rename_in(&mut sr, &at(&d, "a.txt"), "renamed.txt").unwrap();
        assert!(d.path().join("renamed.txt").exists());
        assert!(!d.path().join("a.txt").exists());
        assert!(scan::find_node(&sr.root, &["renamed.txt".to_string()]).is_some());
        assert!(scan::find_node(&sr.root, &["a.txt".to_string()]).is_none());
    }

    #[test]
    fn move_refuses_to_clobber_an_existing_target() {
        let (d, mut sr) = scanned();
        fs::write(d.path().join("sub").join("a.txt"), b"different").unwrap();
        let err = move_in(&mut sr, &at(&d, "a.txt"), &at(&d, "sub")).unwrap_err();
        assert_eq!(err, "target already exists");
        assert_eq!(fs::read(d.path().join("a.txt")).unwrap(), b"aaa");
        assert_eq!(fs::read(d.path().join("sub").join("a.txt")).unwrap(), b"different");
    }

    #[test]
    fn move_updates_disk_and_tree_together() {
        let (d, mut sr) = scanned();
        move_in(&mut sr, &at(&d, "a.txt"), &at(&d, "sub")).unwrap();
        assert!(d.path().join("sub").join("a.txt").exists());
        assert!(!d.path().join("a.txt").exists());
        let sub = scan::find_node(&sr.root, &["sub".to_string()]).unwrap();
        assert_eq!(sub.size, 3, "moved bytes must land on the destination folder");
        assert_eq!(sr.root.size, 5, "total is unchanged by a move");
    }

    // The scan tree is the authority for what the app may touch. Anything that does not
    // resolve under the scanned root is refused before it reaches the filesystem.
    #[test]
    fn actions_refuse_paths_outside_the_scanned_root() {
        let (d, mut sr) = scanned();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("victim.txt"), b"do not touch").unwrap();
        let target = outside.path().join("victim.txt").to_string_lossy().into_owned();

        assert_eq!(delete_in(&mut sr, &target).unwrap_err(), "path outside scanned drive");
        assert_eq!(
            rename_in(&mut sr, &target, "renamed.txt").unwrap_err(),
            "path outside scanned drive"
        );
        assert_eq!(
            move_in(&mut sr, &target, &at(&d, "sub")).unwrap_err(),
            "path outside scanned drive"
        );

        // untouched on disk — in particular, delete_in bailed before trash::delete ran
        assert_eq!(fs::read(outside.path().join("victim.txt")).unwrap(), b"do not touch");
    }

    #[test]
    fn move_refuses_a_destination_that_is_not_in_the_tree() {
        let (d, mut sr) = scanned();
        let ghost = at(&d, "never-scanned");
        fs::create_dir(&ghost).unwrap(); // exists on disk, absent from the scan
        let err = move_in(&mut sr, &at(&d, "a.txt"), &ghost).unwrap_err();
        assert_eq!(err, "destination not in scanned tree");
        assert!(d.path().join("a.txt").exists());
    }
}
