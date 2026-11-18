//! Baixar e instalar jogos de uma fonte: progresso em tempo real, pausa e retomada,
//! conferência do SHA-256, extração segura do .zip e registro na biblioteca.
use crate::error::{AppError, AppResult};
use crate::library::model::{Game, NewGame};
use crate::library::repository::{self as repo, now};
use crate::sources::ManifestGame;
use crate::{images, net, safe};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Plataforma gravada na biblioteca para os jogos instalados pela Loja.
pub const PLATFORM: &str = "fonte";
pub const PAUSED_MSG: &str = "Download pausado. Toque em Baixar para continuar de onde parou.";

const MAX_UNPACKED: u64 = 100 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 200_000;

#[derive(Clone, Serialize)]
struct Progress {
    key: String,
    phase: &'static str,
    done: u64,
    total: u64,
    speed: f64,
}

/// Avisa a interface do andamento (no máximo 4 vezes por segundo), com a velocidade do download.
struct Reporter<'a> {
    app: &'a AppHandle,
    key: &'a str,
    last_at: Option<Instant>,
    last_done: u64,
    speed: f64,
}

impl<'a> Reporter<'a> {
    fn new(app: &'a AppHandle, key: &'a str) -> Self {
        Reporter { app, key, last_at: None, last_done: 0, speed: 0.0 }
    }

    fn emit(&mut self, phase: &'static str, done: u64, total: u64, force: bool) {
        let due = self.last_at.map_or(true, |t| t.elapsed() >= Duration::from_millis(250));
        if !(force || due) {
            return;
        }
        if let Some(t) = self.last_at {
            let dt = t.elapsed().as_secs_f64();
            if dt > 0.0 && done >= self.last_done {
                let inst = (done - self.last_done) as f64 / dt;
                self.speed = if self.speed == 0.0 { inst } else { 0.7 * self.speed + 0.3 * inst };
            }
        }
        self.last_at = Some(Instant::now());
        self.last_done = done;
        let speed = if phase == "download" { self.speed } else { 0.0 };
        let _ = self.app.emit("install-progress", Progress { key: self.key.to_string(), phase, done, total, speed });
    }
}

fn invalid(msg: &str) -> AppError {
    AppError::Invalid(msg.to_string())
}

fn paused() -> AppError {
    invalid(PAUSED_MSG)
}

/// Garante espaço livre suficiente (com uma folga) antes de gravar `need` bytes.
fn check_space(near: &Path, need: u64) -> AppResult<()> {
    if need == 0 {
        return Ok(());
    }
    let dir = near.parent().unwrap_or_else(|| Path::new("."));
    if let Ok(free) = fs2::available_space(dir) {
        let wanted = need + need / 20 + 50 * 1024 * 1024;
        if free < wanted {
            return Err(AppError::Invalid(format!(
                "Espaço insuficiente no disco: são necessários cerca de {} e há {} livres.",
                safe::fmt_bytes(wanted),
                safe::fmt_bytes(free)
            )));
        }
    }
    Ok(())
}

/// Baixa para `part`. Se já existe um pedaço, continua dali; se o servidor não aceitar, recomeça.
fn download(url: &str, part: &Path, cancel: &AtomicBool, rep: &mut Reporter) -> AppResult<u64> {
    let agent = net::https_agent();
    for attempt in 0..2 {
        let have = fs::metadata(part).map(|m| m.len()).unwrap_or(0);
        let resp = net::get_https(&agent, url, if have > 0 { Some(have) } else { None })?;
        if resp.status() == 416 {
            if attempt == 0 && have > 0 {
                let _ = fs::remove_file(part); // o pedaço guardado não combina com o servidor
                continue;
            }
            return Err(invalid("O servidor recusou continuar o download."));
        }
        let resumed = resp.status() == 206 && have > 0;
        let total = if resumed {
            resp.header("Content-Range").and_then(safe::content_range_total).unwrap_or(0)
        } else {
            resp.header("Content-Length").and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0)
        };
        let remaining = total.saturating_sub(if resumed { have } else { 0 });
        check_space(part, remaining)?;
        let mut done = if resumed { have } else { 0 };
        let mut out = if resumed { OpenOptions::new().append(true).open(part)? } else { File::create(part)? };
        let mut reader = resp.into_reader();
        let mut buf = vec![0u8; 256 * 1024];
        rep.emit("download", done, total, true);
        loop {
            if cancel.load(Ordering::Relaxed) {
                out.flush()?;
                return Err(paused());
            }
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    let _ = out.flush();
                    return Err(invalid("A conexão caiu durante o download. Toque em Baixar para continuar de onde parou."));
                }
            };
            out.write_all(&buf[..n])?;
            done += n as u64;
            rep.emit("download", done, total, false);
        }
        out.flush()?;
        if total > 0 && done != total {
            return Err(invalid("O download terminou antes do fim do arquivo. Toque em Baixar para continuar."));
        }
        rep.emit("download", done, total.max(done), true);
        return Ok(done);
    }
    Err(invalid("Não foi possível baixar o arquivo."))
}

fn sha256_file(path: &Path, cancel: &AtomicBool, rep: &mut Reporter) -> AppResult<String> {
    let total = fs::metadata(path)?.len();
    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    let mut done = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(paused());
        }
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        done += n as u64;
        rep.emit("verify", done, total, false);
    }
    rep.emit("verify", total, total, true);
    Ok(safe::hex(hasher.finalize().as_slice()))
}

fn zip_err(_: zip::result::ZipError) -> AppError {
    invalid("Não foi possível ler o .zip (arquivo danificado ou compactação não suportada; use um ZIP comum).")
}

/// Extrai o .zip para `dest`, recusando qualquer caminho que tente sair da pasta.
fn extract_zip(zip_path: &Path, dest: &Path, cancel: &AtomicBool, rep: &mut Reporter) -> AppResult<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path)?).map_err(zip_err)?;
    if archive.len() > MAX_ENTRIES {
        return Err(invalid("O .zip tem arquivos demais."));
    }
    let mut declared: u64 = 0;
    for i in 0..archive.len() {
        declared = declared.saturating_add(archive.by_index(i).map_err(zip_err)?.size());
    }
    if declared > MAX_UNPACKED {
        return Err(invalid("O conteúdo do .zip é grande demais."));
    }
    check_space(dest, declared)?;
    let mut written = 0u64;
    let mut buf = vec![0u8; 256 * 1024];
    rep.emit("extract", 0, declared, true);
    for i in 0..archive.len() {
        if cancel.load(Ordering::Relaxed) {
            return Err(paused());
        }
        let mut entry = archive.by_index(i).map_err(zip_err)?;
        let Some(rel) = entry.enclosed_name() else {
            return Err(invalid("O .zip contém um caminho inseguro e foi recusado."));
        };
        let target = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&target)?;
        let mut left = entry.size();
        while left > 0 {
            if cancel.load(Ordering::Relaxed) {
                return Err(paused());
            }
            let want = left.min(buf.len() as u64) as usize;
            let n = entry.read(&mut buf[..want])?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
            left -= n as u64;
            written += n as u64;
            rep.emit("extract", written, declared, false);
        }
    }
    rep.emit("extract", declared, declared, true);
    Ok(())
}

fn launch_rel(def: &ManifestGame) -> String {
    match def.launch.as_deref() {
        Some(l) => l.to_string(),
        None => format!("{}.exe", def.id),
    }
}

pub struct Installed {
    pub launch: PathBuf,
    pub dir: PathBuf,
    pub cover: Option<String>,
}

/// Baixa, confere e instala. Roda numa thread própria (bloqueia até terminar).
pub fn install(
    app: &AppHandle,
    data: &Path,
    source_id: i64,
    def: &ManifestGame,
    key: &str,
    cancel: &AtomicBool,
) -> AppResult<Installed> {
    let mut rep = Reporter::new(app, key);
    let downloads = data.join("downloads");
    let games = data.join("games");
    fs::create_dir_all(&downloads)?;
    fs::create_dir_all(&games)?;
    let stem = format!("{source_id}-{}", def.id);
    let part = downloads.join(format!("{stem}.part"));
    let dest = games.join(&stem);
    let staging = games.join(format!("{stem}.novo"));
    let launch = launch_rel(def);
    if !safe::safe_rel_path(&launch) {
        return Err(invalid("O caminho do executável no catálogo é inseguro."));
    }

    // 1. Baixar (continua de onde parou).
    download(&def.url, &part, cancel, &mut rep)?;

    // 2. Conferir (quando o catálogo informa o SHA-256): o arquivo precisa ser o que ele promete.
    if !def.sha256.is_empty() {
        let got = sha256_file(&part, cancel, &mut rep)?;
        if !got.eq_ignore_ascii_case(&def.sha256) {
            let _ = fs::remove_file(&part);
            return Err(invalid(
                "O arquivo baixado não confere com o SHA-256 do catálogo e foi descartado. \
                 A fonte pode estar desatualizada ou o arquivo foi alterado.",
            ));
        }
    }

    // 3. Montar numa pasta temporária e só então trocar pela pasta final.
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    let unpacked = if def.kind == "zip" {
        extract_zip(&part, &staging, cancel, &mut rep)
    } else {
        let target = staging.join(&launch);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&part, &target).map(|_| ()).map_err(AppError::from)
    };
    if let Err(e) = unpacked {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }
    if !staging.join(&launch).is_file() {
        let _ = fs::remove_dir_all(&staging);
        return Err(invalid("O executável indicado em \"launch\" não existe dentro do pacote."));
    }
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    fs::rename(&staging, &dest)?;
    let _ = fs::remove_file(&part);
    rep.emit("done", 1, 1, true);

    // A capa é um extra: se falhar, o jogo é instalado do mesmo jeito.
    let cover = def.cover.as_deref().and_then(|u| images::download(data, u).ok()).map(|p| p.to_string_lossy().into_owned());
    Ok(Installed { launch: dest.join(&launch), dir: dest, cover })
}

/// Registra (ou atualiza) o jogo na biblioteca e o vínculo com a fonte.
pub fn register(conn: &Connection, source_id: i64, def: &ManifestGame, done: &Installed) -> AppResult<Game> {
    let key = format!("{source_id}:{}", def.id);
    let launch = done.launch.to_string_lossy().into_owned();
    let existing: Option<i64> = conn
        .query_row("SELECT game_id FROM installs WHERE key = ?1", params![key], |r| r.get(0))
        .optional()?;
    let game_id = match existing.filter(|id| repo::get(conn, *id).is_ok()) {
        Some(id) => {
            conn.execute(
                "UPDATE games SET launch_target = ?1, updated_at = ?2 WHERE id = ?3",
                params![launch, now(), id],
            )?;
            id
        }
        None => match repo::add(
            conn,
            NewGame {
                name: def.name.clone(),
                platform: Some(PLATFORM.to_string()),
                platform_id: Some(key.clone()),
                launch_target: Some(launch.clone()),
                category: def.category.clone(),
                description: def.description.clone(),
            },
        ) {
            Ok(g) => g.id,
            Err(_) => conn.query_row(
                "SELECT id FROM games WHERE platform = ?1 AND platform_id = ?2",
                params![PLATFORM, key],
                |r| r.get(0),
            )?,
        },
    };
    conn.execute("UPDATE games SET install_state = 'installed' WHERE id = ?1", params![game_id])?;
    if let Some(c) = &done.cover {
        conn.execute("UPDATE games SET cover_path = ?1 WHERE id = ?2 AND cover_path IS NULL", params![c, game_id])?;
    }
    conn.execute(
        "INSERT INTO installs (key, source_id, game_id, version, dir, installed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(key) DO UPDATE SET game_id = excluded.game_id, version = excluded.version, \
         dir = excluded.dir, installed_at = excluded.installed_at",
        params![key, source_id, game_id, def.version, done.dir.to_string_lossy().into_owned(), now()],
    )?;
    repo::get(conn, game_id)
}

/// (id do jogo na biblioteca, pasta instalada) de um jogo instalado pela Loja.
pub fn uninstall_plan(conn: &Connection, source_id: i64, game_id: &str) -> AppResult<(i64, PathBuf)> {
    let key = format!("{source_id}:{game_id}");
    conn.query_row("SELECT game_id, dir FROM installs WHERE key = ?1", params![key], |r| {
        Ok((r.get::<_, i64>(0)?, PathBuf::from(r.get::<_, String>(1)?)))
    })
    .optional()?
    .ok_or_else(|| invalid("Este jogo não está instalado."))
}

/// Apaga a pasta do jogo, mas só se estiver dentro da pasta de jogos do app.
pub fn delete_dir(data: &Path, dir: &Path) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    let root = data.join("games").canonicalize()?;
    let target = dir.canonicalize()?;
    if target == root || !target.starts_with(&root) {
        return Err(invalid("Por segurança, só é possível apagar pastas dentro da pasta de jogos do ALEXANDRIA."));
    }
    fs::remove_dir_all(&target)?;
    Ok(())
}

/// Tira o jogo da biblioteca (o vínculo da instalação some junto).
pub fn uninstall_finish(conn: &Connection, library_id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM games WHERE id = ?1", params![library_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn def(version: &str) -> ManifestGame {
        ManifestGame {
            id: "jogo-1".into(),
            name: "Jogo Um".into(),
            version: version.into(),
            description: Some("Texto".into()),
            cover: None,
            category: Some("Indie".into()),
            license: None,
            size: None,
            url: "https://e.com/a.zip".into(),
            sha256: "a".repeat(64),
            kind: "zip".into(),
            launch: Some("Jogo/jogo.exe".into()),
        }
    }

    #[test]
    fn launch_defaults() {
        let mut d = def("1");
        assert_eq!(launch_rel(&d), "Jogo/jogo.exe");
        d.launch = None;
        assert_eq!(launch_rel(&d), "jogo-1.exe");
    }

    #[test]
    fn register_update_and_uninstall() {
        let c = open_in_memory().unwrap();
        let done = Installed { launch: PathBuf::from("C:/g/jogo.exe"), dir: PathBuf::from("C:/g"), cover: None };
        let g = register(&c, 1, &def("1.0"), &done).unwrap();
        assert_eq!(g.platform, PLATFORM);
        assert_eq!(g.launch_target.as_deref(), Some("C:/g/jogo.exe"));
        // Reinstalar/atualizar reaproveita o mesmo jogo da biblioteca.
        let again = register(&c, 1, &def("2.0"), &done).unwrap();
        assert_eq!(again.id, g.id);
        let (lib_id, dir) = uninstall_plan(&c, 1, "jogo-1").unwrap();
        assert_eq!((lib_id, dir), (g.id, PathBuf::from("C:/g")));
        uninstall_finish(&c, lib_id).unwrap();
        assert!(uninstall_plan(&c, 1, "jogo-1").is_err());
        assert!(repo::get(&c, g.id).is_err());
    }

    #[test]
    fn only_deletes_inside_the_games_folder() {
        let base = std::env::temp_dir().join("alexandria_dl_test");
        let _ = fs::remove_dir_all(&base);
        let inside = base.join("games").join("1-jogo");
        let outside = base.join("fora");
        fs::create_dir_all(&inside).unwrap();
        fs::create_dir_all(&outside).unwrap();
        assert!(delete_dir(&base, &outside).is_err());
        assert!(delete_dir(&base, &base.join("games")).is_err());
        assert!(outside.exists());
        delete_dir(&base, &inside).unwrap();
        assert!(!inside.exists());
        let _ = fs::remove_dir_all(&base);
    }
}
