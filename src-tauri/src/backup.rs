//! Backup e restauração da biblioteca e das personalizações (sem chaves de API).
use crate::error::{AppError, AppResult};
use crate::library::repository as repo;
use crate::settings;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const FORMAT: u32 = 1;
const SKIP_SETTINGS: [&str; 3] = ["sgdb_key", "igdb_secret", "app_bg"];

#[derive(Serialize, Deserialize)]
struct BackupGame {
    name: String,
    platform: String,
    platform_id: Option<String>,
    launch_target: Option<String>,
    category: Option<String>,
    favorite: bool,
    description: Option<String>,
    last_played_at: Option<i64>,
    #[serde(default)]
    playtime_minutes: i64,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    rating: Option<i64>,
    cover_path: Option<String>,
    hero_path: Option<String>,
    background_path: Option<String>,
    logo_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Backup {
    format: u32,
    games: Vec<BackupGame>,
    settings: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub added: usize,
    pub skipped: usize,
}

pub fn export(conn: &Connection, path: &Path) -> AppResult<usize> {
    let games: Vec<BackupGame> = repo::list(conn, None, false)?
        .into_iter()
        .map(|g| BackupGame {
            name: g.name,
            platform: g.platform,
            platform_id: g.platform_id,
            launch_target: g.launch_target,
            category: g.category,
            favorite: g.favorite,
            description: g.description,
            last_played_at: g.last_played_at,
            playtime_minutes: g.playtime_minutes,
            status: g.status,
            rating: g.rating,
            cover_path: g.cover_path,
            hero_path: g.hero_path,
            background_path: g.background_path,
            logo_path: g.logo_path,
        })
        .collect();
    let count = games.len();
    let mut settings = settings::all(conn)?;
    for k in SKIP_SETTINGS {
        settings.remove(k);
    }
    let json = serde_json::to_string_pretty(&Backup { format: FORMAT, games, settings })
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    std::fs::write(path, json)?;
    Ok(count)
}

/// Importa sem sobrescrever: jogos que já existem são ignorados.
pub fn import(conn: &Connection, path: &Path) -> AppResult<BackupSummary> {
    let text = std::fs::read_to_string(path)?;
    let b: Backup = serde_json::from_str(&text)
        .map_err(|_| AppError::Invalid("Este arquivo não é um backup válido do ALEXANDRIA.".into()))?;
    if b.format != FORMAT {
        return Err(AppError::Invalid("Versão de backup não suportada.".into()));
    }
    let keep = |p: Option<String>| p.filter(|x| Path::new(x).exists());
    let tx = conn.unchecked_transaction()?;
    let ts = repo::now();
    let mut summary = BackupSummary { added: 0, skipped: 0 };
    for g in b.games {
        let name = g.name.trim().to_string();
        if name.is_empty() || name.chars().count() > 200 || g.platform.trim().is_empty() {
            summary.skipped += 1;
            continue;
        }
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE platform = ?1 AND \
             ((?2 IS NOT NULL AND platform_id = ?2) OR lower(name) = lower(?3)))",
            params![g.platform, g.platform_id, name],
            |r| r.get(0),
        )?;
        if exists {
            summary.skipped += 1;
            continue;
        }
        tx.execute(
            "INSERT INTO games (name, platform, platform_id, launch_target, category, install_state, \
             favorite, last_played_at, description, cover_path, hero_path, background_path, logo_path, \
             playtime_minutes, status, rating, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'unknown', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
            params![
                name, g.platform, g.platform_id, g.launch_target, g.category, g.favorite as i64,
                g.last_played_at, g.description, keep(g.cover_path), keep(g.hero_path),
                keep(g.background_path), keep(g.logo_path), g.playtime_minutes.max(0),
                g.status.filter(|s| repo::STATUSES.contains(&s.as_str())),
                g.rating.filter(|r| (1..=5).contains(r)), ts
            ],
        )?;
        summary.added += 1;
    }
    tx.commit()?;
    for (k, v) in b.settings {
        if !SKIP_SETTINGS.contains(&k.as_str()) {
            let _ = settings::set(conn, &k, &v); // valores inválidos são ignorados
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;
    use crate::library::model::NewGame;

    fn new(name: &str) -> NewGame {
        NewGame { name: name.into(), platform: None, platform_id: None, launch_target: None, category: Some("X".into()), description: None }
    }

    #[test]
    fn export_then_import_roundtrip() {
        let a = open_in_memory().unwrap();
        repo::add(&a, new("Celeste")).unwrap();
        repo::add(&a, new("Hades")).unwrap();
        settings::set(&a, "app_name", "Minha").unwrap();
        settings::set(&a, "sgdb_key", "segredo").unwrap();
        let file = std::env::temp_dir().join("alexandria_backup_test.json");
        assert_eq!(export(&a, &file).unwrap(), 2);
        assert!(!std::fs::read_to_string(&file).unwrap().contains("segredo"));

        let b = open_in_memory().unwrap();
        repo::add(&b, new("Celeste")).unwrap();
        let s = import(&b, &file).unwrap();
        assert_eq!((s.added, s.skipped), (1, 1));
        assert_eq!(settings::get(&b, "app_name").unwrap().as_deref(), Some("Minha"));
        let _ = std::fs::remove_file(file);
    }
}
