-- Migration to set up initial tables for CiteSync

-- CREATE TABLE publishers (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     name TEXT NOT NULL UNIQUE
-- );

CREATE TABLE items (
    id TEXT PRIMARY KEY UNIQUE NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    publisher TEXT,
    kind TEXT NOT NULL
);

CREATE TABLE authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL
);

CREATE TABLE author_item (
    author_id INTEGER NOT NULL,
    item_id TEXT NOT NULL,
    PRIMARY KEY (author_id, item_id),
    FOREIGN KEY (author_id) REFERENCES authors (id),
    FOREIGN KEY (item_id) REFERENCES items (id)
);

CREATE TABLE files (
    id TEXT PRIMARY KEY NOT NULL,
    mime_type TEXT NOT NULL,
    kind TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    FOREIGN KEY (item_id) REFERENCES items (id)
);
