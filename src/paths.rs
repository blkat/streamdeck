use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub base: PathBuf,
    pub db: PathBuf,
    pub sounds: PathBuf,
    pub images: PathBuf,
    pub logs: PathBuf,
    pub temp: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let start = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .or_else(|| std::env::current_dir().ok())
            .context("cannot resolve executable or current directory")?;

        let base = find_data_root(&start).unwrap_or(start);

        let paths = Self {
            db: base.join("soundboard.db"),
            sounds: base.join("assets").join("sounds"),
            images: base.join("assets").join("images"),
            logs: base.join("logs"),
            temp: base.join("temp"),
            base,
        };
        paths.ensure_dirs()?;
        Ok(paths)
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        for d in [&self.sounds, &self.images, &self.logs, &self.temp] {
            std::fs::create_dir_all(d)
                .with_context(|| format!("create dir {}", d.display()))?;
        }
        Ok(())
    }

    pub fn sound_file(&self, name: &str) -> PathBuf {
        self.sounds.join(name)
    }

    pub fn image_file(&self, name: &str) -> PathBuf {
        self.images.join(name)
    }
}

/// Remonte l'arborescence jusqu'à trouver le dossier projet (migrations/ ou assets/).
fn find_data_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..12 {
        if dir.join("migrations").join("001_init.sql").exists()
            || dir.join("Cargo.toml").exists() && dir.join("assets").exists()
        {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
