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
            let db = storage::db::Database::open(&db_path, &app_files_dir).map_err(|e| {
                Box::new(std::io::Error::other(format!("Gagal init SQLite db: {e}")))
            })?;
            app.manage(db);
            app.manage(core::project::ActiveBuildManager::new());
            app.manage(core::pm2::create_log_stream_manager());
            app.manage(core::tunnel::TunnelManager::new());

            // Daftarkan global AppHandle untuk JNI lifecycle callbacks
            core::lifecycle::register_app_handle(app.handle().clone());

            // Bersihkan sisa proses yatim dari sesi sebelumnya jika ada
            let app_files_dir_clone = app_files_dir.clone();
            tauri::async_runtime::spawn(async move {
                let cleaned = core::lifecycle::clean_orphan_processes(&app_files_dir_clone).await;
                if cleaned > 0 {
                    eprintln!(
                        "[DeployInstant Lifecycle] Membersihkan {} proses yatim saat startup.",
                        cleaned
                    );
                }
            });

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
            commands::project::project_repo_status,
            commands::project::project_clone,
            commands::project::project_pull,
            commands::project::project_build,
            commands::project::project_build_cancel,
            commands::project::project_build_logs_list,
            commands::project::project_build_log_read,
            commands::project::project_env_list,
            commands::project::project_env_set,
            commands::project::project_env_delete,
            commands::project::project_env_import_dotenv,
            commands::project::settings_save_pat,
            commands::project::settings_get_pat_status,
            commands::project::settings_delete_pat,
            commands::project::setting_get,
            commands::project::setting_set,
            commands::project::setting_list,
            commands::pm2::pm2_start,
            commands::pm2::pm2_stop,
            commands::pm2::pm2_restart,
            commands::pm2::pm2_delete,
            commands::pm2::pm2_list,
            commands::pm2::pm2_stop_all,
            commands::pm2::pm2_check_autostart,
            commands::pm2::runtime_logs_start,
            commands::pm2::runtime_logs_stop,
            commands::pm2::open_local_url,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::list_active_tunnels,
            commands::tunnel::share_url,
            commands::tunnel::open_external_url,
            commands::lifecycle::get_battery_status,
            commands::lifecycle::open_battery_settings,
            commands::lifecycle::clean_orphans,
            commands::lifecycle::shutdown_cleanly,
            commands::project::project_export_config,
            commands::project::project_export_all_configs,
            commands::project::project_import_config,
            commands::storage::storage_get_system_info,
            commands::storage::storage_get_project_info,
            commands::storage::storage_clean_project,
            commands::storage::storage_check_preflight,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
