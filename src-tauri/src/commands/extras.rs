use super::{join_err, AppState};
use crate::emulators::{self, Emulator};
use crate::error::AppResult;
use crate::stats::{self, YearStats};
use crate::{screenshots, steam};
use tauri::{AppHandle, Manager, State};

/// Resumo do ano; `tz_offset_minutes` é o fuso do usuário (ex.: -180 para Brasília).
#[tauri::command]
pub fn year_stats(state: State<'_, AppState>, year: i32, tz_offset_minutes: i32) -> AppResult<YearStats> {
    let now = crate::library::repository::now();
    state.with_db(|c| stats::year_stats(c, year, tz_offset_minutes as i64, now))
}

/// Capturas de tela da Steam de um jogo (mais novas primeiro). Libera só essas pastas para exibição.
#[tauri::command]
pub async fn list_screenshots(app: AppHandle, app_id: String) -> AppResult<Vec<String>> {
    let (files, dirs) = tauri::async_runtime::spawn_blocking(move || {
        steam::find_steam_dir()
            .map(|d| screenshots::find(&d, &app_id))
            .unwrap_or_default()
    })
    .await
    .map_err(join_err)?;
    let scope = app.asset_protocol_scope();
    for d in &dirs {
        let _ = scope.allow_directory(d, false);
    }
    Ok(files.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
}

#[tauri::command]
pub fn list_emulators(state: State<'_, AppState>) -> AppResult<Vec<Emulator>> {
    state.with_db(emulators::list)
}

#[tauri::command]
pub fn add_emulator(state: State<'_, AppState>, name: String, exe_path: String, args: String) -> AppResult<Emulator> {
    state.with_db(|c| emulators::add(c, &name, &exe_path, &args))
}

#[tauri::command]
pub fn remove_emulator(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.with_db(|c| emulators::remove(c, id))
}
