#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod update;

use app_state::AppState;
use commands::*;

fn main() {
    // Initialize logging directory
    let _ = std::fs::create_dir_all(
        phpvm_core::config::get_log_path()
            .parent()
            .unwrap(),
    );

    let app_state = match AppState::new() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize PHP manager: {}", e);
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            install_version,
            remove_version,
            switch_version,
            list_installed,
            list_available,
            get_active,
            get_install_path,
            get_log_path,
            check_path_status,
            set_path,
            get_version_status,
            get_current_dir,
            list_cached_files,
            remove_cached_file,
            clear_all_cache,
            get_app_version,
            check_for_updates,
            download_update,
            apply_update
        ])
        .setup(|_app| {
            // App initialization code can go here
            // Update check is triggered from frontend after app loads
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
