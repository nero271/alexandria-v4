use super::{join_err, AppState};
use crate::error::{AppError, AppResult};
use crate::library::model::SyncSummary;
use crate::library::repository as repo;
use crate::platforms::{epic, gog, ubisoft, xbox};
use crate::steam;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub steam: bool,
    pub epic: bool,
    pub gog: bool,
    pub ubisoft: bool,
    pub xbox: bool,
}

#[tauri::command]
pub async fn platform_status() -> AppResult<PlatformStatus> {
    tauri::async_runtime::spawn_blocking(|| PlatformStatus {
        steam: steam::find_steam_dir().is_some(),
        epic: epic::scan().is_some(),
        gog: gog::scan().map(|v| !v.is_empty()).unwrap_or(false),
        ubisoft: ubisoft::scan().map(|v| !v.is_empty()).unwrap_or(false),
        xbox: xbox::scan().map(|v| !v.is_empty()).unwrap_or(false),
    })
    .await
    .map_err(join_err)
}

/// Sincroniza uma plataforma ("steam", "epic" ou "gog") sem sobrescrever personalizações.
#[tauri::command]
pub async fn platform_sync(state: State<'_, AppState>, platform: String) -> AppResult<SyncSummary> {
    match platform.as_str() {
        "steam" => super::steam::steam_sync(state).await,
        "epic" => {
            let apps = tauri::async_runtime::spawn_blocking(epic::scan)
                .await
                .map_err(join_err)?
                .ok_or_else(|| AppError::Invalid("A Epic Games Launcher não foi encontrada.".into()))?;
            state.with_db(|c| repo::sync_platform(c, "epic", &apps))
        }
        "gog" => {
            let apps = tauri::async_runtime::spawn_blocking(gog::scan)
                .await
                .map_err(join_err)?
                .ok_or_else(|| AppError::Invalid("O GOG Galaxy não foi encontrado.".into()))?;
            state.with_db(|c| repo::sync_platform(c, "gog", &apps))
        }
        "ubisoft" => {
            let apps = tauri::async_runtime::spawn_blocking(ubisoft::scan)
                .await
                .map_err(join_err)?
                .ok_or_else(|| AppError::Invalid("O Ubisoft Connect não foi encontrado.".into()))?;
            state.with_db(|c| repo::sync_platform(c, "ubisoft", &apps))
        }
        "xbox" => {
            let apps = tauri::async_runtime::spawn_blocking(xbox::scan)
                .await
                .map_err(join_err)?
                .ok_or_else(|| AppError::Invalid("Nenhuma pasta XboxGames foi encontrada.".into()))?;
            state.with_db(|c| repo::sync_platform(c, "xbox", &apps))
        }
        _ => Err(AppError::Invalid("Plataforma desconhecida.".into())),
    }
}
