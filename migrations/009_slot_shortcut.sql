ALTER TABLE slots ADD COLUMN shortcut_key TEXT;
INSERT OR IGNORE INTO settings (key, value) VALUES ('grid_shortcuts_enabled', 'true');
