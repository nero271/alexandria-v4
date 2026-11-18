CREATE TABLE games (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    name             TEXT    NOT NULL,
    platform         TEXT    NOT NULL DEFAULT 'manual',
    platform_id      TEXT,
    launch_target    TEXT,
    category         TEXT,
    install_state    TEXT    NOT NULL DEFAULT 'unknown'
                     CHECK (install_state IN ('installed','not_installed','unknown')),
    favorite         INTEGER NOT NULL DEFAULT 0 CHECK (favorite IN (0,1)),
    last_played_at   INTEGER,
    description      TEXT,
    cover_path       TEXT,
    hero_path        TEXT,
    background_path  TEXT,
    logo_path        TEXT,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL
);

-- Um mesmo jogo de uma plataforma não pode ser cadastrado duas vezes.
CREATE UNIQUE INDEX idx_games_platform_id
    ON games (platform, platform_id) WHERE platform_id IS NOT NULL;
CREATE INDEX idx_games_name     ON games (name COLLATE NOCASE);
CREATE INDEX idx_games_favorite ON games (favorite) WHERE favorite = 1;
CREATE INDEX idx_games_recent   ON games (last_played_at DESC) WHERE last_played_at IS NOT NULL;

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
