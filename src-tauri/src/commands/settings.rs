use super::{join_err, AppState};
use crate::error::AppResult;
use crate::{images, settings};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

/// A chave de API nunca volta para a interface: só a informação de que existe.
fn public(mut map: HashMap<String, String>) -> HashMap<String, String> {
    if map.remove("sgdb_key").is_some() {
        map.insert("sgdb_key_set".into(), "1".into());
    }
    if map.remove("igdb_secret").is_some() {
        map.insert("igdb_secret_set".into(), "1".into());
    }
    map
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<HashMap<String, String>> {
    state.with_db(|c| settings::all(c)).map(public)
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<HashMap<String, String>> {
    state
        .with_db(|c| {
            settings::set(c, &key, &value)?;
            settings::all(c)
        })
        .map(public)
}

#[tauri::command]
pub async fn set_app_background(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<HashMap<String, String>> {
    let data = state.data_dir.clone();
    let copied = tauri::async_runtime::spawn_blocking(move || images::import_local(&data, &PathBuf::from(path)))
        .await
        .map_err(join_err)??;
    state
        .with_db(|c| {
            settings::set(c, "app_bg", &copied.to_string_lossy())?;
            settings::all(c)
        })
        .map(public)
}
