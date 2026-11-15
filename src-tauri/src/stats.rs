//! Resumo do ano: horas, jogos mais jogados e sequências de dias, a partir das sessões registradas.
use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopGame {
    pub id: i64,
    pub name: String,
    pub minutes: i64,
    pub sessions: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearStats {
    pub year: i32,
    pub years: Vec<i32>,
    pub sessions: i64,
    pub minutes: i64,
    pub days: i64,
    pub longest_streak: i64,
    pub current_streak: i64,
    pub top: Vec<TopGame>,
    pub lifetime_minutes: i64,
    pub games_total: i64,
}

/// Dias desde 1970-01-01 (algoritmo de Howard Hinnant).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn year_of_day(z: i64) -> i64 {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 { y + 1 } else { y }
}

fn local_day(ts: i64, offset_min: i64) -> i64 {
    (ts + offset_min * 60).div_euclid(86_400)
}

/// (maior sequência, sequência atual). A atual só conta se o último dia foi hoje ou ontem.
pub fn streaks(days: &BTreeSet<i64>, today: i64) -> (i64, i64) {
    let mut longest = 0;
    let mut run = 0;
    let mut prev: Option<i64> = None;
    for &d in days {
        run = if prev == Some(d - 1) { run + 1 } else { 1 };
        longest = longest.max(run);
        prev = Some(d);
    }
    let mut current = 0;
    let mut d = if days.contains(&today) { today } else { today - 1 };
    while days.contains(&d) {
        current += 1;
        d -= 1;
    }
    (longest, current)
}

pub fn year_stats(conn: &Connection, year: i32, offset_min: i64, now_ts: i64) -> AppResult<YearStats> {
    let mut stmt = conn.prepare("SELECT started_at FROM sessions")?;
    let all: Vec<i64> = stmt.query_map([], |r| r.get::<_, i64>(0))?.collect::<Result<Vec<i64>, _>>()?;
    drop(stmt);

    let today = local_day(now_ts, offset_min);
    let mut years: BTreeSet<i32> = all.iter().map(|&t| year_of_day(local_day(t, offset_min)) as i32).collect();
    years.insert(year_of_day(today) as i32);
    let all_days: BTreeSet<i64> = all.iter().map(|&t| local_day(t, offset_min)).collect();
    let year_days: BTreeSet<i64> = all_days.iter().copied().filter(|&d| year_of_day(d) == year as i64).collect();
    let (longest, _) = streaks(&year_days, today);
    let (_, current) = streaks(&all_days, today);

    let start = days_from_civil(year as i64, 1, 1) * 86_400 - offset_min * 60;
    let end = days_from_civil(year as i64 + 1, 1, 1) * 86_400 - offset_min * 60;
    let (sessions, minutes): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(minutes), 0) FROM sessions WHERE started_at >= ?1 AND started_at < ?2",
        params![start, end],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let mut top_stmt = conn.prepare(
        "SELECT g.id, g.name, COALESCE(SUM(s.minutes), 0) AS m, COUNT(*) AS c \
         FROM sessions s JOIN games g ON g.id = s.game_id \
         WHERE s.started_at >= ?1 AND s.started_at < ?2 \
         GROUP BY g.id ORDER BY m DESC, c DESC LIMIT 5",
    )?;
    let top = top_stmt
        .query_map(params![start, end], |r| {
            Ok(TopGame { id: r.get(0)?, name: r.get(1)?, minutes: r.get(2)?, sessions: r.get(3)? })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(top_stmt);
    let lifetime_minutes: i64 = conn.query_row("SELECT COALESCE(SUM(playtime_minutes), 0) FROM games", [], |r| r.get(0))?;
    let games_total: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))?;
    Ok(YearStats {
        year,
        years: years.into_iter().rev().collect(),
        sessions,
        minutes,
        days: year_days.len() as i64,
        longest_streak: longest,
        current_streak: current,
        top,
        lifetime_minutes,
        games_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn civil_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2024, 3, 1), 19_783);
        assert_eq!(year_of_day(19_783), 2024);
        assert_eq!(year_of_day(days_from_civil(2023, 12, 31)), 2023);
        assert_eq!(year_of_day(days_from_civil(2024, 1, 1)), 2024);
    }

    #[test]
    fn streak_logic() {
        let days: BTreeSet<i64> = [1, 2, 3, 5, 6].into_iter().collect();
        assert_eq!(streaks(&days, 6), (3, 2));
        assert_eq!(streaks(&days, 7), (3, 2));
        assert_eq!(streaks(&days, 9), (3, 0));
        assert_eq!(streaks(&BTreeSet::new(), 5), (0, 0));
    }

    #[test]
    fn year_summary() {
        let c = open_in_memory().unwrap();
        c.execute("INSERT INTO games (name, created_at, updated_at) VALUES ('Hades', 0, 0)", []).unwrap();
        let day = 19_783 * 86_400; // 2024-03-01 (UTC)
        for (offset, mins) in [(0, 60), (1, 30), (2, 90), (10, 15)] {
            c.execute(
                "INSERT INTO sessions (game_id, started_at, minutes) VALUES (1, ?1, ?2)",
                params![day + offset * 86_400 + 3600, mins],
            )
            .unwrap();
        }
        let s = year_stats(&c, 2024, 0, day + 11 * 86_400).unwrap();
        assert_eq!((s.sessions, s.minutes, s.days), (4, 195, 4));
        assert_eq!(s.longest_streak, 3);
        assert_eq!(s.top[0].name, "Hades");
        assert!(s.years.contains(&2024));
    }
}
