use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;

fn invalid(msg: &str) -> AppError {
    AppError::Invalid(msg.into())
}

fn is_hex_color(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Só chaves conhecidas e valores válidos entram no banco.
pub fn validate(key: &str, value: &str) -> AppResult<()> {
    match key {
        "app_name" if value.chars().count() <= 40 => Ok(()),
        "app_name" => Err(invalid("O nome do app deve ter até 40 caracteres.")),
        "accent" if value.is_empty() || is_hex_color(value) => Ok(()),
        "accent" => Err(invalid("Cor inválida.")),
        "animations" if matches!(value, "on" | "off") => Ok(()),
        "animations" => Err(invalid("Valor inválido para animações.")),
        "mode" if matches!(value, "desktop" | "fullscreen") => Ok(()),
        "mode" => Err(invalid("Modo inválido.")),
        "scrim" if value.parse::<u8>().map(|n| n <= 100).unwrap_or(false) => Ok(()),
        "scrim" => Err(invalid("O ajuste de transparência deve ficar entre 0 e 100.")),
        "sgdb_key" if value.len() <= 200 => Ok(()),
        "sgdb_key" => Err(invalid("Chave grande demais.")),
        "igdb_id" if value.len() <= 100 => Ok(()),
        "igdb_id" => Err(invalid("Client ID grande demais.")),
        "igdb_secret" if value.len() <= 200 => Ok(()),
        "igdb_secret" => Err(invalid("Client Secret grande demais.")),
        "app_bg" => Ok(()),
        _ => Err(invalid("Configuração desconhecida.")),
    }
}

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

/// Valor vazio apaga a configuração.
pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    let value = value.trim();
    validate(key, value)?;
    if value.is_empty() {
        conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
    } else {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
    }
    Ok(())
}

pub fn all(conn: &Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn validates_values() {
        assert!(validate("accent", "#cbb98d").is_ok());
        assert!(validate("accent", "red").is_err());
        assert!(validate("scrim", "101").is_err());
        assert!(validate("scrim", "60").is_ok());
        assert!(validate("animations", "maybe").is_err());
        assert!(validate("whatever", "x").is_err());
    }

    #[test]
    fn set_get_and_clear() {
        let c = open_in_memory().unwrap();
        set(&c, "app_name", "Minha Biblioteca").unwrap();
        assert_eq!(get(&c, "app_name").unwrap().as_deref(), Some("Minha Biblioteca"));
        set(&c, "app_name", "").unwrap();
        assert_eq!(get(&c, "app_name").unwrap(), None);
        assert!(set(&c, "mode", "tv").is_err());
    }
}
