use super::{join_err, AppState};
use crate::error::{AppError, AppResult};
use crate::library::model::Game;
use crate::library::repository as repo;
use crate::metadata::{self, IgdbEntry, IgdbGame, SgdbGame, SgdbImage};
use crate::{images, settings};
use std::path::PathBuf;
use tauri::State;

fn sgdb_key(state: &AppState) -> AppResult<String> {
    state
        .with_db(|c| settings::get(c, "sgdb_key"))?
        .ok_or_else(|| AppError::Invalid("Configure a chave do SteamGridDB em Configurações.".into()))
}

#[tauri::command]
pub async fn sgdb_search(state: State<'_, AppState>, term: String) -> AppResult<Vec<SgdbGame>> {
    let key = sgdb_key(state.inner())?;
    tauri::async_runtime::spawn_blocking(move || metadata::sgdb_search(&key, &term))
        .await
        .map_err(join_err)?
}

#[tauri::command]
pub async fn sgdb_images(
    state: State<'_, AppState>,
    game_id: i64,
    kind: String,
) -> AppResult<Vec<SgdbImage>> {
    let key = sgdb_key(state.inner())?;
    tauri::async_runtime::spawn_blocking(move || metadata::sgdb_images(&key, game_id, &kind))
        .await
        .map_err(join_err)?
}

/// Baixa a imagem escolhida pelo usuário e a associa ao jogo.
#[tauri::command]
pub async fn apply_image(
    state: State<'_, AppState>,
    id: i64,
    kind: String,
    url: String,
) -> AppResult<Game> {
    let data = state.data_dir.clone();
    let path = tauri::async_runtime::spawn_blocking(move || images::download(&data, &url))
        .await
        .map_err(join_err)??;
    let p = path.to_string_lossy().into_owned();
    state.with_db(|c| repo::set_image(c, id, &kind, Some(p.as_str())))
}

#[tauri::command]
pub async fn apply_local_image(
    state: State<'_, AppState>,
    id: i64,
    kind: String,
    path: String,
) -> AppResult<Game> {
    let data = state.data_dir.clone();
    let copied = tauri::async_runtime::spawn_blocking(move || images::import_local(&data, &PathBuf::from(path)))
        .await
        .map_err(join_err)??;
    let p = copied.to_string_lossy().into_owned();
    state.with_db(|c| repo::set_image(c, id, &kind, Some(p.as_str())))
}

/// Completa capas, fundo, logo e descrição de um jogo da Steam, preenchendo só o que falta.
#[tauri::command]
pub async fn enrich_steam_game(state: State<'_, AppState>, id: i64) -> AppResult<Game> {
    let game = state.with_db(|c| repo::get(c, id))?;
    let app_id = match (game.platform.as_str(), game.platform_id.clone()) {
        ("steam", Some(a)) => a,
        _ => return Err(AppError::Invalid("Este jogo não é da Steam.".into())),
    };
    let mut missing: Vec<&'static str> = Vec::new();
    if game.cover_path.is_none() {
        missing.push("cover");
    }
    if game.background_path.is_none() {
        missing.push("background");
    }
    if game.hero_path.is_none() {
        missing.push("hero");
    }
    if game.logo_path.is_none() {
        missing.push("logo");
    }
    let need_desc = game.description.is_none();
    let data = state.data_dir.clone();
    let (found, desc) = tauri::async_runtime::spawn_blocking(move || {
        metadata::enrich_steam(&data, &app_id, &missing, need_desc)
    })
    .await
    .map_err(join_err)?;
    state.with_db(|c| {
        for (kind, path) in &found {
            repo::set_image(c, id, kind, Some(path.as_str()))?;
        }
        if let Some(d) = desc {
            repo::set_description_if_empty(c, id, &d)?;
        }
        repo::get(c, id)
    })
}

fn igdb_keys(state: &AppState) -> AppResult<(String, String)> {
    let (id, secret) = state.with_db(|c| Ok((settings::get(c, "igdb_id")?, settings::get(c, "igdb_secret")?)))?;
    match (id, secret) {
        (Some(i), Some(s)) => Ok((i, s)),
        _ => Err(AppError::Invalid(
            "Configure o Client ID e o Client Secret do IGDB em Configurações.".into(),
        )),
    }
}

/// Capas e artes do IGDB para o usuário escolher.
#[tauri::command]
pub async fn igdb_images(state: State<'_, AppState>, game_id: i64, kind: String) -> AppResult<Vec<SgdbImage>> {
    let (id, secret) = igdb_keys(state.inner())?;
    tauri::async_runtime::spawn_blocking(move || metadata::igdb_images(&id, &secret, game_id, &kind))
        .await
        .map_err(join_err)?
}

/// Busca descrições no IGDB; o usuário escolhe qual usar.
#[tauri::command]
pub async fn igdb_search(state: State<'_, AppState>, term: String) -> AppResult<Vec<IgdbGame>> {
    let (id, secret) = state.with_db(|c| Ok((settings::get(c, "igdb_id")?, settings::get(c, "igdb_secret")?)))?;
    let (id, secret) = match (id, secret) {
        (Some(i), Some(s)) => (i, s),
        _ => {
            return Err(AppError::Invalid(
                "Configure o Client ID e o Client Secret do IGDB em Configurações.".into(),
            ))
        }
    };
    tauri::async_runtime::spawn_blocking(move || metadata::igdb_search(&id, &secret, &term))
        .await
        .map_err(join_err)?
}

/// Vitrine da Loja ("popular", "recent", "upcoming" ou "search" com o nome digitado).
#[tauri::command]
pub async fn igdb_discover(state: State<'_, AppState>, kind: String, term: String) -> AppResult<Vec<IgdbEntry>> {
    let (id, secret) = igdb_keys(state.inner())?;
    let now = crate::library::repository::now();
    tauri::async_runtime::spawn_blocking(move || metadata::igdb_discover(&id, &secret, &kind, &term, now))
        .await
        .map_err(join_err)?
}
