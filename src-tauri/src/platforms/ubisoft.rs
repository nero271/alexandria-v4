use super::ExternalApp;

/// Pega o nome da pasta de instalação (o Ubisoft Connect não guarda o título no Registro).
pub fn title_from_dir(dir: &str) -> String {
    dir.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(windows)]
pub fn scan() -> Option<Vec<ExternalApp>> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let installs = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs").ok()?;
    let mut out: Vec<ExternalApp> = Vec::new();
    for id in installs.enum_keys().flatten() {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let Ok(k) = installs.open_subkey(&id) else {
            continue;
        };
        let dir: String = k.get_value("InstallDir").unwrap_or_default();
        let name = title_from_dir(&dir);
        if name.is_empty() || !std::path::Path::new(&dir).is_dir() {
            continue;
        }
        out.push(ExternalApp {
            launch_target: Some(format!("uplay://launch/{id}/0")),
            app_id: id,
            name,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_from_install_dir() {
        assert_eq!(title_from_dir("C:/Program Files (x86)/Ubisoft/games/Far Cry 6/"), "Far Cry 6");
        assert_eq!(title_from_dir("D:\\Games\\Anno 1800"), "Anno 1800");
        assert_eq!(title_from_dir(""), "");
    }
}
