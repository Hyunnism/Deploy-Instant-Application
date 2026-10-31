use tauri::Manager;

pub mod commands;
pub mod core;
pub mod error;
pub mod models;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            let app_files_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(std::io::Error::other(format!("Gagal app data dir: {e}"))))?;
            let db_path = app_files_dir.join("db").join("deploy_instant.sqlite");
            let db = storage::db::Database::open(&db_path).map_err(|e| {
                Box::new(std::io::Error::other(format!("Gagal init SQLite db: {e}")))
            })?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::system_ping,
            commands::system::system_run_probe,
            commands::setup::setup_check_status,
            commands::setup::setup_start,
            commands::setup::setup_verify,
            commands::project::project_create,
            commands::project::project_list,
            commands::project::project_get,
            commands::project::project_update,
            commands::project::project_delete,
            commands::project::setting_get,
            commands::project::setting_set,
            commands::project::setting_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
