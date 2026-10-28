pub mod commands;
pub mod core;
pub mod error;
pub mod models;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::system::system_ping,
            commands::system::system_run_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
