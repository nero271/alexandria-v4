//! Lista de desejos (local): jogos do IGDB que o usuário quer acompanhar.
use crate::error::{AppError, AppResult};
use crate::library::repository::now;
use crate::safe;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WishItem {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
    pub year: Option<i32>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WishInput {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
    pub year: Option<i32>,
    pub url: Option<String>,
}

pub fn list(conn: &Connection) -> AppResult<Vec<WishItem>> {
    let mut stmt = conn.prepare("SELECT igdb_id, name, cover, year, url FROM wishlist ORDER BY added_at DESC")?;
    let rows = stmt
        .query_map([], |r| Ok(WishItem { id: r.get(0)?, name: r.get(1)?, cover: r.get(2)?, year: r.get(3)?, url: r.get(4)? }))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn add(conn: &Connection, item: WishInput) -> AppResult<()> {
    let name = item.name.trim();
    if item.id <= 0 || name.is_empty() || name.chars().count() > 200 {
        return Err(AppError::Invalid("Jogo inválido.".into()));
    }
    let cover = item.cover.filter(|c| safe::is_https_url(c));
    let url = item.url.filter(|u| safe::store_link_ok(u));
    conn.execute(
        "INSERT INTO wishlist (igdb_id, name, cover, year, url, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(igdb_id) DO UPDATE SET name = excluded.name, cover = excluded.cover, year = excluded.year, url = excluded.url",
        params![item.id, name, cover, item.year, url, now()],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM wishlist WHERE igdb_id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn input(id: i64, url: &str) -> WishInput {
        WishInput { id, name: "Hades II".into(), cover: Some("https://images.igdb.com/x.jpg".into()), year: Some(2024), url: Some(url.into()) }
    }

    #[test]
    fn add_list_remove_and_filter_links() {
        let c = open_in_memory().unwrap();
        add(&c, input(1, "https://store.steampowered.com/app/1")).unwrap();
        add(&c, input(2, "https://evil.com/x")).unwrap();
        add(&c, input(1, "https://store.steampowered.com/app/1")).unwrap(); // repetido: atualiza
        let l = list(&c).unwrap();
        assert_eq!(l.len(), 2);
        assert!(l.iter().find(|w| w.id == 2).unwrap().url.is_none());
        remove(&c, 1).unwrap();
        assert_eq!(list(&c).unwrap().len(), 1);
        assert!(add(&c, WishInput { id: 0, name: "x".into(), cover: None, year: None, url: None }).is_err());
    }
}
