use super::AppState;
use crate::error::AppResult;
use crate::launcher;
use crate::library::model::{Game, GameUpdate, NewGame};
use crate::library::repository as repo;
use crate::session;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn list_games(
    state: State<'_, AppState>,
    query: Option<String>,
    favorites_only: Option<bool>,
) -> AppResult<Vec<Game>> {
    state.with_db(|c| repo::list(c, query.as_deref(), favorites_only.unwrap_or(false)))
}

#[tauri::command]
pub fn add_game(state: State<'_, AppState>, game: NewGame) -> AppResult<Game> {
    state.with_db(|c| repo::add(c, game))
}

#[tauri::command]
pub fn update_game(state: State<'_, AppState>, id: i64, patch: GameUpdate) -> AppResult<Game> {
    state.with_db(|c| repo::update(c, id, patch))
}

#[tauri::command]
pub fn set_favorite(state: State<'_, AppState>, id: i64, favorite: bool) -> AppResult<Game> {
    state.with_db(|c| repo::set_favorite(c, id, favorite))
}

#[tauri::command]
pub fn remove_game(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    state.with_db(|c| repo::remove(c, id))
}

#[tauri::command]
pub fn clear_image(state: State<'_, AppState>, id: i64, kind: String) -> AppResult<Game> {
    state.with_db(|c| repo::set_image(c, id, &kind, None))
}

/// Inicia o jogo e começa a acompanhá-lo (a interface mostra o carregamento, minimiza e volta).
#[tauri::command]
pub fn launch_game(app: AppHandle, state: State<'_, AppState>, id: i64) -> AppResult<Game> {
    let game = state.with_db(|c| repo::get(c, id))?;
    match launcher::launch(&game)? {
        Some(child) => session::watch_child(app, id, child),
        None if game.platform == "steam" => {
            if let Some(app_id) = game.platform_id.as_deref().and_then(|s| s.parse::<u32>().ok()) {
                session::watch_steam(app, id, app_id);
            }
        }
        None => session::emit_state(&app, id, "untracked"),
    }
    state.with_db(|c| repo::mark_played(c, id))
}
