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
