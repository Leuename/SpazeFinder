#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod scan;
mod state;

use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const INSTANCE: &str = "club.formura.spazefinder";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Claims the single-instance slot, or `None` if another instance already holds it.
///
/// ponytail: a named mutex over tauri-plugin-single-instance — windows-sys is already a
/// dependency and exiting is enough. Session-local, so elevated and standard instances of
/// the same user still see each other.
fn claim_instance(name: &str) -> Option<HANDLE> {
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 1, wide(name).as_ptr());
        if handle.is_null() {
            return None;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            return None;
        }
        Some(handle)
    }
}

/// Asks Windows to relaunch this executable elevated.
///
/// `true` means a new elevated process started and this one should exit. `false` means
/// the user declined the UAC prompt (or the launch failed), and we carry on with whatever
/// access a standard user has — the UI then tells them protected folders are skipped.
fn request_elevation() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let file: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let started = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            wide("runas").as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW returns >32 on success; a declined prompt comes back as ERROR_CANCELLED
    started as isize > 32
}

fn main() {
    // Claim before prompting, so a second launch exits quietly instead of raising a UAC
    // dialog only to discover an instance is already running.
    let Some(instance) = claim_instance(INSTANCE) else {
        return;
    };

    if !commands::is_elevated() {
        // Release first: the elevated copy starts while this process is still alive, and
        // would otherwise find the slot taken and exit — leaving nothing running at all.
        unsafe { CloseHandle(instance) };
        if request_elevation() {
            return; // the elevated copy takes over
        }
        // Declined. Carry on unelevated and take the slot back.
        if claim_instance(INSTANCE).is_none() {
            return;
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::is_elevated,
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
    use super::{claim_instance, CloseHandle};

    #[test]
    fn the_slot_admits_one_holder_and_frees_up_on_release() {
        // unique per run so a real SpazeFinder instance can be open during `cargo test`
        let name = format!("spazefinder-test-{}", std::process::id());
        let first = claim_instance(&name).expect("first claim must succeed");
        assert!(claim_instance(&name).is_none(), "second claim must be refused");
        // release matters: main() drops the slot before relaunching elevated, and the
        // elevated copy has to be able to take it
        unsafe { CloseHandle(first) };
        assert!(claim_instance(&name).is_some(), "slot must free up once released");
    }
}
