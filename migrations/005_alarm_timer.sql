ALTER TABLE slots ADD COLUMN alarm_mode TEXT NOT NULL DEFAULT 'clock';
ALTER TABLE slots ADD COLUMN alarm_minutes INTEGER;
ALTER TABLE slots ADD COLUMN alarm_armed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE slots ADD COLUMN alarm_armed_at_ms INTEGER;

INSERT OR IGNORE INTO settings (key, value) VALUES ('alarm_preset_minutes', '5,10,15,30');
