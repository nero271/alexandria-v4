use super::{join_err, AppState};
use crate::error::{AppError, AppResult};
use crate::library::model::SyncSummary;
use crate::library::repository as repo;
use crate::steam;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamStatus {
    pub found: bool,
    pub path: Option<String>,
}

#[tauri::command]
pub fn steam_status() -> SteamStatus {
    let dir = steam::find_steam_dir();
    SteamStatus {
        found: dir.is_some(),
        path: dir.map(|d| d.to_string_lossy().into_owned()),
    }
}

/// Lê os jogos instalados na Steam e atualiza a biblioteca (sem sobrescrever personalizações).
#[tauri::command]
pub async fn steam_sync(state: State<'_, AppState>) -> AppResult<SyncSummary> {
    let apps = tauri::async_runtime::spawn_blocking(|| {
        steam::find_steam_dir().map(|d| steam::scan_installed(&d))
    })
    .await
    .map_err(join_err)?
    .ok_or_else(|| AppError::Invalid("A Steam não foi encontrada neste computador.".into()))?;
    state.with_db(|c| repo::sync_steam(c, &apps))
}
