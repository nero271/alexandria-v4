//! Acompanha o jogo iniciado: avisa a interface, registra a sessão e atualiza o Discord.
use crate::commands::AppState;
use crate::library::repository as repo;
use crate::{discord, settings};
use serde::Serialize;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const STEAM_POLL: Duration = Duration::from_millis(1500);
const STEAM_START_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Serialize)]
struct GameState {
    id: i64,
    state: &'static str,
}

/// Estados: "tracked" (o fim do jogo será informado), "running" (a Steam confirmou que o jogo abriu),
/// "exited", "timeout" (a Steam não confirmou a tempo) e "untracked" (não dá para acompanhar).
pub fn emit_state(app: &AppHandle, id: i64, state: &'static str) {
    let _ = app.emit("game-state", GameState { id, state });
}

/// Application ID do Discord, se a opção estiver ligada.
fn rpc_config(app: &AppHandle) -> Option<String> {
    let st = app.try_state::<AppState>()?;
    let (on, id) = st
        .with_db(|c| Ok((settings::get(c, "discord_rpc")?, settings::get(c, "discord_app_id")?)))
        .ok()?;
    if on.as_deref() == Some("on") { id } else { None }
}

fn rpc_on(app: &AppHandle, name: &str) {
    if let Some(id) = rpc_config(app) {
        let name = name.to_string();
        let ts = repo::now();
        thread::spawn(move || discord::set_activity(&id, &name, "Jogando no ALEXANDRIA", ts));
    }
}

fn rpc_off(app: &AppHandle) {
    if let Some(id) = rpc_config(app) {
        thread::spawn(move || discord::clear(&id));
    }
}

fn close_session(app: &AppHandle, session_id: i64, minutes: i64) {
    if let Some(st) = app.try_state::<AppState>() {
        let _ = st.with_db(|c| repo::end_session(c, session_id, minutes));
    }
}

/// Jogo aberto direto por executável: espera o processo terminar e soma o tempo jogado.
pub fn watch_child(app: AppHandle, id: i64, name: String, session_id: i64, mut child: Child) {
    thread::spawn(move || {
        emit_state(&app, id, "tracked");
        rpc_on(&app, &name);
        let start = Instant::now();
        let _ = child.wait();
        let minutes = (start.elapsed().as_secs() / 60) as i64;
        if minutes > 0 {
            if let Some(st) = app.try_state::<AppState>() {
                let _ = st.with_db(|c| repo::add_playtime(c, id, minutes));
            }
        }
        close_session(&app, session_id, minutes);
        rpc_off(&app);
        emit_state(&app, id, "exited");
    });
}

#[derive(Debug, PartialEq)]
enum Step {
    Started,
    Exited,
    TimedOut,
    Wait,
}

fn step(seen: bool, current: u32, app_id: u32, waited: Duration) -> Step {
    if current == app_id {
        if seen {
            Step::Wait
        } else {
            Step::Started
        }
    } else if seen {
        Step::Exited
    } else if waited > STEAM_START_TIMEOUT {
        Step::TimedOut
    } else {
        Step::Wait
    }
}

/// A Steam grava no Registro qual jogo está rodando (RunningAppID; 0 = nenhum).
#[cfg(windows)]
fn running_app_id() -> u32 {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("RunningAppID").ok())
        .unwrap_or(0)
}

#[cfg(not(windows))]
fn running_app_id() -> u32 {
    0
}

/// Jogo da Steam: consulta o Registro a cada 1,5 s, só enquanto o jogo estiver abrindo ou rodando.
pub fn watch_steam(app: AppHandle, id: i64, name: String, session_id: i64, app_id: u32) {
    thread::spawn(move || {
        emit_state(&app, id, "tracked");
        let start = Instant::now();
        let mut running_since: Option<Instant> = None;
        loop {
            thread::sleep(STEAM_POLL);
            match step(running_since.is_some(), running_app_id(), app_id, start.elapsed()) {
                Step::Started => {
                    running_since = Some(Instant::now());
                    rpc_on(&app, &name);
                    emit_state(&app, id, "running");
                }
                Step::Exited => {
                    let minutes = running_since.map(|t| (t.elapsed().as_secs() / 60) as i64).unwrap_or(0);
                    close_session(&app, session_id, minutes);
                    rpc_off(&app);
                    emit_state(&app, id, "exited");
                    break;
                }
                Step::TimedOut => {
                    emit_state(&app, id, "timeout");
                    break;
                }
                Step::Wait => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_state_machine() {
        let t0 = Duration::from_secs(1);
        assert_eq!(step(false, 0, 620, t0), Step::Wait);
        assert_eq!(step(false, 620, 620, t0), Step::Started);
        assert_eq!(step(true, 620, 620, t0), Step::Wait);
        assert_eq!(step(true, 0, 620, t0), Step::Exited);
        assert_eq!(step(false, 0, 620, Duration::from_secs(121)), Step::TimedOut);
        assert_eq!(step(false, 730, 620, t0), Step::Wait);
    }
}
