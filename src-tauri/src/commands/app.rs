use super::join_err;
use crate::error::{AppError, AppResult};
use crate::{launcher, net};
use serde::Serialize;
use tauri::AppHandle;

const UPDATE_REPO: &str = "nero271/alexandria-v4";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: Option<String>,
    pub newer: bool,
    pub url: Option<String>,
}

fn parse_version(s: &str) -> Vec<u64> {
    s.trim()
        .trim_start_matches('v')
        .split('.')
        .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0))
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let (mut a, mut b) = (parse_version(latest), parse_version(current));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

/// Consulta a última versão publicada no GitHub (só funciona se o repositório for público).
#[tauri::command]
pub async fn check_update(app: AppHandle) -> AppResult<UpdateInfo> {
    let current = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || -> AppResult<UpdateInfo> {
        let url = format!("https://api.github.com/repos/{UPDATE_REPO}/releases/latest");
        let v = net::get_json_headers(&url, &[("User-Agent", "ALEXANDRIA"), ("Accept", "application/vnd.github+json")])
            .map_err(|_| {
                AppError::Invalid(
                    "Não foi possível verificar: o repositório é privado, ainda não há versão publicada ou você está sem internet."
                        .into(),
                )
            })?;
        let tag = v.get("tag_name").and_then(|t| t.as_str()).map(String::from);
        let page = v.get("html_url").and_then(|t| t.as_str()).map(String::from);
        let newer = tag.as_deref().map(|t| is_newer(t, &current)).unwrap_or(false);
        Ok(UpdateInfo { current, latest: tag, newer, url: page })
    })
    .await
    .map_err(join_err)?
}

#[tauri::command]
pub fn open_external(url: String) -> AppResult<()> {
    launcher::open_external(&url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(is_newer("v0.6.0", "0.5.1"));
        assert!(is_newer("0.5.10", "0.5.9"));
        assert!(!is_newer("v0.5.1", "0.5.1"));
        assert!(!is_newer("0.4", "0.5.1"));
    }
}
