#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod scan;
mod state;

use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;

/// True if another instance already holds `name` in this logon session.
///
/// ponytail: a named mutex over tauri-plugin-single-instance — windows-sys is already a
/// dependency and exiting is enough. Without this, a second launch means a second UAC
/// prompt and a second full-drive scan. Upgrade to the plugin if raising the running
/// window (rather than just exiting) ever matters.
///
/// The handle is deliberately never closed: Windows releases it at process exit, which is
/// exactly the lifetime we want. Session-local, so elevated and standard instances of the
/// same user still see each other.
fn instance_already_running(name: &str) -> bool {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 1, wide.as_ptr());
        handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS
    }
}

fn main() {
    if instance_already_running("club.formura.spazefinder") {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::instance_already_running;

    #[test]
    fn first_caller_wins_and_the_next_one_is_told_to_back_off() {
        // unique per run so a real SpazeFinder instance can be open during `cargo test`
        let name = format!("spazefinder-test-{}", std::process::id());
        assert!(!instance_already_running(&name), "first acquisition must succeed");
        assert!(instance_already_running(&name), "second acquisition must be refused");
    }
}
