use super::{join_err, AppState};
use crate::error::{AppError, AppResult};
use crate::library::model::Game;
use crate::{downloads, sources};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, State};

fn key_of(source_id: i64, game_id: &str) -> String {
    format!("{source_id}:{game_id}")
}

#[tauri::command]
pub fn source_list(state: State<'_, AppState>) -> AppResult<Vec<sources::SourceInfo>> {
    state.with_db(sources::list)
}

/// Todos os jogos de todas as fontes, com o estado de instalação.
#[tauri::command]
pub fn source_games(state: State<'_, AppState>) -> AppResult<Vec<sources::SourceGame>> {
    state.with_db(sources::all_games)
}

/// Cadastra uma fonte pelo endereço do catálogo (ou atualiza, se já estiver cadastrada).
#[tauri::command]
pub async fn source_add(state: State<'_, AppState>, url: String) -> AppResult<sources::SourceInfo> {
    let url = sources::clean_url(&url)?;
    let u = url.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || sources::fetch(&u))
        .await
        .map_err(join_err)??;
    state.with_db(|c| sources::save(c, &url, &manifest))
}

#[tauri::command]
pub async fn source_refresh(state: State<'_, AppState>, id: i64) -> AppResult<sources::SourceInfo> {
    let url = state.with_db(|c| sources::url_of(c, id))?;
    let u = url.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || sources::fetch(&u))
        .await
        .map_err(join_err)??;
    state.with_db(|c| sources::save(c, &url, &manifest))
}

#[tauri::command]
pub fn source_remove(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.with_db(|c| sources::remove(c, id))
}

/// Baixa, confere e instala. O andamento chega pelo evento "install-progress".
#[tauri::command]
pub async fn install_game(
    app: AppHandle,
    state: State<'_, AppState>,
    source_id: i64,
    game_id: String,
) -> AppResult<Game> {
    let key = key_of(source_id, &game_id);
    let def = state.with_db(|c| sources::find_game(c, source_id, &game_id))?;
    let data = state.data_dir.clone();
    let flag = {
        let mut running = state.installing.lock().map_err(|_| AppError::Lock)?;
        if running.contains_key(&key) {
            return Err(AppError::Invalid("Este download já está em andamento.".into()));
        }
        let f = Arc::new(AtomicBool::new(false));
        running.insert(key.clone(), f.clone());
        f
    };
    let (job_app, job_key, job_def) = (app.clone(), key.clone(), def.clone());
    let result = tauri::async_runtime::spawn_blocking(move || {
        downloads::install(&job_app, &data, source_id, &job_def, &job_key, &flag)
    })
    .await
    .map_err(join_err);
    if let Ok(mut running) = state.installing.lock() {
        running.remove(&key);
    }
    let done = result??;
    state.with_db(|c| downloads::register(c, source_id, &def, &done))
}

/// Pausa o download. O pedaço já baixado fica guardado; "Baixar" de novo continua dele.
#[tauri::command]
pub fn pause_install(state: State<'_, AppState>, source_id: i64, game_id: String) -> AppResult<()> {
    let running = state.installing.lock().map_err(|_| AppError::Lock)?;
    if let Some(flag) = running.get(&key_of(source_id, &game_id)) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// Apaga a pasta do jogo e tira o jogo da biblioteca.
#[tauri::command]
pub async fn uninstall_game(state: State<'_, AppState>, source_id: i64, game_id: String) -> AppResult<()> {
    let (library_id, dir) = state.with_db(|c| downloads::uninstall_plan(c, source_id, &game_id))?;
    let data = state.data_dir.clone();
    tauri::async_runtime::spawn_blocking(move || downloads::delete_dir(&data, &dir))
        .await
        .map_err(join_err)??;
    state.with_db(|c| downloads::uninstall_finish(c, library_id))
}
