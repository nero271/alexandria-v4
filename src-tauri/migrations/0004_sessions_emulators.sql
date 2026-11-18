CREATE TABLE sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id    INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at INTEGER NOT NULL,
    minutes    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_sessions_started ON sessions (started_at);
CREATE INDEX idx_sessions_game    ON sessions (game_id);

CREATE TABLE emulators (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL,
    exe_path TEXT NOT NULL,
    args     TEXT NOT NULL DEFAULT '"{rom}"'
);
