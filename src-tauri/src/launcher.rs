//! Inicialização de jogos: endereços das lojas ou executável local.
use crate::error::{AppError, AppResult};
use crate::library::model::Game;
use std::path::Path;
use std::process::Child;

const URI_PREFIXES: [&str; 2] = ["steam://", "com.epicgames.launcher://apps/"];

fn valid_uri(uri: &str) -> bool {
    let rest = URI_PREFIXES.iter().find_map(|p| uri.strip_prefix(p));
    match rest {
        Some(r) => {
            !r.is_empty()
                && uri.len() < 300
                && uri.chars().all(|c| c.is_ascii_alphanumeric() || "/_-%:?=&.".contains(c))
        }
        None => false,
    }
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub(crate) fn open_uri(uri: &str) -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", uri])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn open_uri(_uri: &str) -> AppResult<()> {
    Err(AppError::Invalid("Esta ação só funciona no Windows.".into()))
}

/// Só abre páginas do GitHub (usado pela verificação de atualização).
pub fn open_external(url: &str) -> AppResult<()> {
    let ok = url.starts_with("https://github.com/")
        && url.len() < 300
        && url.chars().all(|c| c.is_ascii_alphanumeric() || "/_-.%:?=&#~".contains(c));
    if !ok {
        return Err(AppError::Invalid("Endereço não permitido.".into()));
    }
    open_uri(url)
}

/// Devolve o processo quando o jogo é aberto diretamente (permite contar o tempo de jogo).
pub fn launch(g: &Game) -> AppResult<Option<Child>> {
    if g.platform == "steam" {
        let id = g
            .platform_id
            .as_deref()
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .ok_or_else(|| AppError::Invalid("Este jogo da Steam não tem um AppID válido.".into()))?;
        open_uri(&format!("steam://rungameid/{id}"))?;
        return Ok(None);
    }
    let target = g
        .launch_target
        .as_deref()
        .ok_or_else(|| AppError::Invalid("Defina o executável do jogo antes de iniciar.".into()))?;
    if URI_PREFIXES.iter().any(|p| target.starts_with(p)) {
        if !valid_uri(target) {
            return Err(AppError::Invalid("Endereço de inicialização inválido.".into()));
        }
        open_uri(target)?;
        return Ok(None);
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
    Ok(Some(cmd.spawn()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_validation() {
        assert!(valid_uri("steam://rungameid/620"));
        assert!(valid_uri("com.epicgames.launcher://apps/a%3Ab%3Ac?action=launch&silent=true"));
        assert!(!valid_uri("steam://x y"));
        assert!(!valid_uri("http://a"));
        assert!(!valid_uri("steam://"));
        assert!(!valid_uri("com.epicgames.launcher://other/x"));
    }

    #[test]
    fn external_only_github() {
        assert!(open_external("https://evil.com/x").is_err());
        assert!(open_external("https://github.com/a b").is_err());
    }
}
