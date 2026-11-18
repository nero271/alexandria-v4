CREATE TABLE wishlist (
    igdb_id  INTEGER PRIMARY KEY,
    name     TEXT    NOT NULL,
    cover    TEXT,
    year     INTEGER,
    url      TEXT,
    added_at INTEGER NOT NULL
);
