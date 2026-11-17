use super::ExternalApp;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Manifest {
    display_name: Option<String>,
    app_name: Option<String>,
    catalog_namespace: Option<String>,
    catalog_item_id: Option<String>,
    install_location: Option<String>,
    launch_executable: Option<String>,
    #[serde(rename = "bIsIncompleteInstall", default)]
    incomplete: bool,
}

fn safe_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
}

pub fn parse_manifest(json: &str) -> Option<ExternalApp> {
    let m: Manifest = serde_json::from_str(json).ok()?;
    if m.incomplete {
        return None;
    }
    let name = m.display_name?.trim().to_string();
    let app = m.app_name?;
    if name.is_empty() || !safe_id(&app) {
        return None;
    }
    let target = match (m.catalog_namespace.as_deref(), m.catalog_item_id.as_deref()) {
        (Some(ns), Some(id)) if safe_id(ns) && safe_id(id) => Some(format!(
            "com.epicgames.launcher://apps/{ns}%3A{id}%3A{app}?action=launch&silent=true"
        )),
        _ => match (m.install_location, m.launch_executable) {
            (Some(dir), Some(exe)) if !exe.is_empty() => {
                Some(PathBuf::from(dir).join(exe).to_string_lossy().into_owned())
            }
            _ => None,
        },
    };
    Some(ExternalApp { app_id: app, name, launch_target: target, playtime_minutes: None })
}

fn manifests_dir() -> Option<PathBuf> {
    std::env::var_os("ProgramData").map(|p| {
        PathBuf::from(p)
            .join("Epic")
            .join("EpicGamesLauncher")
            .join("Data")
            .join("Manifests")
    })
}

/// `None` quando a Epic não está instalada.
pub fn scan() -> Option<Vec<ExternalApp>> {
    let dir = manifests_dir()?;
    if !dir.is_dir() {
        return None;
    }
    let mut apps: Vec<ExternalApp> = Vec::new();
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Some(app) = parse_manifest(&text) {
                if !apps.iter().any(|a| a.app_id == app.app_id) {
                    apps.push(app);
                }
            }
        }
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Some(apps)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: &str = r#"{
        "DisplayName": "Alan Wake 2", "AppName": "Pike",
        "CatalogNamespace": "abc123", "CatalogItemId": "def456",
        "InstallLocation": "D:\\Games\\AW2", "LaunchExecutable": "AW2.exe",
        "bIsIncompleteInstall": false
    }"#;

    #[test]
    fn builds_launch_uri() {
        let a = parse_manifest(ITEM).unwrap();
        assert_eq!(a.name, "Alan Wake 2");
        assert_eq!(
            a.launch_target.as_deref(),
            Some("com.epicgames.launcher://apps/abc123%3Adef456%3APike?action=launch&silent=true")
        );
    }

    #[test]
    fn rejects_incomplete_and_unsafe() {
        assert!(parse_manifest(&ITEM.replace("false", "true")).is_none());
        assert!(parse_manifest(&ITEM.replace("Pike", "a b;c")).is_none());
        assert!(parse_manifest("nada").is_none());
    }
}
