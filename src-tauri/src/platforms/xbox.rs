use super::ExternalApp;
use std::path::{Path, PathBuf};

fn pick_exe(content: &Path) -> Option<PathBuf> {
    let helper = content.join("gamelaunchhelper.exe");
    if helper.is_file() {
        return Some(helper);
    }
    let mut exes: Vec<PathBuf> = std::fs::read_dir(content)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("exe")).unwrap_or(false))
        .collect();
    exes.sort();
    exes.into_iter().next()
}

/// Lê uma pasta "XboxGames": cada subpasta é um jogo, com o programa em "Content".
pub fn scan_dir(root: &Path) -> Vec<ExternalApp> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if !p.is_dir() || name.is_empty() {
            continue;
        }
        if let Some(exe) = pick_exe(&p.join("Content")) {
            out.push(ExternalApp {
                app_id: name.clone(),
                name,
                launch_target: Some(exe.to_string_lossy().into_owned()),
                playtime_minutes: None,
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Procura "XboxGames" na raiz de cada disco (jogos do Xbox app e do Game Pass para PC).
#[cfg(windows)]
pub fn scan() -> Option<Vec<ExternalApp>> {
    let mut found = false;
    let mut out: Vec<ExternalApp> = Vec::new();
    for letter in b'C'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\XboxGames", letter as char));
        if root.is_dir() {
            found = true;
            out.extend(scan_dir(&root));
        }
    }
    if found { Some(out) } else { None }
}

#[cfg(not(windows))]
pub fn scan() -> Option<Vec<ExternalApp>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_xbox_games_folder() {
        let root = std::env::temp_dir().join("alexandria_xbox_test");
        let _ = std::fs::remove_dir_all(&root);
        let content = root.join("Forza Horizon 5").join("Content");
        std::fs::create_dir_all(&content).unwrap();
        std::fs::write(content.join("gamelaunchhelper.exe"), b"x").unwrap();
        std::fs::create_dir_all(root.join("Vazio")).unwrap();
        let apps = scan_dir(&root);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Forza Horizon 5");
        let _ = std::fs::remove_dir_all(&root);
    }
}
