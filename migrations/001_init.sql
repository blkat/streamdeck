PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER REFERENCES pages(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    volume_linear REAL NOT NULL DEFAULT 1.0,
    loudness_gain_db REAL NOT NULL DEFAULT 0.0,
    peak_db REAL,
    duration_ms INTEGER,
    source_kind TEXT NOT NULL DEFAULT 'import',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS slots (
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    row INTEGER NOT NULL CHECK (row >= 0 AND row < 3),
    col INTEGER NOT NULL CHECK (col >= 0 AND col < 5),
    kind TEXT NOT NULL DEFAULT 'empty' CHECK (kind IN ('sound', 'folder', 'empty')),
    label TEXT,
    image_path TEXT,
    sound_id INTEGER REFERENCES sounds(id) ON DELETE SET NULL,
    child_page_id INTEGER REFERENCES pages(id) ON DELETE SET NULL,
    slot_volume REAL NOT NULL DEFAULT 1.0,
    color_hex TEXT,
    PRIMARY KEY (page_id, row, col)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO pages (id, parent_id, name, sort_order) VALUES (1, NULL, 'Home', 0);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('global_volume', '0.7'),
    ('audio_policy', 'stop_previous'),
    ('max_channels', '3'),
    ('normalize_target_lufs', '-16'),
    ('capture_max_duration_ms', '60000'),
    ('capture_min_duration_ms', '300'),
    ('capture_input_device', ''),
    ('window_width', '900'),
    ('window_height', '700'),
    ('edit_mode', 'false'),
    ('clip_max_duration_s', '15');
