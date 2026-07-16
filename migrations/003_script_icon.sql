PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS slots_new (
    page_id INTEGER NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    row INTEGER NOT NULL CHECK (row >= 0 AND row < 3),
    col INTEGER NOT NULL CHECK (col >= 0 AND col < 5),
    kind TEXT NOT NULL DEFAULT 'empty',
    label TEXT,
    image_path TEXT,
    sound_id INTEGER REFERENCES sounds(id) ON DELETE SET NULL,
    child_page_id INTEGER REFERENCES pages(id) ON DELETE SET NULL,
    slot_volume REAL NOT NULL DEFAULT 1.0,
    color_hex TEXT,
    script_command TEXT,
    PRIMARY KEY (page_id, row, col)
);

INSERT INTO slots_new (
    page_id, row, col, kind, label, image_path, sound_id, child_page_id, slot_volume, color_hex, script_command
)
SELECT page_id, row, col, kind, label, image_path, sound_id, child_page_id, slot_volume, color_hex, NULL
FROM slots;

DROP TABLE slots;

ALTER TABLE slots_new RENAME TO slots;

PRAGMA foreign_keys = ON;
