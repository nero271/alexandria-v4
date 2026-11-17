//! Emuladores cadastrados pelo usuário: o jogo guarda o arquivo (ROM) e o emulador o abre.
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Emulator {
    pub id: i64,
    pub name: String,
    pub exe_path: String,
    pub args: String,
}

fn invalid(m: &str) -> AppError {
    AppError::Invalid(m.into())
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<Emulator> {
    Ok(Emulator { id: r.get(0)?, name: r.get(1)?, exe_path: r.get(2)?, args: r.get(3)? })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Emulator>> {
    let mut stmt = conn.prepare("SELECT id, name, exe_path, args FROM emulators ORDER BY name COLLATE NOCASE")?;
    let v = stmt.query_map([], row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(v)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Emulator> {
    conn.query_row("SELECT id, name, exe_path, args FROM emulators WHERE id = ?1", [id], row)
        .optional()?
        .ok_or_else(|| invalid("Emulador não encontrado. Cadastre-o de novo nos ajustes."))
}

pub fn validate(name: &str, exe: &str, args: &str, check_file: bool) -> AppResult<()> {
    if name.trim().is_empty() || name.chars().count() > 60 {
        return Err(invalid("Dê ao emulador um nome de até 60 caracteres."));
    }
    let ext = Path::new(exe).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext != "exe" && ext != "bat" {
        return Err(invalid("Escolha o programa do emulador (.exe ou .bat)."));
    }
    if check_file && !Path::new(exe).is_file() {
        return Err(invalid("O arquivo do emulador não foi encontrado."));
    }
    if !args.contains("{rom}") || args.len() > 300 {
        return Err(invalid("O modelo de argumentos precisa conter {rom}, por exemplo: \"{rom}\"."));
    }
    Ok(())
}

pub fn add(conn: &Connection, name: &str, exe: &str, args: &str) -> AppResult<Emulator> {
    let args = if args.trim().is_empty() { "\"{rom}\"" } else { args.trim() };
    validate(name, exe, args, true)?;
    conn.execute(
        "INSERT INTO emulators (name, exe_path, args) VALUES (?1, ?2, ?3)",
        params![name.trim(), exe, args],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM emulators WHERE id = ?1", [id])?;
    Ok(())
}

/// Separa o modelo de argumentos respeitando aspas e troca {rom} pelo caminho do jogo.
pub fn split_args(template: &str, rom: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut has = false;
    for c in template.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                has = true;
            }
            ' ' if !quoted => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            _ => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    out.into_iter().map(|a| a.replace("{rom}", rom)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn splits_arguments() {
        assert_eq!(split_args("\"{rom}\"", "C:\\a b\\g.iso"), vec!["C:\\a b\\g.iso"]);
        assert_eq!(split_args("-f --full \"{rom}\"", "r"), vec!["-f", "--full", "r"]);
        assert_eq!(split_args("--rom={rom}", "x"), vec!["--rom=x"]);
    }

    #[test]
    fn validation_rules() {
        assert!(validate("Dolphin", "C:\\d\\Dolphin.exe", "\"{rom}\"", false).is_ok());
        assert!(validate("", "C:\\d\\Dolphin.exe", "{rom}", false).is_err());
        assert!(validate("X", "C:\\d\\readme.txt", "{rom}", false).is_err());
        assert!(validate("X", "C:\\d\\x.exe", "sem-marcador", false).is_err());
    }

    #[test]
    fn missing_emulator_is_an_error() {
        let c = open_in_memory().unwrap();
        assert!(get(&c, 42).is_err());
        assert!(list(&c).unwrap().is_empty());
    }
}
