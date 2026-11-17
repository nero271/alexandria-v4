-- Fontes cadastradas pelo usuário (endereço de um catálogo JSON) e jogos instalados por elas.
CREATE TABLE sources (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    url       TEXT    NOT NULL UNIQUE,
    name      TEXT    NOT NULL,
    manifest  TEXT    NOT NULL,
    synced_at INTEGER NOT NULL
);

CREATE TABLE installs (
    key          TEXT    PRIMARY KEY,
    source_id    INTEGER NOT NULL,
    game_id      INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    version      TEXT    NOT NULL,
    dir          TEXT    NOT NULL,
    installed_at INTEGER NOT NULL
);
CREATE INDEX idx_installs_game ON installs (game_id);
