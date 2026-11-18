//! Fontes da Loja: catálogos (JSON) cujo endereço o próprio usuário cadastra.
//! O ALEXANDRIA não traz fontes prontas e não indica fontes.
use crate::error::{AppError, AppResult};
use crate::library::repository::now;
use crate::{net, safe};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Read;

const MAX_MANIFEST: u64 = 2 * 1024 * 1024;
const MAX_GAMES: usize = 500;

fn default_type() -> String {
    "zip".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestGame {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    pub url: String,
    /// Opcional: quando informado, o arquivo baixado é conferido.
    #[serde(default)]
    pub sha256: String,
    #[serde(rename = "type", default = "default_type")]
    pub kind: String,
    #[serde(default)]
    pub launch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub format: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub games: Vec<ManifestGame>,
}

fn clip(v: &mut Option<String>, max: usize) {
    *v = v
        .take()
        .map(|s| s.trim().chars().take(max).collect::<String>())
        .filter(|s| !s.is_empty());
}

fn launch_ok(l: &str) -> bool {
    safe::safe_rel_path(l) && safe::has_ext(l, &["exe", "bat"])
}

/// Confere o catálogo inteiro; o primeiro problema vira uma mensagem clara.
pub fn validate(m: &mut Manifest, fallback_name: &str) -> AppResult<()> {
    if m.format != 1 {
        return Err(AppError::Invalid("Catálogo não suportado: use \"format\": 1.".into()));
    }
    m.name = m.name.trim().chars().take(80).collect();
    if m.name.is_empty() {
        m.name = fallback_name.to_string();
    }
    if m.games.is_empty() {
        return Err(AppError::Invalid("O catálogo não tem nenhum jogo em \"games\".".into()));
    }
    if m.games.len() > MAX_GAMES {
        return Err(AppError::Invalid(format!("O catálogo tem jogos demais (limite de {MAX_GAMES}).")));
    }
    let mut seen = HashSet::new();
    for (i, g) in m.games.iter_mut().enumerate() {
        let bad = |msg: &str| AppError::Invalid(format!("Jogo nº {}: {msg}", i + 1));
        if !safe::valid_id(&g.id) {
            return Err(bad("\"id\" inválido (minúsculas, números, ponto, hífen ou sublinhado)."));
        }
        if !seen.insert(g.id.clone()) {
            return Err(bad("\"id\" repetido."));
        }
        g.name = g.name.trim().chars().take(200).collect();
        if g.name.is_empty() {
            return Err(bad("falta o \"name\"."));
        }
        g.version = g.version.trim().chars().take(40).collect();
        if g.version.is_empty() {
            g.version = "1".into();
        }
        clip(&mut g.description, 2000);
        clip(&mut g.category, 60);
        clip(&mut g.license, 60);
        g.cover = g.cover.take().map(|c| c.trim().to_string()).filter(|c| safe::is_https_url(c));
        g.url = g.url.trim().to_string();
        if !safe::is_https_url(&g.url) {
            return Err(bad("\"url\" precisa ser um endereço https:// completo."));
        }
        g.sha256 = g.sha256.trim().to_ascii_lowercase();
        if !g.sha256.is_empty() && !safe::is_sha256(&g.sha256) {
            return Err(bad("\"sha256\" deve ter 64 caracteres hexadecimais (ou ser omitido)."));
        }
        g.kind = g.kind.trim().to_ascii_lowercase();
        let launch = g.launch.take().map(|l| l.trim().to_string()).filter(|l| !l.is_empty());
        match g.kind.as_str() {
            "zip" => {
                let l = launch.ok_or_else(|| bad("falta \"launch\" (o .exe dentro do .zip)."))?;
                if !launch_ok(&l) {
                    return Err(bad("\"launch\" deve ser um caminho relativo terminado em .exe ou .bat."));
                }
                g.launch = Some(l);
            }
            "exe" => {
                if let Some(l) = &launch {
                    if !launch_ok(l) {
                        return Err(bad("\"launch\" deve ser um nome de arquivo terminado em .exe ou .bat."));
                    }
                }
                g.launch = launch;
            }
            _ => return Err(bad("\"type\" deve ser \"zip\" ou \"exe\".")),
        }
    }
    Ok(())
}

pub fn parse(text: &str, fallback_name: &str) -> AppResult<Manifest> {
    let text = text.trim_start_matches('\u{feff}');
    let mut m: Manifest = serde_json::from_str(text)
        .map_err(|e| AppError::Invalid(format!("O catálogo não é um JSON válido: {e}")))?;
    validate(&mut m, fallback_name)?;
    Ok(m)
}

pub fn clean_url(url: &str) -> AppResult<String> {
    let url = url.trim();
    if !safe::is_https_url(url) {
        return Err(AppError::Invalid("Use o endereço completo do catálogo, começando com https://".into()));
    }
    Ok(url.to_string())
}

/// Baixa e confere o catálogo de uma fonte.
pub fn fetch(url: &str) -> AppResult<Manifest> {
    let url = clean_url(url)?;
    let resp = net::get_https(&net::https_agent(), &url, None)?;
    let mut text = String::new();
    resp.into_reader()
        .take(MAX_MANIFEST + 1)
        .read_to_string(&mut text)
        .map_err(|_| AppError::Invalid("Não foi possível ler o catálogo (precisa ser texto JSON).".into()))?;
    if text.len() as u64 > MAX_MANIFEST {
        return Err(AppError::Invalid("O catálogo é grande demais (limite de 2 MB).".into()));
    }
    parse(&text, safe::https_host(&url).unwrap_or("fonte"))
}

// ---------------------------------------------------------------------------
// Banco de dados
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: i64,
    pub url: String,
    pub name: String,
    pub game_count: usize,
    pub synced_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceGame {
    pub source_id: i64,
    pub source_name: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub category: Option<String>,
    pub license: Option<String>,
    pub size: Option<u64>,
    pub host: String,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub update: bool,
    pub library_id: Option<i64>,
    /// `true` quando o catálogo informa o SHA-256 (o arquivo é conferido antes de instalar).
    pub verified: bool,
}

struct Row {
    id: i64,
    url: String,
    name: String,
    manifest: String,
    synced_at: i64,
}

fn rows(conn: &Connection) -> AppResult<Vec<Row>> {
    let mut stmt = conn.prepare("SELECT id, url, name, manifest, synced_at FROM sources ORDER BY name COLLATE NOCASE, id")?;
    let it = stmt.query_map([], |r| {
        Ok(Row { id: r.get(0)?, url: r.get(1)?, name: r.get(2)?, manifest: r.get(3)?, synced_at: r.get(4)? })
    })?;
    Ok(it.collect::<Result<Vec<_>, _>>()?)
}

fn info(r: &Row) -> SourceInfo {
    let count = serde_json::from_str::<Manifest>(&r.manifest).map(|m| m.games.len()).unwrap_or(0);
    SourceInfo { id: r.id, url: r.url.clone(), name: r.name.clone(), game_count: count, synced_at: r.synced_at }
}

/// Cadastra a fonte (ou atualiza o catálogo, se o endereço já existir).
pub fn save(conn: &Connection, url: &str, m: &Manifest) -> AppResult<SourceInfo> {
    let json = serde_json::to_string(m).map_err(|e| AppError::Invalid(e.to_string()))?;
    conn.execute(
        "INSERT INTO sources (url, name, manifest, synced_at) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(url) DO UPDATE SET name = excluded.name, manifest = excluded.manifest, synced_at = excluded.synced_at",
        params![url, m.name, json, now()],
    )?;
    let id: i64 = conn.query_row("SELECT id FROM sources WHERE url = ?1", params![url], |r| r.get(0))?;
    rows(conn)?
        .iter()
        .find(|r| r.id == id)
        .map(info)
        .ok_or_else(|| AppError::Invalid("Não foi possível salvar a fonte.".into()))
}

pub fn list(conn: &Connection) -> AppResult<Vec<SourceInfo>> {
    Ok(rows(conn)?.iter().map(info).collect())
}

/// Tira a fonte da lista. Os jogos já instalados continuam na biblioteca.
pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM sources WHERE id = ?1", [id])?;
    Ok(())
}

pub fn url_of(conn: &Connection, id: i64) -> AppResult<String> {
    rows(conn)?
        .into_iter()
        .find(|r| r.id == id)
        .map(|r| r.url)
        .ok_or_else(|| AppError::Invalid("Fonte não encontrada.".into()))
}

pub fn find_game(conn: &Connection, source_id: i64, game_id: &str) -> AppResult<ManifestGame> {
    let row = rows(conn)?
        .into_iter()
        .find(|r| r.id == source_id)
        .ok_or_else(|| AppError::Invalid("Fonte não encontrada.".into()))?;
    let m: Manifest = serde_json::from_str(&row.manifest).map_err(|e| AppError::Invalid(e.to_string()))?;
    m.games
        .into_iter()
        .find(|g| g.id == game_id)
        .ok_or_else(|| AppError::Invalid("Esse jogo não está mais no catálogo. Atualize a fonte.".into()))
}

/// Todos os jogos de todas as fontes, com o estado de instalação.
pub fn all_games(conn: &Connection) -> AppResult<Vec<SourceGame>> {
    let mut stmt = conn.prepare("SELECT key, version, game_id FROM installs")?;
    let installed: HashMap<String, (String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get::<_, String>(1)?, r.get::<_, i64>(2)?))))?
        .collect::<Result<HashMap<String, (String, i64)>, _>>()?;
    drop(stmt);
    let mut out = Vec::new();
    for row in rows(conn)? {
        let Ok(m) = serde_json::from_str::<Manifest>(&row.manifest) else {
            continue;
        };
        for g in m.games {
            let inst = installed.get(&format!("{}:{}", row.id, g.id));
            out.push(SourceGame {
                source_id: row.id,
                source_name: row.name.clone(),
                host: safe::https_host(&g.url).unwrap_or("").to_string(),
                installed: inst.is_some(),
                installed_version: inst.map(|(v, _)| v.clone()),
                update: inst.map(|(v, _)| *v != g.version).unwrap_or(false),
                library_id: inst.map(|(_, id)| *id),
                verified: !g.sha256.is_empty(),
                id: g.id,
                name: g.name,
                version: g.version,
                description: g.description,
                cover: g.cover,
                category: g.category,
                license: g.license,
                size: g.size,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn catalog() -> String {
        format!(
            r#"{{"format":1,"name":"Teste","games":[{{"id":"jogo-1","name":"Jogo Um","version":"1.0","url":"https://e.com/a.zip","sha256":"{}","launch":"Jogo/jogo.exe"}}]}}"#,
            "a".repeat(64)
        )
    }

    #[test]
    fn accepts_a_valid_catalog() {
        let m = parse(&catalog(), "x").unwrap();
        assert_eq!(m.name, "Teste");
        assert_eq!(m.games[0].kind, "zip");
    }

    #[test]
    fn sha256_is_optional() {
        let sem = catalog().replace(&format!(",\"sha256\":\"{}\"", "a".repeat(64)), "");
        assert!(!sem.contains("sha256"));
        let m = parse(&sem, "x").unwrap();
        assert!(m.games[0].sha256.is_empty());
    }

    #[test]
    fn rejects_unsafe_entries() {
        assert!(parse(&catalog().replace("\"format\":1", "\"format\":2"), "x").is_err());
        assert!(parse(&catalog().replace("https://e.com/a.zip", "http://e.com/a.zip"), "x").is_err());
        assert!(parse(&catalog().replace("Jogo/jogo.exe", "../jogo.exe"), "x").is_err());
        assert!(parse(&catalog().replace(&"a".repeat(64), "123"), "x").is_err());
        assert!(parse(&catalog().replace("jogo-1", "Jogo 1"), "x").is_err());
        assert!(parse("não é json", "x").is_err());
    }

    #[test]
    fn save_list_find_remove() {
        let c = open_in_memory().unwrap();
        let m = parse(&catalog(), "x").unwrap();
        let s = save(&c, "https://e.com/catalogo.json", &m).unwrap();
        assert_eq!(s.game_count, 1);
        save(&c, "https://e.com/catalogo.json", &m).unwrap(); // mesma fonte: atualiza
        assert_eq!(list(&c).unwrap().len(), 1);
        assert_eq!(find_game(&c, s.id, "jogo-1").unwrap().name, "Jogo Um");
        assert!(find_game(&c, s.id, "outro").is_err());
        let all = all_games(&c).unwrap();
        assert!(!all[0].installed && !all[0].update);
        remove(&c, s.id).unwrap();
        assert!(list(&c).unwrap().is_empty());
    }
}
