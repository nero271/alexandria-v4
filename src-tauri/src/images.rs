//! Cache de imagens em disco: cada URL é baixada uma única vez.
use crate::error::{AppError, AppResult};
use crate::net;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_BYTES: u64 = 15 * 1024 * 1024;
const EXTS: [&str; 5] = ["jpg", "jpeg", "png", "webp", "gif"];

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn ext_of(path: &str) -> &'static str {
    let clean = path.split(['?', '#']).next().unwrap_or("");
    let ext = clean.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    EXTS.iter().copied().find(|e| *e == ext).unwrap_or("jpg")
}

pub fn cache_dir(data: &Path) -> PathBuf {
    data.join("images")
}

pub fn download(data: &Path, url: &str) -> AppResult<PathBuf> {
    if !url.starts_with("https://") {
        return Err(AppError::Invalid("Endereço de imagem inválido.".into()));
    }
    let dir = cache_dir(data);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{:016x}.{}", fnv(url), ext_of(url)));
    if path.exists() {
        return Ok(path);
    }
    let resp = ureq::get(url)
        .timeout(Duration::from_secs(20))
        .call()
        .map_err(net::map_err)?;
    let mut buf = Vec::new();
    resp.into_reader().take(MAX_BYTES).read_to_end(&mut buf)?;
    if buf.is_empty() {
        return Err(AppError::Invalid("A imagem veio vazia.".into()));
    }
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, &buf)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Copia uma imagem escolhida pelo usuário para o cache do app.
pub fn import_local(data: &Path, src: &Path) -> AppResult<PathBuf> {
    let name = src.to_string_lossy();
    let ext = ext_of(&name);
    let ok = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false);
    if !ok || !src.is_file() {
        return Err(AppError::Invalid("Escolha um arquivo de imagem (png, jpg, webp ou gif).".into()));
    }
    let meta = std::fs::metadata(src)?;
    if meta.len() > MAX_BYTES {
        return Err(AppError::Invalid("A imagem é grande demais (limite de 15 MB).".into()));
    }
    let dir = cache_dir(data);
    std::fs::create_dir_all(&dir)?;
    let key = format!("{}|{}", name, meta.len());
    let dest = dir.join(format!("{:016x}.{}", fnv(&key), ext));
    std::fs::copy(src, &dest)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_detection() {
        assert_eq!(ext_of("https://x/y/z.PNG?v=1"), "png");
        assert_eq!(ext_of("https://x/y/z"), "jpg");
        assert_eq!(ext_of("https://x/y/z.exe"), "jpg");
    }

    #[test]
    fn rejects_non_https() {
        assert!(download(Path::new("."), "http://x/y.jpg").is_err());
    }
}
