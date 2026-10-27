mod commands;
mod process;
mod state;

use state::{AppState, BuildState, DeployState, PreviewState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState::default())
        .manage(PreviewState::default())
        .manage(BuildState::default())
        .manage(DeployState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            commands::preview::cleanup_stale_preview(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::open_project,
            commands::files::read_dir,
            commands::files::read_file,
            commands::files::save_file,
            commands::files::file_meta,
            commands::insights::project_insights,
            commands::themes::list_themes,
            commands::themes::theme_screenshot,
            commands::themes::apply_theme,
            commands::themes::open_theme_folder,
            commands::preview::start_preview,
            commands::preview::stop_preview,
            commands::preview::preview_alive,
            commands::build::start_build,
            commands::build::cancel_build,
            commands::build::open_build_output,
            commands::deploy::netlify_credential_status,
            commands::deploy::save_netlify_token,
            commands::deploy::clear_netlify_token,
            commands::deploy::test_netlify_connection,
            commands::deploy::start_deploy,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            commands::build::cancel_build_impl(handle);
            commands::preview::stop_preview_impl(handle);
        }
    });
}
