// commands/project.rs
// Thin IPC layer untuk operasi Project dan Settings (Fase 3)

use crate::models::{CreateProjectInput, Project, SettingItem, UpdateProjectInput};
use crate::storage::db::Database;
use tauri::State;

#[tauri::command]
pub fn project_create(
    db: State<'_, Database>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    db.create_project(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_list(db: State<'_, Database>) -> Result<Vec<Project>, String> {
    db.list_projects().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_get(db: State<'_, Database>, id: i64) -> Result<Project, String> {
    db.get_project(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_update(
    db: State<'_, Database>,
    id: i64,
    input: UpdateProjectInput,
) -> Result<Project, String> {
    db.update_project(id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn project_delete(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_project(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_get(db: State<'_, Database>, key: String) -> Result<Option<String>, String> {
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_set(db: State<'_, Database>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn setting_list(db: State<'_, Database>) -> Result<Vec<SettingItem>, String> {
    db.list_settings().map_err(|e| e.to_string())
}
