pub mod app;
pub mod backup;
pub mod extras;
pub mod library;
pub mod metadata;
pub mod platforms;
pub mod settings;
pub mod sources;
pub mod steam;

use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
    /// Downloads em andamento ("fonte:jogo"); marcar o AtomicBool pausa o download.
    pub installing: Mutex<HashMap<String, Arc<AtomicBool>>>,
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
