mod commands;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::import_audio,
            commands::start_translation,
            commands::export_subtitle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
