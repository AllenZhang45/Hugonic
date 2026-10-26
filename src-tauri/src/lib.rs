mod commands;
mod state;

use state::{AppState, PreviewState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState::default())
        .manage(PreviewState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::project::open_project,
            commands::files::read_dir,
            commands::files::read_file,
            commands::files::save_file,
            commands::files::file_meta,
            commands::preview::start_preview,
            commands::preview::stop_preview,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            commands::preview::stop_preview_impl(handle);
        }
    });
}
