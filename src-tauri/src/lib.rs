use std::sync::Mutex;

use tauri::Manager;

mod commands;
mod git;
mod ide;
mod models;
mod store;

use commands::AppState;
use store::Store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            // re-enable native window decorations so OS app bar is visible
            window.set_decorations(true)?;

            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from(".codefinder"));

            app.manage(Mutex::new(AppState { store: Store::load(data_dir) }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // folders
            commands::list_folders,
            commands::add_folder,
            commands::remove_folder,
            // repos
            commands::list_repos,
            commands::list_all_repos,
            commands::add_repo,
            commands::remove_repo,
            commands::get_repo_detail,
            commands::get_commits,
            commands::list_recent_activity,
            commands::checkout_branch,
            commands::get_status,
            commands::toggle_favorite,
            commands::update_description,
            // editors
            commands::detect_ides,
            commands::add_custom_editor,
            commands::remove_custom_editor,
            commands::open_in,
            // git actions
            commands::fetch,
            commands::pull,
            // scan
            commands::scan_dir,
            // shell helpers
            commands::open_terminal,
            commands::open_folder,
            // dialog
            commands::pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
