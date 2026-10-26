mod commands;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::project::open_project,
            commands::files::read_dir,
            commands::files::read_file,
            commands::files::save_file,
            commands::files::file_meta,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
