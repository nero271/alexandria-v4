use super::ExternalApp;

/// Jogos do GOG Galaxy ficam registrados no Registro do Windows.
#[cfg(windows)]
pub fn scan() -> Option<Vec<ExternalApp>> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let games = hklm.open_subkey("SOFTWARE\\WOW6432Node\\GOG.com\\Games").ok()?;
    let mut out: Vec<ExternalApp> = Vec::new();
    for key_name in games.enum_keys().flatten() {
        let Ok(k) = games.open_subkey(&key_name) else {
            continue;
        };
        let title: String = k.get_value("gameName").unwrap_or_default();
        let exe: String = k.get_value("exe").unwrap_or_default();
        let id: String = k.get_value("gameID").unwrap_or_else(|_| key_name.clone());
        if title.trim().is_empty() || id.is_empty() {
            continue;
        }
        out.push(ExternalApp {
            app_id: id,
            name: title.trim().to_string(),
            launch_target: if exe.is_empty() { None } else { Some(exe) },
            playtime_minutes: None,
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Some(out)
}

#[cfg(not(windows))]
pub fn scan() -> Option<Vec<ExternalApp>> {
    None
}
