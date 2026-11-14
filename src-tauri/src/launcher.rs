//! Inicialização de jogos: protocolo da Steam ou executável local.
use crate::error::{AppError, AppResult};
use crate::library::model::Game;
use std::path::Path;

fn valid_steam_uri(uri: &str) -> bool {
    uri.strip_prefix("steam://")
        .map(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric() || "/_-".contains(c)))
        .unwrap_or(false)
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn open_uri(uri: &str) -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", uri])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(not(windows))]
fn open_uri(_uri: &str) -> AppResult<()> {
    Err(AppError::Invalid("Iniciar jogos só funciona no Windows.".into()))
}

pub fn launch(g: &Game) -> AppResult<()> {
    if g.platform == "steam" {
        let id = g
            .platform_id
            .as_deref()
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .ok_or_else(|| AppError::Invalid("Este jogo da Steam não tem um AppID válido.".into()))?;
        return open_uri(&format!("steam://rungameid/{id}"));
    }
    let target = g
        .launch_target
        .as_deref()
        .ok_or_else(|| AppError::Invalid("Defina o executável do jogo antes de iniciar.".into()))?;
    if target.starts_with("steam://") {
        if !valid_steam_uri(target) {
            return Err(AppError::Invalid("Endereço steam:// inválido.".into()));
        }
        return open_uri(target);
    }
    let path = Path::new(target);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !path.is_file() {
        return Err(AppError::Invalid(format!("Executável não encontrado: {target}")));
    }
    if ext != "exe" && ext != "bat" {
        return Err(AppError::Invalid("Escolha um arquivo .exe ou .bat.".into()));
    }
    let mut cmd = std::process::Command::new(path);
    if let Some(dir) = path.parent() {
        cmd.current_dir(dir);
    }
    cmd.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_uri_validation() {
        assert!(valid_steam_uri("steam://rungameid/620"));
        assert!(!valid_steam_uri("steam://x y"));
        assert!(!valid_steam_uri("http://a"));
        assert!(!valid_steam_uri("steam://"));
    }
}
