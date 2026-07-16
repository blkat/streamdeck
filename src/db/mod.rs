pub mod models;
pub mod repo;

use crate::paths::AppPaths;
use anyhow::{Context, Result};
use std::fs;

pub use models::*;
pub use repo::DbRepository;

const MIGRATION_001: &str = include_str!("../../migrations/001_init.sql");
const MIGRATION_002: &str = include_str!("../../migrations/002_slot_color.sql");
const MIGRATION_003: &str = include_str!("../../migrations/003_script_icon.sql");
const MIGRATION_004: &str = include_str!("../../migrations/004_alarm.sql");
const MIGRATION_005: &str = include_str!("../../migrations/005_alarm_timer.sql");
const MIGRATION_006: &str = include_str!("../../migrations/006_capture_source.sql");
const MIGRATION_007: &str = include_str!("../../migrations/007_slot_appearance.sql");
const MIGRATION_008: &str = include_str!("../../migrations/008_script_shell.sql");
const MIGRATION_009: &str = include_str!("../../migrations/009_slot_shortcut.sql");
const MIGRATION_010: &str = include_str!("../../migrations/010_ui_language.sql");

fn load_migration_001(paths: &AppPaths) -> Result<String> {
    let migration_path = paths.base.join("migrations").join("001_init.sql");
    if migration_path.exists() {
        fs::read_to_string(&migration_path)
            .with_context(|| format!("read migration {}", migration_path.display()))
    } else {
        Ok(MIGRATION_001.to_string())
    }
}

pub fn init_database(paths: &AppPaths) -> Result<DbRepository> {
    let repo = DbRepository::open(&paths.db)?;
    repo.init_schema_migrations()?;
    repo.bootstrap_legacy_migrations()?;

    if !repo.is_migration_applied(1)? {
        let sql = load_migration_001(paths)?;
        repo.migrate(&sql)?;
        repo.mark_migration_applied(1)?;
    }

    if !repo.is_migration_applied(2)? {
        if !repo.slots_have_color_hex()? {
            repo.migrate(MIGRATION_002)?;
        }
        repo.mark_migration_applied(2)?;
    }

    if !repo.is_migration_applied(3)? {
        if !repo.slots_have_script_command()? {
            repo.migrate(MIGRATION_003)?;
        }
        repo.mark_migration_applied(3)?;
    }

    if !repo.is_migration_applied(4)? {
        if !repo.slots_have_alarm_time()? {
            repo.migrate(MIGRATION_004)?;
        }
        repo.mark_migration_applied(4)?;
    }

    if !repo.is_migration_applied(5)? {
        if !repo.slots_have_alarm_mode()? {
            repo.migrate(MIGRATION_005)?;
        }
        repo.mark_migration_applied(5)?;
    }

    if !repo.is_migration_applied(6)? {
        repo.migrate(MIGRATION_006)?;
        repo.mark_migration_applied(6)?;
    }

    if !repo.is_migration_applied(7)? {
        if !repo.slots_have_slot_appearance()? {
            repo.migrate(MIGRATION_007)?;
        }
        repo.mark_migration_applied(7)?;
    }

    if !repo.is_migration_applied(8)? {
        if !repo.slots_have_script_shell()? {
            repo.migrate(MIGRATION_008)?;
        }
        repo.mark_migration_applied(8)?;
    }

    if !repo.is_migration_applied(9)? {
        if !repo.slots_have_shortcut_key()? {
            repo.migrate(MIGRATION_009)?;
        }
        repo.mark_migration_applied(9)?;
    }

    if !repo.is_migration_applied(10)? {
        repo.migrate(MIGRATION_010)?;
        repo.mark_migration_applied(10)?;
    }

    repo.ensure_root_page()?;
    Ok(repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::AppPaths;
    use tempfile::tempdir;

    #[test]
    fn migration_and_root() {
        let dir = tempdir().unwrap();
        let base = dir.path().to_path_buf();
        fs::create_dir_all(base.join("migrations")).unwrap();
        fs::create_dir_all(base.join("assets/sounds")).unwrap();
        fs::create_dir_all(base.join("assets/images")).unwrap();
        fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("migrations/001_init.sql"),
            base.join("migrations/001_init.sql"),
        )
        .unwrap();
        let paths = AppPaths {
            base: base.clone(),
            db: base.join("test.db"),
            sounds: base.join("assets/sounds"),
            images: base.join("assets/images"),
            logs: base.join("logs"),
            temp: base.join("temp"),
        };
        let repo = init_database(&paths).unwrap();
        assert_eq!(repo.root_page_id().unwrap(), 1);
        init_database(&paths).unwrap();
    }
}
