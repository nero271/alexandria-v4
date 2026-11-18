use super::AppState;
use crate::backup::{self, BackupSummary};
use crate::error::AppResult;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn export_backup(state: State<'_, AppState>, path: String) -> AppResult<usize> {
    state.with_db(|c| backup::export(c, &PathBuf::from(path)))
}

#[tauri::command]
pub fn import_backup(state: State<'_, AppState>, path: String) -> AppResult<BackupSummary> {
    state.with_db(|c| backup::import(c, &PathBuf::from(path)))
}
