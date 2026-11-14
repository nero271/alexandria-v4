pub mod library;
pub mod metadata;
pub mod settings;
pub mod steam;

use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn with_db<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let guard = self.db.lock().map_err(|_| AppError::Lock)?;
        f(&guard)
    }
}

/// Erro de uma tarefa em segundo plano que não chegou a terminar.
pub fn join_err(e: tauri::Error) -> AppError {
    AppError::Invalid(format!("A tarefa em segundo plano falhou: {e}"))
}
