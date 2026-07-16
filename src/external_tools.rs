use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Sous-processus sans fenêtre console (Windows) et sans entrée stdin.
pub fn configure_subprocess(cmd: &mut Command) {
    cmd.stdin(Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
}

const FFMPEG_DIR: &str = "tools/ffmpeg";
const YTDLP_DIR: &str = "tools/yt-dlp";

/// Chemins embarqués puis PATH système (`where` / `which`).
pub fn resolve_ffmpeg(base: &Path) -> Option<PathBuf> {
    bundled_tool(base, FFMPEG_DIR, "ffmpeg").or_else(|| resolve_on_path("ffmpeg"))
}

pub fn resolve_ffprobe(base: &Path) -> Option<PathBuf> {
    bundled_tool(base, FFMPEG_DIR, "ffprobe")
        .or_else(|| resolve_on_path("ffprobe"))
        .or_else(|| {
            resolve_ffmpeg(base).map(|ffmpeg| {
                #[cfg(windows)]
                let name = "ffprobe.exe";
                #[cfg(not(windows))]
                let name = "ffprobe";
                ffmpeg.with_file_name(name)
            })
        })
}

pub fn resolve_yt_dlp(base: &Path) -> Option<PathBuf> {
    bundled_tool(base, YTDLP_DIR, "yt-dlp").or_else(|| resolve_on_path("yt-dlp"))
}

/// Dossier contenant ffmpeg/ffprobe pour `--ffmpeg-location` de yt-dlp.
pub fn ffmpeg_location_dir(base: &Path) -> Option<PathBuf> {
    resolve_ffmpeg(base)
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .filter(|d| d.is_dir())
}

pub fn has_ffmpeg(base: &Path) -> bool {
    resolve_ffmpeg(base).is_some()
}

pub fn has_yt_dlp(base: &Path) -> bool {
    resolve_yt_dlp(base).is_some()
}

pub fn tool_status_message(base: &Path) -> String {
    let ffmpeg = match resolve_ffmpeg(base) {
        Some(p) => format!("ffmpeg: OK ({})", short_tool_label(base, &p)),
        None => "ffmpeg: absent — installez tools/ffmpeg (voir PACKAGING.md)".to_string(),
    };
    let yt = match resolve_yt_dlp(base) {
        Some(p) => format!("yt-dlp: OK ({})", short_tool_label(base, &p)),
        None => "yt-dlp: absent (optionnel, tools/yt-dlp)".to_string(),
    };
    format!("{ffmpeg} | {yt}")
}

fn bundled_tool(base: &Path, subdir: &str, stem: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let names = [format!("{stem}.exe"), stem.to_string()];
    #[cfg(not(windows))]
    let names = [stem.to_string()];
    for name in names {
        let path = base.join(subdir).join(&name);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn resolve_on_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let output = Command::new("where").arg(name).output().ok()?;
        if !output.status.success() {
            return None;
        }
        first_line_path(&String::from_utf8_lossy(&output.stdout))
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("which").arg(name).output().ok()?;
        if !output.status.success() {
            return None;
        }
        first_line_path(&String::from_utf8_lossy(&output.stdout))
    }
}

fn first_line_path(stdout: &str) -> Option<PathBuf> {
    stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

fn short_tool_label(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string())
        })
}
