use super::model::{Game, GameUpdate, InstallState, NewGame, SyncSummary};
use crate::platforms::ExternalApp;
use crate::steam::SteamApp;
use std::collections::HashSet;
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension, Row};
use std::time::{SystemTime, UNIX_EPOCH};

const COLUMNS: &str = "id, name, platform, platform_id, launch_target, category, \
    install_state, favorite, last_played_at, description, cover_path, hero_path, \
    background_path, logo_path, created_at, updated_at, playtime_minutes";

const MAX_NAME_LEN: usize = 200;

pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Remove espaços nas pontas; texto vazio vira None.
fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn valid_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("O nome do jogo é obrigatório.".into()));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Invalid(format!(
            "O nome do jogo deve ter até {MAX_NAME_LEN} caracteres."
        )));
    }
    Ok(name.to_string())
}

fn from_row(r: &Row) -> rusqlite::Result<Game> {
    Ok(Game {
        id: r.get(0)?,
        name: r.get(1)?,
        platform: r.get(2)?,
        platform_id: r.get(3)?,
        launch_target: r.get(4)?,
        category: r.get(5)?,
        install_state: InstallState::parse(&r.get::<_, String>(6)?),
        favorite: r.get::<_, i64>(7)? != 0,
        last_played_at: r.get(8)?,
        description: r.get(9)?,
        cover_path: r.get(10)?,
        hero_path: r.get(11)?,
        background_path: r.get(12)?,
        logo_path: r.get(13)?,
        created_at: r.get(14)?,
        updated_at: r.get(15)?,
        playtime_minutes: r.get(16)?,
    })
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Game> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM games WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub fn list(conn: &Connection, query: Option<&str>, favorites_only: bool) -> AppResult<Vec<Game>> {
    let q = query.map(str::trim).filter(|s| !s.is_empty());
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM games \
         WHERE (?1 IS NULL OR name LIKE '%' || ?1 || '%') \
           AND (?2 = 0 OR favorite = 1) \
         ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map(params![q, favorites_only as i64], from_row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn add(conn: &Connection, new: NewGame) -> AppResult<Game> {
    let name = valid_name(&new.name)?;
    let platform = clean(new.platform).unwrap_or_else(|| "manual".to_string());
    let ts = now();
    let res = conn.execute(
        "INSERT INTO games (name, platform, platform_id, launch_target, category, \
         install_state, description, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![
            name,
            platform,
            clean(new.platform_id),
            clean(new.launch_target),
            clean(new.category),
            InstallState::Unknown.as_str(),
            clean(new.description),
            ts
        ],
    );
    match res {
        Ok(_) => get(conn, conn.last_insert_rowid()),
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == ErrorCode::ConstraintViolation => {
            Err(AppError::Invalid("Este jogo já está na biblioteca.".into()))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool) -> AppResult<Game> {
    let n = conn.execute(
        "UPDATE games SET favorite = ?1, updated_at = ?2 WHERE id = ?3",
        params![favorite as i64, now(), id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn update(conn: &Connection, id: i64, patch: GameUpdate) -> AppResult<Game> {
    let cur = get(conn, id)?;
    let name = match &patch.name {
        Some(n) => valid_name(n)?,
        None => cur.name.clone(),
    };
    // A plataforma de jogos da Steam não pode ser trocada.
    let platform = match patch.platform {
        Some(p) if cur.platform != "steam" => clean(Some(p)).unwrap_or_else(|| cur.platform.clone()),
        _ => cur.platform.clone(),
    };
    let launch_target = patch.launch_target.map_or(cur.launch_target.clone(), |v| clean(Some(v)));
    let category = patch.category.map_or(cur.category.clone(), |v| clean(Some(v)));
    let description = patch.description.map_or(cur.description.clone(), |v| clean(Some(v)));
    conn.execute(
        "UPDATE games SET name = ?1, platform = ?2, launch_target = ?3, category = ?4, \
         description = ?5, updated_at = ?6 WHERE id = ?7",
        params![name, platform, launch_target, category, description, now(), id],
    )?;
    get(conn, id)
}

fn image_column(kind: &str) -> AppResult<&'static str> {
    match kind {
        "cover" => Ok("cover_path"),
        "background" => Ok("background_path"),
        "hero" => Ok("hero_path"),
        "logo" => Ok("logo_path"),
        _ => Err(AppError::Invalid("Tipo de imagem desconhecido.".into())),
    }
}

/// `path = None` limpa a imagem.
pub fn set_image(conn: &Connection, id: i64, kind: &str, path: Option<&str>) -> AppResult<Game> {
    let col = image_column(kind)?;
    let n = conn.execute(
        &format!("UPDATE games SET {col} = ?1, updated_at = ?2 WHERE id = ?3"),
        params![path, now(), id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn set_description_if_empty(conn: &Connection, id: i64, text: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE games SET description = ?1, updated_at = ?2 WHERE id = ?3 AND description IS NULL",
        params![text, now(), id],
    )?;
    Ok(())
}

pub fn mark_played(conn: &Connection, id: i64) -> AppResult<Game> {
    let n = conn.execute(
        "UPDATE games SET last_played_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now(), id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    get(conn, id)
}

pub fn add_playtime(conn: &Connection, id: i64, minutes: i64) -> AppResult<()> {
    conn.execute(
        "UPDATE games SET playtime_minutes = playtime_minutes + ?1, updated_at = ?2 WHERE id = ?3",
        params![minutes, now(), id],
    )?;
    Ok(())
}

/// Sincroniza jogos instalados da Steam.
pub fn sync_steam(conn: &Connection, apps: &[SteamApp]) -> AppResult<SyncSummary> {
    let list: Vec<ExternalApp> = apps
        .iter()
        .map(|a| ExternalApp {
            app_id: a.app_id.clone(),
            name: a.name.clone(),
            launch_target: Some(format!("steam://rungameid/{}", a.app_id)),
            playtime_minutes: a.playtime,
        })
        .collect();
    sync_platform(conn, "steam", &list)
}

/// Sincroniza jogos de uma plataforma (Steam, Epic, GOG). Nunca sobrescreve nome,
/// imagens ou outras personalizações de jogos já cadastrados.
pub fn sync_platform(conn: &Connection, platform: &str, apps: &[ExternalApp]) -> AppResult<SyncSummary> {
    let tx = conn.unchecked_transaction()?;
    let ts = now();
    let mut s = SyncSummary::default();
    for a in apps {
        let existing: Option<(i64, String)> = tx
            .query_row(
                "SELECT id, install_state FROM games WHERE platform = ?1 AND platform_id = ?2",
                params![platform, a.app_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match existing {
            Some((id, state)) => {
                if state != "installed" {
                    tx.execute(
                        "UPDATE games SET install_state = 'installed', updated_at = ?1 WHERE id = ?2",
                        params![ts, id],
                    )?;
                    s.updated += 1;
                }
                if let Some(p) = a.playtime_minutes {
                    tx.execute(
                        "UPDATE games SET playtime_minutes = ?1 WHERE id = ?2 AND playtime_minutes <> ?1",
                        params![p, id],
                    )?;
                }
            }
            None => {
                tx.execute(
                    "INSERT INTO games (name, platform, platform_id, launch_target, install_state, \
                     playtime_minutes, created_at, updated_at) \
                     VALUES (?1, ?2, ?3, ?4, 'installed', ?5, ?6, ?6)",
                    params![a.name, platform, a.app_id, a.launch_target, a.playtime_minutes.unwrap_or(0), ts],
                )?;
                s.added += 1;
            }
        }
    }
    let present: HashSet<&str> = apps.iter().map(|a| a.app_id.as_str()).collect();
    let gone: Vec<i64> = {
        let mut stmt = tx.prepare(
            "SELECT id, platform_id FROM games WHERE platform = ?1 \
             AND install_state = 'installed' AND platform_id IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![platform], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        let mut v = Vec::new();
        for row in rows {
            let (id, pid) = row?;
            if !present.contains(pid.as_str()) {
                v.push(id);
            }
        }
        v
    };
    for id in gone {
        tx.execute(
            "UPDATE games SET install_state = 'not_installed', updated_at = ?1 WHERE id = ?2",
            params![ts, id],
        )?;
        s.uninstalled += 1;
    }
    tx.commit()?;
    Ok(s)
}

pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    let n = conn.execute("DELETE FROM games WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn new(name: &str) -> NewGame {
        NewGame {
            name: name.into(),
            platform: None,
            platform_id: None,
            launch_target: None,
            category: None,
            description: None,
        }
    }

    #[test]
    fn add_and_list_sorted_case_insensitive() {
        let c = open_in_memory().unwrap();
        add(&c, new("stardew Valley")).unwrap();
        add(&c, new("DOOM")).unwrap();
        add(&c, new("Celeste")).unwrap();
        let names: Vec<_> = list(&c, None, false).unwrap().into_iter().map(|g| g.name).collect();
        assert_eq!(names, ["Celeste", "DOOM", "stardew Valley"]);
    }

    #[test]
    fn rejects_empty_name() {
        let c = open_in_memory().unwrap();
        assert!(matches!(add(&c, new("   ")), Err(AppError::Invalid(_))));
    }

    #[test]
    fn rejects_duplicate_platform_id() {
        let c = open_in_memory().unwrap();
        let mut g = new("Hades");
        g.platform = Some("steam".into());
        g.platform_id = Some("1145360".into());
        add(&c, g.clone()).unwrap();
        assert!(matches!(add(&c, g), Err(AppError::Invalid(_))));
    }

    #[test]
    fn search_and_favorites_filter() {
        let c = open_in_memory().unwrap();
        let a = add(&c, new("Red Dead Redemption 2")).unwrap();
        add(&c, new("Doom Eternal")).unwrap();
        set_favorite(&c, a.id, true).unwrap();
        assert_eq!(list(&c, Some("doom"), false).unwrap().len(), 1);
        let favs = list(&c, None, true).unwrap();
        assert_eq!(favs.len(), 1);
        assert!(favs[0].favorite);
    }

    #[test]
    fn update_changes_and_clears_fields() {
        let c = open_in_memory().unwrap();
        let mut n = new("Celeste");
        n.category = Some("Plataforma".into());
        let g = add(&c, n).unwrap();
        let u = update(
            &c,
            g.id,
            GameUpdate {
                name: Some("Celeste 2".into()),
                category: Some("".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(u.name, "Celeste 2");
        assert_eq!(u.category, None);
    }

    #[test]
    fn remove_and_not_found() {
        let c = open_in_memory().unwrap();
        let g = add(&c, new("Hollow Knight")).unwrap();
        remove(&c, g.id).unwrap();
        assert!(matches!(get(&c, g.id), Err(AppError::NotFound)));
        assert!(matches!(remove(&c, g.id), Err(AppError::NotFound)));
    }

    fn app(id: &str, name: &str) -> SteamApp {
        SteamApp { app_id: id.into(), name: name.into(), playtime: None }
    }

    #[test]
    fn steam_sync_adds_keeps_customizations_and_marks_uninstalled() {
        let c = open_in_memory().unwrap();
        let s1 = sync_steam(&c, &[app("1", "Hades"), app("2", "Celeste")]).unwrap();
        assert_eq!((s1.added, s1.updated, s1.uninstalled), (2, 0, 0));
        let hades = list(&c, Some("Hades"), false).unwrap().remove(0);
        update(&c, hades.id, GameUpdate { name: Some("Meu Hades".into()), ..Default::default() }).unwrap();
        // Celeste sai; Hades continua; nome personalizado é preservado.
        let s2 = sync_steam(&c, &[app("1", "Hades")]).unwrap();
        assert_eq!((s2.added, s2.updated, s2.uninstalled), (0, 0, 1));
        assert_eq!(get(&c, hades.id).unwrap().name, "Meu Hades");
        // Celeste volta.
        let s3 = sync_steam(&c, &[app("1", "Hades"), app("2", "Celeste")]).unwrap();
        assert_eq!((s3.added, s3.updated, s3.uninstalled), (0, 1, 0));
    }

    #[test]
    fn images_and_played_and_description() {
        let c = open_in_memory().unwrap();
        let g = add(&c, new("Hades")).unwrap();
        let g = set_image(&c, g.id, "cover", Some("C:/x.jpg")).unwrap();
        assert_eq!(g.cover_path.as_deref(), Some("C:/x.jpg"));
        let g = set_image(&c, g.id, "cover", None).unwrap();
        assert_eq!(g.cover_path, None);
        assert!(set_image(&c, g.id, "foo", None).is_err());
        set_description_if_empty(&c, g.id, "Primeira").unwrap();
        set_description_if_empty(&c, g.id, "Segunda").unwrap();
        assert_eq!(get(&c, g.id).unwrap().description.as_deref(), Some("Primeira"));
        assert!(mark_played(&c, g.id).unwrap().last_played_at.is_some());
    }

    #[test]
    fn platform_sync_is_scoped_and_updates_playtime() {
        let c = open_in_memory().unwrap();
        let epic = |id: &str, name: &str| ExternalApp {
            app_id: id.into(),
            name: name.into(),
            launch_target: Some("C:/x.exe".into()),
            playtime_minutes: None,
        };
        sync_platform(&c, "epic", &[epic("a", "Fortnite")]).unwrap();
        sync_steam(&c, &[SteamApp { app_id: "1".into(), name: "Hades".into(), playtime: Some(90) }]).unwrap();
        // Sincronizar a Epic sem jogos não mexe nos jogos da Steam.
        let s = sync_platform(&c, "epic", &[]).unwrap();
        assert_eq!(s.uninstalled, 1);
        let hades = list(&c, Some("Hades"), false).unwrap().remove(0);
        assert_eq!(hades.install_state, InstallState::Installed);
        assert_eq!(hades.playtime_minutes, 90);
        add_playtime(&c, hades.id, 15).unwrap();
        assert_eq!(get(&c, hades.id).unwrap().playtime_minutes, 105);
    }
}
