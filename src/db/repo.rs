use super::models::{AudioPolicy, Page, Slot, SlotKind, Sound};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct DbRepository {
    conn: Connection,
}

impl DbRepository {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path).context("open database")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
    }

    pub fn migrate(&self, sql: &str) -> Result<()> {
        self.conn.execute_batch(sql).context("run migrations")?;
        Ok(())
    }

    pub fn init_schema_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    pub fn is_migration_applied(&self, version: i64) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            params![version],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn mark_migration_applied(&self, version: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
            params![version],
        )?;
        Ok(())
    }

    pub fn table_exists(&self, name: &str) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    /// Bases créées avant le suivi des migrations : marquer 001 comme déjà appliquée.
    pub fn bootstrap_legacy_migrations(&self) -> Result<()> {
        if self.table_exists("pages")? && !self.is_migration_applied(1)? {
            self.mark_migration_applied(1)?;
        }
        if self.table_exists("slots")? {
            let has_color: bool = self.conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'color_hex'",
                [],
                |r| {
                    let c: i64 = r.get(0)?;
                    Ok(c > 0)
                },
            )?;
            if has_color && !self.is_migration_applied(2)? {
                self.mark_migration_applied(2)?;
            }
            let has_script = self.conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'script_command'",
                [],
                |r| {
                    let c: i64 = r.get(0)?;
                    Ok(c > 0)
                },
            )?;
            if has_script && !self.is_migration_applied(3)? {
                self.mark_migration_applied(3)?;
            }
            let has_alarm = self.conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'alarm_time'",
                [],
                |r| {
                    let c: i64 = r.get(0)?;
                    Ok(c > 0)
                },
            )?;
            if has_alarm && !self.is_migration_applied(4)? {
                self.mark_migration_applied(4)?;
            }
            let has_alarm_mode = self.conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'alarm_mode'",
                [],
                |r| {
                    let c: i64 = r.get(0)?;
                    Ok(c > 0)
                },
            )?;
            if has_alarm_mode && !self.is_migration_applied(5)? {
                self.mark_migration_applied(5)?;
            }
        }
        Ok(())
    }

    pub fn slots_have_alarm_mode(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'alarm_mode'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_alarm_time(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'alarm_time'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_script_command(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'script_command'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_color_hex(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'color_hex'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_slot_appearance(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'slot_appearance'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_script_shell(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'script_shell'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn slots_have_shortcut_key(&self) -> Result<bool> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('slots') WHERE name = 'shortcut_key'",
            [],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn ensure_root_page(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM pages", [], |r| r.get(0))?;
        if count == 0 {
            self.conn.execute(
                "INSERT INTO pages (id, parent_id, name, sort_order) VALUES (1, NULL, ?1, 0)",
                rusqlite::params![crate::i18n::root_page_name()],
            )?;
        }
        Ok(())
    }

    pub fn root_page_id(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM pages WHERE parent_id IS NULL ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .context("root page")?)
    }

    pub fn get_page(&self, id: i64) -> Result<Page> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, parent_id, name, sort_order FROM pages WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Page {
                        id: r.get(0)?,
                        parent_id: r.get(1)?,
                        name: r.get(2)?,
                        sort_order: r.get(3)?,
                    })
                },
            )?)
    }

    pub fn update_page_name(&self, page_id: i64, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE pages SET name = ?1 WHERE id = ?2",
            params![name, page_id],
        )?;
        Ok(())
    }

    pub fn create_child_page(&self, parent_id: i64, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO pages (parent_id, name, sort_order) VALUES (?1, ?2, 0)",
            params![parent_id, name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn map_slot(r: &rusqlite::Row<'_>) -> rusqlite::Result<Slot> {
        Ok(Slot {
            page_id: r.get(0)?,
            row: r.get(1)?,
            col: r.get(2)?,
            kind: SlotKind::from_str(&r.get::<_, String>(3)?),
            label: r.get(4)?,
            image_path: r.get(5)?,
            sound_id: r.get(6)?,
            child_page_id: r.get(7)?,
            slot_volume: r.get(8)?,
            color_hex: r.get(9)?,
            script_command: r.get(10)?,
            script_shell: r.get(11).ok(),
            alarm_time: r.get(12)?,
            alarm_mode: r.get::<_, String>(13).unwrap_or_else(|_| "clock".to_string()),
            alarm_minutes: r.get(14).ok(),
            alarm_armed: r.get::<_, i32>(15).unwrap_or(0) != 0,
            alarm_armed_at_ms: r.get(16).ok(),
            slot_appearance: r.get::<_, String>(17).unwrap_or_else(|_| "color".to_string()),
            shortcut_key: r.get(18).ok(),
        })
    }

    const SLOT_SELECT: &'static str = "SELECT page_id, row, col, kind, label, image_path, sound_id, child_page_id, slot_volume, color_hex, script_command, script_shell, alarm_time, alarm_mode, alarm_minutes, alarm_armed, alarm_armed_at_ms, slot_appearance, shortcut_key";

    pub fn list_slots(&self, page_id: i64) -> Result<Vec<Slot>> {
        let mut stmt = self.conn.prepare(&format!(
            "{} FROM slots WHERE page_id = ?1",
            Self::SLOT_SELECT
        ))?;
        let rows = stmt.query_map(params![page_id], Self::map_slot)?;
        Ok(rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("list slots")?)
    }

    pub fn list_alarm_slots(&self) -> Result<Vec<Slot>> {
        let mut stmt = self.conn.prepare(&format!(
            "{} FROM slots WHERE kind = 'alarm' AND alarm_armed = 1 AND (
                (COALESCE(alarm_mode, 'clock') = 'clock' AND alarm_time IS NOT NULL AND trim(alarm_time) != '')
                OR (alarm_mode = 'timer' AND alarm_armed_at_ms IS NOT NULL AND alarm_minutes IS NOT NULL)
            )",
            Self::SLOT_SELECT
        ))?;
        let rows = stmt.query_map([], Self::map_slot)?;
        Ok(rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("list alarm slots")?)
    }

    pub fn upsert_slot(&self, slot: &Slot) -> Result<()> {
        self.conn.execute(
            "INSERT INTO slots (page_id, row, col, kind, label, image_path, sound_id, child_page_id, slot_volume, color_hex, script_command, script_shell, alarm_time, alarm_mode, alarm_minutes, alarm_armed, alarm_armed_at_ms, slot_appearance, shortcut_key)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)
             ON CONFLICT(page_id, row, col) DO UPDATE SET
               kind=excluded.kind, label=excluded.label, image_path=excluded.image_path,
               sound_id=excluded.sound_id, child_page_id=excluded.child_page_id,
               slot_volume=excluded.slot_volume, color_hex=excluded.color_hex,
               script_command=excluded.script_command,
               script_shell=excluded.script_shell,
               alarm_time=excluded.alarm_time,
               alarm_mode=excluded.alarm_mode,
               alarm_minutes=excluded.alarm_minutes,
               alarm_armed=excluded.alarm_armed,
               alarm_armed_at_ms=excluded.alarm_armed_at_ms,
               slot_appearance=excluded.slot_appearance,
               shortcut_key=excluded.shortcut_key",
            params![
                slot.page_id,
                slot.row,
                slot.col,
                slot.kind.as_str(),
                slot.label,
                slot.image_path,
                slot.sound_id,
                slot.child_page_id,
                slot.slot_volume,
                slot.color_hex,
                slot.script_command,
                slot.script_shell,
                slot.alarm_time,
                slot.alarm_mode,
                slot.alarm_minutes,
                if slot.alarm_armed { 1 } else { 0 },
                slot.alarm_armed_at_ms,
                slot.slot_appearance,
                slot.shortcut_key,
            ],
        )?;
        Ok(())
    }

    /// Autre slot sur la même page utilisant déjà ce raccourci (comparaison sur clé effective).
    pub fn find_slot_at_shortcut_key(
        &self,
        page_id: i64,
        exclude_row: i32,
        exclude_col: i32,
        key_code: &str,
    ) -> Result<Option<(i32, i32)>> {
        let slots = self.list_slots(page_id)?;
        for slot in &slots {
            if slot.row == exclude_row && slot.col == exclude_col {
                continue;
            }
            let effective = crate::shortcuts::effective_key_code(Some(slot), slot.row, slot.col);
            if effective == key_code {
                return Ok(Some((slot.row, slot.col)));
            }
        }
        Ok(None)
    }

    /// Raccourcis minuteur alarme (minutes) — fixes ; autre valeur via le champ libre de l'éditeur.
    pub fn alarm_preset_minutes(&self) -> Result<Vec<i32>> {
        Ok(vec![5, 10, 15, 30])
    }

    pub fn get_sound(&self, id: i64) -> Result<Sound> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, title, file_path, volume_linear, loudness_gain_db, peak_db, duration_ms, source_kind
             FROM sounds WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Sound {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        file_path: r.get(2)?,
                        volume_linear: r.get(3)?,
                        loudness_gain_db: r.get(4)?,
                        peak_db: r.get(5)?,
                        duration_ms: r.get(6)?,
                        source_kind: r.get(7)?,
                    })
                },
            )?)
    }

    pub fn list_sounds(&self) -> Result<Vec<Sound>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, file_path, volume_linear, loudness_gain_db, peak_db, duration_ms, source_kind
             FROM sounds ORDER BY title",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Sound {
                id: r.get(0)?,
                title: r.get(1)?,
                file_path: r.get(2)?,
                volume_linear: r.get(3)?,
                loudness_gain_db: r.get(4)?,
                peak_db: r.get(5)?,
                duration_ms: r.get(6)?,
                source_kind: r.get(7)?,
            })
        })?;
        Ok(rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("list sounds")?)
    }

    pub fn insert_sound(&self, sound: &Sound) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO sounds (title, file_path, volume_linear, loudness_gain_db, peak_db, duration_ms, source_kind)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                sound.title,
                sound.file_path,
                sound.volume_linear,
                sound.loudness_gain_db,
                sound.peak_db,
                sound.duration_ms,
                sound.source_kind,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_sound(&self, sound: &Sound) -> Result<()> {
        self.conn.execute(
            "UPDATE sounds SET title=?2, file_path=?3, volume_linear=?4, loudness_gain_db=?5,
             peak_db=?6, duration_ms=?7, source_kind=?8 WHERE id=?1",
            params![
                sound.id,
                sound.title,
                sound.file_path,
                sound.volume_linear,
                sound.loudness_gain_db,
                sound.peak_db,
                sound.duration_ms,
                sound.source_kind,
            ],
        )?;
        Ok(())
    }

    pub fn delete_sound(&self, id: i64) -> Result<usize> {
        let n = self
            .conn
            .execute("DELETE FROM sounds WHERE id = ?1", params![id])?;
        Ok(n)
    }

    pub fn count_slots_using_sound(&self, sound_id: i64) -> Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM slots WHERE sound_id = ?1",
                params![sound_id],
                |r| r.get(0),
            )?)
    }

    pub fn count_slots_using_image_file(&self, filename: &str) -> Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM slots WHERE slot_appearance = 'image' AND image_path = ?1",
                params![filename],
                |r| r.get(0),
            )?)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get::<_, String>(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn audio_policy(&self) -> Result<AudioPolicy> {
        let v = self
            .get_setting("audio_policy")?
            .unwrap_or_else(|| "stop_previous".into());
        Ok(AudioPolicy::from_setting(&v))
    }

    pub fn global_volume(&self) -> Result<f32> {
        Ok(self
            .get_setting("global_volume")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7))
    }

    pub fn capture_max_duration_ms(&self) -> Result<u64> {
        Ok(self
            .get_setting("capture_max_duration_ms")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(60_000))
    }

    pub fn capture_min_duration_ms(&self) -> Result<u64> {
        Ok(self
            .get_setting("capture_min_duration_ms")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(300))
    }
}
