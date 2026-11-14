//! Detecção da Steam e leitura dos jogos instalados (arquivos .acf e libraryfolders.vdf).
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SteamApp {
    pub app_id: String,
    pub name: String,
    /// Minutos jogados, lidos do arquivo local da própria Steam.
    pub playtime: Option<i64>,
}

#[derive(Debug, PartialEq)]
enum Tok {
    Str(String),
    Open,
    Close,
}

fn tokenize(text: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '"' => {
                let mut s = String::new();
                while let Some(ch) = it.next() {
                    match ch {
                        '"' => break,
                        '\\' => match it.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                            None => break,
                        },
                        _ => s.push(ch),
                    }
                }
                out.push(Tok::Str(s));
            }
            '{' => out.push(Tok::Open),
            '}' => out.push(Tok::Close),
            '/' if it.peek() == Some(&'/') => {
                for ch in it.by_ref() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Primeiro valor associado à chave (sem diferenciar maiúsculas).
fn value_of(toks: &[Tok], key: &str) -> Option<String> {
    toks.windows(2).find_map(|w| match (&w[0], &w[1]) {
        (Tok::Str(k), Tok::Str(v)) if k.eq_ignore_ascii_case(key) => Some(v.clone()),
        _ => None,
    })
}

pub fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    let toks = tokenize(vdf);
    toks.windows(2)
        .filter_map(|w| match (&w[0], &w[1]) {
            (Tok::Str(k), Tok::Str(v)) if k.eq_ignore_ascii_case("path") => Some(PathBuf::from(v)),
            _ => None,
        })
        .collect()
}

/// Ferramentas da Steam que aparecem como "jogos" mas não são.
const IGNORED_APP_IDS: [&str; 2] = ["228980", "250820"];

pub fn parse_manifest(acf: &str) -> Option<SteamApp> {
    let toks = tokenize(acf);
    let app_id = value_of(&toks, "appid")?;
    let name = value_of(&toks, "name")?;
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) || name.trim().is_empty() {
        return None;
    }
    if IGNORED_APP_IDS.contains(&app_id.as_str()) {
        return None;
    }
    // StateFlags: bit 4 = totalmente instalado.
    if let Some(flags) = value_of(&toks, "StateFlags").and_then(|f| f.parse::<u32>().ok()) {
        if flags & 4 == 0 {
            return None;
        }
    }
    Some(SteamApp { app_id, name, playtime: None })
}

/// Lê "Playtime" (minutos) de cada AppID no localconfig.vdf.
pub fn parse_playtimes(vdf: &str) -> HashMap<String, i64> {
    let toks = tokenize(vdf);
    let mut stack: Vec<String> = Vec::new();
    let mut out: HashMap<String, i64> = HashMap::new();
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            Tok::Str(k) => match toks.get(i + 1) {
                Some(Tok::Open) => {
                    stack.push(k.clone());
                    i += 2;
                    continue;
                }
                Some(Tok::Str(v)) => {
                    if k.eq_ignore_ascii_case("playtime") {
                        if let Some(app) = stack.last() {
                            if !app.is_empty() && app.chars().all(|c| c.is_ascii_digit()) {
                                if let Ok(n) = v.parse::<i64>() {
                                    let e = out.entry(app.clone()).or_insert(0);
                                    if n > *e {
                                        *e = n;
                                    }
                                }
                            }
                        }
                    }
                    i += 2;
                    continue;
                }
                _ => {}
            },
            Tok::Close => {
                stack.pop();
            }
            Tok::Open => {}
        }
        i += 1;
    }
    out
}

fn playtimes(steam_dir: &Path) -> HashMap<String, i64> {
    let mut all: HashMap<String, i64> = HashMap::new();
    let Ok(users) = std::fs::read_dir(steam_dir.join("userdata")) else {
        return all;
    };
    for user in users.flatten() {
        let file = user.path().join("config").join("localconfig.vdf");
        if let Ok(text) = std::fs::read_to_string(file) {
            for (app, n) in parse_playtimes(&text) {
                let e = all.entry(app).or_insert(0);
                if n > *e {
                    *e = n;
                }
            }
        }
    }
    all
}

pub fn find_steam_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey("Software\\Valve\\Steam") {
            if let Ok(p) = key.get_value::<String, _>("SteamPath") {
                let p = PathBuf::from(p);
                if p.join("steamapps").is_dir() {
                    return Some(p);
                }
            }
        }
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(base) = std::env::var_os(var) {
            let p = PathBuf::from(base).join("Steam");
            if p.join("steamapps").is_dir() {
                return Some(p);
            }
        }
    }
    None
}

pub fn scan_installed(steam_dir: &Path) -> Vec<SteamApp> {
    let mut libs = vec![steam_dir.to_path_buf()];
    if let Ok(text) = std::fs::read_to_string(steam_dir.join("steamapps").join("libraryfolders.vdf")) {
        for p in parse_library_paths(&text) {
            if !libs.contains(&p) {
                libs.push(p);
            }
        }
    }
    let times = playtimes(steam_dir);
    let mut apps: Vec<SteamApp> = Vec::new();
    for lib in libs {
        let Ok(entries) = std::fs::read_dir(lib.join("steamapps")) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(entry.path()) {
                if let Some(mut app) = parse_manifest(&text) {
                    app.playtime = times.get(&app.app_id).copied();
                    if !apps.iter().any(|a| a.app_id == app.app_id) {
                        apps.push(app);
                    }
                }
            }
        }
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIBS: &str = r#"
"libraryfolders"
{
    "0" { "path"  "C:\\Program Files (x86)\\Steam"  "apps" { "620" "123" } }
    "1" { "path"  "D:\\SteamLibrary" }
}"#;

    const ACF: &str = r#"
"AppState"
{
    "appid"  "1145360"
    "name"   "Hades"
    "StateFlags"  "4"
    "installdir"  "Hades"
}"#;

    #[test]
    fn reads_library_paths() {
        let p = parse_library_paths(LIBS);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0], PathBuf::from("C:\\Program Files (x86)\\Steam"));
        assert_eq!(p[1], PathBuf::from("D:\\SteamLibrary"));
    }

    #[test]
    fn reads_manifest() {
        let a = parse_manifest(ACF).unwrap();
        assert_eq!((a.app_id.as_str(), a.name.as_str()), ("1145360", "Hades"));
    }

    #[test]
    fn reads_playtimes() {
        let vdf = r#"
"UserLocalConfigStore"
{
    "Software" { "Valve" { "Steam" { "apps" {
        "620" { "LastPlayed" "1700000000" "Playtime" "125" }
        "1145360" { "cloud" { "Playtime" "7" } "Playtime" "300" }
        "notanapp" { "Playtime" "9" }
    } } } }
}"#;
        let p = parse_playtimes(vdf);
        assert_eq!(p.get("620"), Some(&125));
        assert_eq!(p.get("1145360"), Some(&300));
        assert!(!p.contains_key("notanapp"));
    }

    #[test]
    fn skips_incomplete_downloads_and_tools() {
        assert!(parse_manifest(&ACF.replace("\"4\"", "\"1026\"")).is_none());
        assert!(parse_manifest(&ACF.replace("1145360", "228980")).is_none());
        assert!(parse_manifest("lixo").is_none());
    }
}
