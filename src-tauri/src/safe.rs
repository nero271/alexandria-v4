//! Verificações pequenas e testáveis (só `std`) usadas pela Loja.

/// Servidor (com porta) de um endereço https. `None` se não for https ou tiver usuário@.
pub fn authority(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let a = &rest[..end];
    if a.is_empty() || a.contains('@') {
        None
    } else {
        Some(a)
    }
}

pub fn https_host(url: &str) -> Option<&str> {
    let h = authority(url)?.split(':').next().unwrap_or("");
    if h.is_empty() {
        None
    } else {
        Some(h)
    }
}

/// Endereço https completo, sem espaços nem caracteres de controle.
pub fn is_https_url(url: &str) -> bool {
    url.len() <= 500 && !url.chars().any(|c| c.is_whitespace() || c.is_control()) && https_host(url).is_some()
}

/// 64 caracteres hexadecimais (um SHA-256 em texto).
pub fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn hex(bytes: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(D[(b >> 4) as usize] as char);
        out.push(D[(b & 15) as usize] as char);
    }
    out
}

/// Caminho relativo seguro: sem raiz, sem unidade (C:), sem "..", sem partes vazias.
pub fn safe_rel_path(p: &str) -> bool {
    let p = p.trim();
    if p.is_empty() || p.len() > 260 || p.starts_with('/') || p.starts_with('\\') || p.contains(':') {
        return false;
    }
    if p.chars().any(|c| c.is_control()) {
        return false;
    }
    p.split(['/', '\\']).all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// Identificador no catálogo: minúsculas, números, ponto, hífen e sublinhado.
pub fn valid_id(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b.len() <= 64
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(*c, b'.' | b'_' | b'-'))
}

/// "bytes 100-199/1000" -> Some(1000)
pub fn content_range_total(h: &str) -> Option<u64> {
    h.rsplit('/').next()?.trim().parse().ok()
}

pub fn has_ext(path: &str, exts: &[&str]) -> bool {
    let lower = path.to_ascii_lowercase();
    exts.iter().any(|e| lower.ends_with(&format!(".{e}")))
}

const STORE_HOSTS: [&str; 10] = [
    "store.steampowered.com",
    "www.gog.com",
    "www.epicgames.com",
    "store.epicgames.com",
    "www.xbox.com",
    "www.igdb.com",
    "store.ubisoft.com",
    "www.ea.com",
    "www.battle.net",
    "github.com",
];

/// Páginas que o programa aceita abrir no navegador (lojas conhecidas, IGDB e GitHub).
pub fn store_link_ok(url: &str) -> bool {
    if !is_https_url(url) || url.len() > 300 {
        return false;
    }
    let Some(h) = https_host(url) else {
        return false;
    };
    let h = h.to_ascii_lowercase();
    STORE_HOSTS.contains(&h.as_str()) || h == "itch.io" || h.ends_with(".itch.io")
}

/// Ordem de preferência ao escolher qual loja abrir (menor = melhor).
pub fn store_link_rank(url: &str) -> u8 {
    match https_host(url).map(|h| h.to_ascii_lowercase()).as_deref() {
        Some("store.steampowered.com") => 0,
        Some("www.gog.com") => 1,
        Some("store.epicgames.com") | Some("www.epicgames.com") => 2,
        Some("www.igdb.com") => 9,
        _ => 5,
    }
}

/// 1536000000 -> "1,4 GB" (para mensagens de erro).
pub fn fmt_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i >= 3 { format!("{v:.1} {}", units[i]) } else { format!("{v:.0} {}", units[i]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_addresses() {
        assert_eq!(https_host("https://exemplo.com/a/b"), Some("exemplo.com"));
        assert_eq!(https_host("https://exemplo.com:8443/a"), Some("exemplo.com"));
        assert_eq!(https_host("http://exemplo.com"), None);
        assert_eq!(https_host("https://user@exemplo.com/"), None);
        assert_eq!(https_host("https:///x"), None);
        assert!(is_https_url("https://exemplo.com/a.json"));
        assert!(!is_https_url("https://exemplo.com/a b"));
        assert!(!is_https_url(&format!("https://e.com/{}", "a".repeat(600))));
    }

    #[test]
    fn store_links() {
        assert!(store_link_ok("https://store.steampowered.com/app/620/Portal_2/"));
        assert!(store_link_ok("https://autor.itch.io/jogo"));
        assert!(store_link_ok("https://github.com/a/b"));
        assert!(!store_link_ok("https://store.steampowered.com.evil.com/x"));
        assert!(!store_link_ok("https://evil.com/steam"));
        assert!(!store_link_ok("http://store.steampowered.com/app/1"));
        assert!(store_link_rank("https://store.steampowered.com/app/1") < store_link_rank("https://www.gog.com/game/x"));
    }

    #[test]
    fn hashes_and_hex() {
        assert!(is_sha256(&"a".repeat(64)));
        assert!(!is_sha256(&"a".repeat(63)));
        assert!(!is_sha256(&"g".repeat(64)));
        assert_eq!(hex(&[0x00, 0xff, 0x1a]), "00ff1a");
    }

    #[test]
    fn relative_paths() {
        assert!(safe_rel_path("Jogo/jogo.exe"));
        assert!(safe_rel_path("a\\b\\c.bat"));
        assert!(!safe_rel_path("../jogo.exe"));
        assert!(!safe_rel_path("a/../../b.exe"));
        assert!(!safe_rel_path("/abs.exe"));
        assert!(!safe_rel_path("\\\\srv\\x.exe"));
        assert!(!safe_rel_path("C:\\x.exe"));
        assert!(!safe_rel_path("a//b.exe"));
        assert!(!safe_rel_path(""));
    }

    #[test]
    fn ids_ranges_sizes() {
        assert!(valid_id("meu-jogo_2.0"));
        assert!(!valid_id("Meu"));
        assert!(!valid_id("-x"));
        assert!(!valid_id("a b"));
        assert_eq!(content_range_total("bytes 100-199/1000"), Some(1000));
        assert_eq!(content_range_total("bytes 0-9/*"), None);
        assert!(has_ext("A/B.EXE", &["exe", "bat"]));
        assert!(!has_ext("a.exe.txt", &["exe"]));
        assert_eq!(fmt_bytes(1536 * 1024 * 1024), "1.5 GB");
    }
}
