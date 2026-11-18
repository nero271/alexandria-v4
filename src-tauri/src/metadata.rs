//! Busca de capas e informações: SteamGridDB (com chave) e Steam (sem chave).
use crate::error::{AppError, AppResult};
use crate::{images, net};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SgdbGame {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SgdbImage {
    pub url: String,
    pub thumb: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
}

fn igdb_token_fresh(client_id: &str, secret: &str) -> AppResult<String> {
    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
        encode(client_id),
        encode(secret)
    );
    let tok = ureq::post(&url)
        .timeout(Duration::from_secs(15))
        .call()
        .map_err(net::map_err)?
        .into_json::<serde_json::Value>()
        .map_err(|e| AppError::Invalid(format!("Resposta inválida da Twitch: {e}")))?;
    tok.get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from)
        .ok_or_else(|| AppError::Invalid("Não foi possível autenticar no IGDB. Confira o Client ID e o Client Secret.".into()))
}

static TOKEN: Mutex<Option<(String, String, Instant)>> = Mutex::new(None);
const TOKEN_TTL: Duration = Duration::from_secs(50 * 60);

/// Token da Twitch com cache de 50 minutos (evita uma autenticação a cada busca).
fn igdb_token(client_id: &str, secret: &str) -> AppResult<String> {
    if let Ok(g) = TOKEN.lock() {
        if let Some((id, tok, at)) = g.as_ref() {
            if id == client_id && at.elapsed() < TOKEN_TTL {
                return Ok(tok.clone());
            }
        }
    }
    let tok = igdb_token_fresh(client_id, secret)?;
    if let Ok(mut g) = TOKEN.lock() {
        *g = Some((client_id.to_string(), tok.clone(), Instant::now()));
    }
    Ok(tok)
}

fn igdb_query(client_id: &str, token: &str, body: &str) -> AppResult<serde_json::Value> {
    ureq::post("https://api.igdb.com/v4/games")
        .timeout(Duration::from_secs(15))
        .set("Client-ID", client_id)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/json")
        .send_string(body)
        .map_err(net::map_err)?
        .into_json::<serde_json::Value>()
        .map_err(|e| AppError::Invalid(format!("Resposta inválida do IGDB: {e}")))
}

/// Busca no IGDB (autenticação da Twitch por client credentials).
pub fn igdb_search(client_id: &str, secret: &str, term: &str) -> AppResult<Vec<IgdbGame>> {
    let term = term.trim();
    if term.is_empty() {
        return Ok(vec![]);
    }
    let token = igdb_token(client_id, secret)?;
    let safe: String = term.chars().filter(|c| *c != '"' && *c != '\\' && !c.is_control()).collect();
    let v = igdb_query(client_id, &token, &format!("search \"{safe}\"; fields name,summary; limit 6;"))?;
    let list = v.as_array().cloned().unwrap_or_default();
    Ok(list
        .iter()
        .filter_map(|g| {
            Some(IgdbGame {
                id: g.get("id")?.as_i64()?,
                name: g.get("name")?.as_str()?.to_string(),
                summary: g
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            })
        })
        .collect())
}

fn igdb_url(size: &str, image_id: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/{size}/{image_id}.jpg")
}

/// Capas (cover) e artes/capturas (background e hero) de um jogo do IGDB.
pub fn igdb_images(client_id: &str, secret: &str, game_id: i64, kind: &str) -> AppResult<Vec<SgdbImage>> {
    if !matches!(kind, "cover" | "background" | "hero") {
        return Err(AppError::Invalid("O IGDB não tem logos. Use o SteamGridDB para logos.".into()));
    }
    let token = igdb_token(client_id, secret)?;
    let body = format!("where id = {game_id}; fields cover.image_id,artworks.image_id,screenshots.image_id; limit 1;");
    let v = igdb_query(client_id, &token, &body)?;
    let g = v.as_array().and_then(|a| a.first()).cloned().unwrap_or(serde_json::Value::Null);
    let ids = |key: &str| -> Vec<String> {
        g.get(key)
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("image_id")?.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    };
    if kind == "cover" {
        let id = g.get("cover").and_then(|c| c.get("image_id")).and_then(|s| s.as_str());
        return Ok(id
            .map(|i| vec![SgdbImage { url: igdb_url("t_cover_big_2x", i), thumb: igdb_url("t_cover_big", i) }])
            .unwrap_or_default());
    }
    let mut all = ids("artworks");
    all.extend(ids("screenshots"));
    Ok(all
        .iter()
        .map(|i| SgdbImage { url: igdb_url("t_1080p", i), thumb: igdb_url("t_screenshot_med", i) })
        .collect())
}

/// Jogo do IGDB para a vitrine da Loja.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IgdbEntry {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<String>,
    pub year: Option<i32>,
    pub rating: Option<i64>,
    pub genres: Vec<String>,
    pub platforms: Vec<String>,
    /// Páginas de lojas conhecidas (a melhor primeiro).
    pub links: Vec<String>,
    /// Página do jogo no IGDB.
    pub page: Option<String>,
}

fn names(g: &serde_json::Value, key: &str, field: &str) -> Vec<String> {
    g.get(key)
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.get(field).and_then(|n| n.as_str()).map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn entry_from_json(g: &serde_json::Value) -> Option<IgdbEntry> {
    let id = g.get("id")?.as_i64()?;
    let name = g.get("name")?.as_str()?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    let text = |k: &str| g.get(k).and_then(|s| s.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let mut links: Vec<String> = names(g, "websites", "url").into_iter().filter(|u| crate::safe::store_link_ok(u)).collect();
    links.sort_by_key(|u| crate::safe::store_link_rank(u));
    links.dedup();
    links.truncate(4);
    let mut platforms: Vec<String> = Vec::new();
    if let Some(arr) = g.get("platforms").and_then(|a| a.as_array()) {
        for p in arr {
            let label = p
                .get("abbreviation")
                .and_then(|s| s.as_str())
                .or_else(|| p.get("name").and_then(|s| s.as_str()))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            if let Some(l) = label {
                if !platforms.contains(&l) {
                    platforms.push(l);
                }
            }
        }
    }
    platforms.truncate(8);
    let mut genres = names(g, "genres", "name");
    genres.truncate(4);
    Some(IgdbEntry {
        id,
        name,
        summary: text("summary"),
        cover: g
            .get("cover")
            .and_then(|c| c.get("image_id"))
            .and_then(|s| s.as_str())
            .map(|i| igdb_url("t_cover_big", i)),
        year: g
            .get("first_release_date")
            .and_then(|v| v.as_i64())
            .map(|ts| crate::stats::year_of_day(ts.div_euclid(86_400)) as i32),
        rating: g.get("total_rating").and_then(|v| v.as_f64()).map(|r| r.round() as i64),
        genres,
        platforms,
        links,
        page: text("url").filter(|u| crate::safe::store_link_ok(u)),
    })
}

fn sanitize_term(term: &str) -> String {
    term.chars()
        .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
        .take(80)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Monta a consulta do IGDB. `now` é o instante atual (segundos Unix).
pub fn discover_query(kind: &str, term: &str, now: i64) -> AppResult<String> {
    const FIELDS: &str = "fields name,summary,url,first_release_date,total_rating,cover.image_id,genres.name,platforms.abbreviation,platforms.name,websites.url;";
    const DAY: i64 = 86_400;
    match kind {
        "search" => {
            let t = sanitize_term(term);
            if t.is_empty() {
                return Err(AppError::Invalid("Digite o nome de um jogo.".into()));
            }
            Ok(format!("search \"{t}\"; {FIELDS} where version_parent = null; limit 30;"))
        }
        "popular" => Ok(format!(
            "{FIELDS} where total_rating_count > 80 & version_parent = null & cover != null; sort total_rating_count desc; limit 30;"
        )),
        "recent" => Ok(format!(
            "{FIELDS} where first_release_date >= {} & first_release_date <= {} & cover != null & version_parent = null & total_rating_count > 0; sort total_rating_count desc; limit 30;",
            now - 120 * DAY,
            now
        )),
        "upcoming" => Ok(format!(
            "{FIELDS} where first_release_date > {} & first_release_date <= {} & cover != null & version_parent = null; sort first_release_date asc; limit 30;",
            now,
            now + 365 * DAY
        )),
        _ => Err(AppError::Invalid("Tipo de busca desconhecido.".into())),
    }
}

/// Vitrine da Loja: populares, lançamentos, em breve ou busca por nome.
pub fn igdb_discover(client_id: &str, secret: &str, kind: &str, term: &str, now: i64) -> AppResult<Vec<IgdbEntry>> {
    let body = discover_query(kind, term, now)?;
    let token = igdb_token(client_id, secret)?;
    let v = igdb_query(client_id, &token, &body)?;
    Ok(v.as_array().map(|a| a.iter().filter_map(entry_from_json).collect()).unwrap_or_default())
}

pub fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn sgdb_search(key: &str, term: &str) -> AppResult<Vec<SgdbGame>> {
    let term = term.trim();
    if term.is_empty() {
        return Ok(vec![]);
    }
    let url = format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{}", encode(term));
    let v = net::get_json(&url, Some(key))?;
    let list = v.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
    Ok(list
        .iter()
        .filter_map(|g| {
            Some(SgdbGame {
                id: g.get("id")?.as_i64()?,
                name: g.get("name")?.as_str()?.to_string(),
            })
        })
        .take(8)
        .collect())
}

pub fn sgdb_images(key: &str, game_id: i64, kind: &str) -> AppResult<Vec<SgdbImage>> {
    let path = match kind {
        "cover" => format!("grids/game/{game_id}?dimensions=600x900"),
        "hero" => format!("grids/game/{game_id}?dimensions=920x430,460x215"),
        "background" => format!("heroes/game/{game_id}"),
        "logo" => format!("logos/game/{game_id}"),
        _ => return Err(AppError::Invalid("Tipo de imagem desconhecido.".into())),
    };
    let v = net::get_json(&format!("https://www.steamgriddb.com/api/v2/{path}"), Some(key))?;
    let list = v.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
    Ok(list
        .iter()
        .filter_map(|i| {
            let url = i.get("url")?.as_str()?.to_string();
            let thumb = i.get("thumb").and_then(|t| t.as_str()).unwrap_or(&url).to_string();
            Some(SgdbImage { url, thumb })
        })
        .take(24)
        .collect())
}

fn steam_art_url(app_id: &str, kind: &str) -> Option<String> {
    let file = match kind {
        "cover" => "library_600x900.jpg",
        "background" => "library_hero.jpg",
        "hero" => "header.jpg",
        "logo" => "logo.png",
        _ => return None,
    };
    Some(format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/{file}"))
}

pub fn steam_description(app_id: &str) -> AppResult<Option<String>> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={app_id}&l=brazilian");
    let v = net::get_json(&url, None)?;
    Ok(v.get(app_id)
        .and_then(|e| e.get("data"))
        .and_then(|d| d.get("short_description"))
        .and_then(|s| s.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()))
}

/// Baixa as imagens que faltam e a descrição, sem nunca falhar o conjunto por um item.
pub fn enrich_steam(
    data: &Path,
    app_id: &str,
    missing: &[&'static str],
    need_desc: bool,
) -> (Vec<(&'static str, String)>, Option<String>) {
    let mut found = Vec::new();
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) {
        return (found, None);
    }
    for kind in missing {
        if let Some(url) = steam_art_url(app_id, kind) {
            if let Ok(p) = images::download(data, &url) {
                found.push((*kind, p.to_string_lossy().into_owned()));
            }
        }
    }
    let desc = if need_desc { steam_description(app_id).ok().flatten() } else { None };
    (found, desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_queries() {
        let q = discover_query("search", "Hollow \"Knight\"\\", 0).unwrap();
        assert!(q.starts_with("search \"Hollow Knight\";"));
        assert!(discover_query("search", "   ", 0).is_err());
        assert!(discover_query("popular", "", 0).unwrap().contains("sort total_rating_count desc"));
        let r = discover_query("recent", "", 10_000_000).unwrap();
        assert!(r.contains("first_release_date <= 10000000"));
        assert!(discover_query("upcoming", "", 100).unwrap().contains("first_release_date > 100"));
        assert!(discover_query("outro", "", 0).is_err());
    }

    #[test]
    fn entry_parsing_filters_links() {
        let v = serde_json::json!({
            "id": 7, "name": " Portal 2 ", "summary": "Resumo", "url": "https://www.igdb.com/games/portal-2",
            "first_release_date": 1_302_652_800, "total_rating": 94.6,
            "cover": {"image_id": "co1abc"},
            "genres": [{"name": "Puzzle"}], "platforms": [{"abbreviation": "PC"}, {"name": "Xbox 360"}],
            "websites": [{"url": "https://evil.com/x"}, {"url": "https://www.gog.com/game/portal_2"}, {"url": "https://store.steampowered.com/app/620"}]
        });
        let e = entry_from_json(&v).unwrap();
        assert_eq!((e.name.as_str(), e.year, e.rating), ("Portal 2", Some(2011), Some(95)));
        assert_eq!(e.links, vec!["https://store.steampowered.com/app/620", "https://www.gog.com/game/portal_2"]);
        assert_eq!(e.platforms, vec!["PC", "Xbox 360"]);
        assert!(e.cover.unwrap().ends_with("t_cover_big/co1abc.jpg"));
        assert!(entry_from_json(&serde_json::json!({"id": 1})).is_none());
    }

    #[test]
    fn percent_encoding() {
        assert_eq!(encode("Red Dead 2"), "Red%20Dead%202");
        assert_eq!(encode("Pokémon"), "Pok%C3%A9mon");
        assert_eq!(encode("a-b_c.d~"), "a-b_c.d~");
    }

    #[test]
    fn steam_urls_only_for_known_kinds() {
        assert!(steam_art_url("620", "cover").unwrap().ends_with("/620/library_600x900.jpg"));
        assert!(steam_art_url("620", "x").is_none());
    }
}
