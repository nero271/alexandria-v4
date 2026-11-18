//! Discord Rich Presence pelo canal local (named pipe), sem bibliotecas extras.
//! Precisa do "Application ID" de um aplicativo criado pelo próprio usuário no Discord.
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Mutex;

static CONN: Mutex<Option<(String, File)>> = Mutex::new(None);

fn frame(op: u32, body: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + body.len());
    v.extend_from_slice(&op.to_le_bytes());
    v.extend_from_slice(&(body.len() as u32).to_le_bytes());
    v.extend_from_slice(body.as_bytes());
    v
}

#[cfg(windows)]
fn open_pipe() -> Option<File> {
    (0..10).find_map(|i| {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!(r"\\.\pipe\discord-ipc-{i}"))
            .ok()
    })
}

#[cfg(not(windows))]
fn open_pipe() -> Option<File> {
    let _ = OpenOptions::new();
    None
}

fn read_frame(f: &mut File) -> std::io::Result<String> {
    let mut head = [0u8; 8];
    f.read_exact(&mut head)?;
    let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
    if len > 65_536 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "quadro grande demais"));
    }
    let mut body = vec![0u8; len];
    f.read_exact(&mut body)?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

fn connect(app_id: &str) -> Option<File> {
    let mut f = open_pipe()?;
    let hello = json!({ "v": 1, "client_id": app_id }).to_string();
    f.write_all(&frame(0, &hello)).ok()?;
    read_frame(&mut f).ok()?;
    Some(f)
}

fn clip(s: &str) -> String {
    let t: String = s.chars().take(120).collect();
    if t.chars().count() < 2 { format!("{t} ") } else { t }
}

fn send(app_id: &str, activity: serde_json::Value) {
    let Ok(mut guard) = CONN.lock() else {
        return;
    };
    if guard.as_ref().map(|(id, _)| id.as_str() != app_id).unwrap_or(true) {
        *guard = connect(app_id).map(|f| (app_id.to_string(), f));
    }
    let Some((_, file)) = guard.as_mut() else {
        return;
    };
    let msg = json!({
        "cmd": "SET_ACTIVITY",
        "args": { "pid": std::process::id(), "activity": activity },
        "nonce": format!("{}", crate::library::repository::now())
    })
    .to_string();
    let ok = file.write_all(&frame(1, &msg)).is_ok() && read_frame(file).is_ok();
    if !ok {
        *guard = None; // Discord fechou: reconecta na próxima vez
    }
}

/// Mostra "Jogando X" no perfil. Se o Discord não estiver aberto, não faz nada.
pub fn set_activity(app_id: &str, details: &str, state: &str, start_ts: i64) {
    send(
        app_id,
        json!({ "details": clip(details), "state": clip(state), "timestamps": { "start": start_ts } }),
    );
}

pub fn clear(app_id: &str) {
    send(app_id, serde_json::Value::Null);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_layout() {
        assert_eq!(frame(1, "{}"), vec![1, 0, 0, 0, 2, 0, 0, 0, b'{', b'}']);
    }

    #[test]
    fn clip_keeps_text_valid() {
        assert_eq!(clip("a").chars().count(), 2);
        assert_eq!(clip(&"x".repeat(300)).chars().count(), 120);
    }
}
