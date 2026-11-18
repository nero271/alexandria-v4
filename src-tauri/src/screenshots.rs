//! Capturas de tela da Steam: userdata/<conta>/760/remote/<AppID>/screenshots.
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAX_SHOTS: usize = 24;

/// Devolve (arquivos mais novos primeiro, pastas onde foram achados).
pub fn find(steam_dir: &Path, app_id: &str) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut files: Vec<(SystemTime, PathBuf)> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) {
        return (vec![], dirs);
    }
    let Ok(users) = std::fs::read_dir(steam_dir.join("userdata")) else {
        return (vec![], dirs);
    };
    for user in users.flatten() {
        let dir = user.path().join("760").join("remote").join(app_id).join("screenshots");
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut any = false;
        for e in entries.flatten() {
            let p = e.path();
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
            if p.is_file() && matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
                let t = e.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                files.push((t, p));
                any = true;
            }
        }
        if any {
            dirs.push(dir);
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.truncate(MAX_SHOTS);
    (files.into_iter().map(|(_, p)| p).collect(), dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_only_images_for_the_app() {
        let root = std::env::temp_dir().join("alexandria_shots_test");
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("userdata").join("1").join("760").join("remote").join("620").join("screenshots");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.jpg"), b"x").unwrap();
        std::fs::write(dir.join("b.txt"), b"x").unwrap();
        let (files, dirs) = find(&root, "620");
        assert_eq!(files.len(), 1);
        assert_eq!(dirs.len(), 1);
        assert!(find(&root, "999").0.is_empty());
        assert!(find(&root, "../x").0.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
