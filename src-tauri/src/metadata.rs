//! Busca de capas e informações: SteamGridDB (com chave) e Steam (sem chave).
use crate::error::{AppError, AppResult};
use crate::{images, net};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

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

fn igdb_token(client_id: &str, secret: &str) -> AppResult<String> {
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
