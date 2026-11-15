pub mod migrations;

use crate::error::AppResult;
use rusqlite::Connection;
use std::path::Path;

const PRAGMAS: &str = "PRAGMA journal_mode = WAL;
                       PRAGMA synchronous = NORMAL;
                       PRAGMA foreign_keys = ON;";

/// Abre (ou cria) o banco em disco e aplica as migrações pendentes.
pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut conn = Connection::open(path)?;
    conn.execute_batch(PRAGMAS)?;
    migrations::run(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    let mut conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::run(&mut conn)?;
    Ok(conn)
}
