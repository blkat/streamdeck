use crate::db::Sound;
use crate::external_tools::{
    configure_subprocess, ffmpeg_location_dir, resolve_ffmpeg, resolve_ffprobe, resolve_yt_dlp,
};
use crate::paths::AppPaths;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub struct ImportResult {
    pub file_name: String,
    pub peak_db: Option<f32>,
    pub loudness_gain_db: f32,
    pub duration_ms: Option<i32>,
}

pub struct SoundPipeline {
    paths: AppPaths,
    target_lufs: std::cell::Cell<f32>,
}

impl SoundPipeline {
    pub fn new(paths: AppPaths, target_lufs: f32) -> Self {
        Self {
            paths,
            target_lufs: std::cell::Cell::new(target_lufs),
        }
    }

    pub fn set_target_lufs(&self, lufs: f32) {
        self.target_lufs.set(lufs);
    }

    pub fn target_lufs(&self) -> f32 {
        self.target_lufs.get()
    }

    pub fn import_file(
        &self,
        source: &Path,
        title: &str,
        start_sec: Option<f64>,
        end_sec: Option<f64>,
        source_kind: &str,
    ) -> Result<(Sound, ImportResult)> {
        if let Some(ffmpeg) = resolve_ffmpeg(&self.paths.base) {
            return self.import_with_ffmpeg(
                ffmpeg.as_path(),
                source,
                title,
                start_sec,
                end_sec,
                source_kind,
            );
        }

        if start_sec.is_some() || end_sec.is_some() {
            bail!(
                "ffmpeg introuvable — placez ffmpeg dans tools/ffmpeg/ ou sur le PATH (voir PACKAGING.md)."
            );
        }

        tracing::warn!(
            "ffmpeg absent : import sans normalisation LUFS (son potentiellement plus faible/fort que les autres)"
        );
        self.import_copy(source, title, source_kind)
    }

    fn import_with_ffmpeg(
        &self,
        ffmpeg: &Path,
        source: &Path,
        title: &str,
        start_sec: Option<f64>,
        end_sec: Option<f64>,
        source_kind: &str,
    ) -> Result<(Sound, ImportResult)> {

        let id = Uuid::new_v4();
        let out_name = format!("{}.wav", id);
        let out_path = self.paths.sounds.join(&out_name);

        let target = self.target_lufs();
        if start_sec.is_some() || end_sec.is_some() {
            let start = start_sec.unwrap_or(0.0);
            let filter = format!("loudnorm=I={target}:TP=-1.0:LRA=11");
            let mut args = ffmpeg_quiet_args();
            args.extend([
                "-y".into(),
                "-ss".into(),
                format!("{start}"),
                "-i".into(),
                source.display().to_string(),
            ]);
            if let Some(end) = end_sec {
                args.push("-to".into());
                args.push(format!("{end}"));
            }
            args.extend([
                "-af".into(),
                filter,
                "-ac".into(),
                "2".into(),
                "-ar".into(),
                "44100".into(),
                out_path.display().to_string(),
            ]);
            run_ffmpeg(ffmpeg, &args)?;
        } else {
            let ext = source
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if ext.eq_ignore_ascii_case("wav") {
                normalize_wav(ffmpeg, source, &out_path, target)?;
            } else {
                let filter = format!("loudnorm=I={target}:TP=-1.0:LRA=11");
                run_ffmpeg(
                    ffmpeg,
                    &ffmpeg_quiet_args_with(vec![
                        "-y".into(),
                        "-i".into(),
                        source.display().to_string(),
                        "-af".into(),
                        filter,
                        "-ac".into(),
                        "2".into(),
                        "-ar".into(),
                        "44100".into(),
                        out_path.display().to_string(),
                    ]),
                )?;
            }
        }

        let duration_ms = probe_duration_ms(&self.paths.base, ffmpeg, &out_path).ok();
        // Déjà normalisé via loudnorm — pas de 2e passe d'analyse.
        let peak_db = None;
        let loudness_gain_db = 0.0;

        let sound = Sound {
            id: 0,
            title: title.to_string(),
            file_path: out_name,
            volume_linear: 1.0,
            loudness_gain_db,
            peak_db,
            duration_ms,
            source_kind: source_kind.to_string(),
        };

        Ok((
            sound,
            ImportResult {
                file_name: out_path.file_name().unwrap().to_string_lossy().into(),
                peak_db,
                loudness_gain_db,
                duration_ms,
            },
        ))
    }

    fn import_copy(
        &self,
        source: &Path,
        title: &str,
        source_kind: &str,
    ) -> Result<(Sound, ImportResult)> {
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        if !is_supported_audio_ext(ext) {
            bail!(
                "Format «.{ext}» non supporté. Utilisez wav, mp3, ogg, flac ou m4a."
            );
        }

        let id = Uuid::new_v4();
        let out_name = format!("{id}.{ext}");
        let out_path = self.paths.sounds.join(&out_name);
        std::fs::copy(source, &out_path).context("copie du fichier audio")?;

        let duration_ms = probe_duration_hound(&out_path).ok();

        let sound = Sound {
            id: 0,
            title: title.to_string(),
            file_path: out_name,
            volume_linear: 1.0,
            loudness_gain_db: 0.0,
            peak_db: None,
            duration_ms,
            source_kind: source_kind.to_string(),
        };

        Ok((
            sound,
            ImportResult {
                file_name: out_path.file_name().unwrap().to_string_lossy().into(),
                peak_db: None,
                loudness_gain_db: 0.0,
                duration_ms,
            },
        ))
    }

    pub fn reanalyze(&self, file_path: &Path) -> Result<(Option<f32>, f32, Option<i32>)> {
        let ffmpeg = resolve_ffmpeg(&self.paths.base).context(
            "ffmpeg introuvable — tools/ffmpeg/ ou PATH (voir PACKAGING.md)",
        )?;
        let path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.paths.sounds.join(file_path)
        };
        let duration_ms = probe_duration_ms(&self.paths.base, &ffmpeg, &path).ok();
        let (peak_db, gain) = analyze_loudness(ffmpeg.as_path(), &path, self.target_lufs())?;
        Ok((peak_db, gain, duration_ms))
    }
}

fn is_supported_audio_ext(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "wav" | "mp3" | "ogg" | "flac" | "m4a" | "aac"
    )
}

fn probe_duration_hound(path: &Path) -> Result<i32> {
    let reader = hound::WavReader::open(path).context("lecture WAV")?;
    let rate = reader.spec().sample_rate;
    let frames = reader.len();
    if rate == 0 {
        return Ok(0);
    }
    Ok((frames as i64 * 1000 / rate as i64) as i32)
}

fn ffmpeg_quiet_args() -> Vec<String> {
    vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-nostats".into(),
    ]
}

fn ffmpeg_quiet_args_with(extra: Vec<String>) -> Vec<String> {
    let mut args = ffmpeg_quiet_args();
    args.extend(extra);
    args
}

fn run_ffmpeg(ffmpeg: &Path, args: &[String]) -> Result<()> {
    let mut cmd = Command::new(ffmpeg);
    cmd.args(args);
    configure_subprocess(&mut cmd);
    let status = cmd.status().context("spawn ffmpeg")?;
    if !status.success() {
        bail!("ffmpeg failed: {:?}", args);
    }
    Ok(())
}

fn normalize_wav(ffmpeg: &Path, input: &Path, output: &Path, target_lufs: f32) -> Result<()> {
    let tmp = output.with_extension("norm.tmp.wav");
    let filter = format!(
        "loudnorm=I={}:TP=-1.0:LRA=11",
        target_lufs
    );
    run_ffmpeg(
        ffmpeg,
        &ffmpeg_quiet_args_with(vec![
            "-y".into(),
            "-i".into(),
            input.display().to_string(),
            "-af".into(),
            filter,
            "-ac".into(),
            "2".into(),
            "-ar".into(),
            "44100".into(),
            tmp.display().to_string(),
        ]),
    )?;
    std::fs::rename(&tmp, output).context("replace normalized wav")?;
    Ok(())
}

fn probe_duration_ms(base: &Path, ffmpeg: &Path, path: &Path) -> Result<i32> {
    let probe = resolve_ffprobe(base)
        .unwrap_or_else(|| ffmpeg.to_path_buf());
    let mut cmd = Command::new(&probe);
    cmd.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        &path.display().to_string(),
    ]);
    configure_subprocess(&mut cmd);
    let output = cmd.output().context("ffprobe duration")?;
    let s = String::from_utf8_lossy(&output.stdout);
    let sec: f64 = s.trim().parse().unwrap_or(0.0);
    Ok((sec * 1000.0) as i32)
}

fn analyze_loudness(ffmpeg: &Path, path: &Path, target_lufs: f32) -> Result<(Option<f32>, f32)> {
    let mut cmd = Command::new(ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostats",
        "-i",
        &path.display().to_string(),
        "-af",
        &format!("loudnorm=I={target_lufs}:print_format=json"),
        "-f",
        "null",
        "-",
    ]);
    configure_subprocess(&mut cmd);
    let output = cmd.output().ok();
    if let Some(out) = output {
        let stderr = String::from_utf8_lossy(&out.stderr);
        if let Some(input_i) = extract_json_field(&stderr, "input_i") {
            if let Ok(measured) = input_i.replace(',', ".").parse::<f32>() {
                let gain = target_lufs - measured;
                return Ok((Some(measured), gain));
            }
        }
    }
    Ok((None, 0.0))
}

fn extract_json_field(haystack: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let pos = haystack.find(&needle)?;
    let rest = &haystack[pos + needle.len()..];
    let rest = rest.split(':').nth(1)?;
    let value = rest
        .trim()
        .trim_start_matches('"')
        .split(['"', ',']).next()?;
    Some(value.trim().to_string())
}

/// Durée maximale par défaut pour le chargement URL (5 minutes).
pub const URL_DEFAULT_MAX_DURATION_SECS: f64 = 300.0;

#[derive(Clone)]
pub struct UrlProbeInfo {
    pub duration_secs: f64,
    pub title: Option<String>,
}

/// Durée et titre via yt-dlp, sinon ffprobe sur l’URL directe.
pub fn probe_url_info(paths: &AppPaths, url: &str) -> Result<UrlProbeInfo> {
    let mut yt_dlp_err: Option<anyhow::Error> = None;
    if let Some(yt_dlp) = resolve_yt_dlp(&paths.base) {
        match probe_url_with_yt_dlp(&paths.base, &yt_dlp, url) {
            Ok(info) => return Ok(info),
            Err(e) => yt_dlp_err = Some(e),
        }
    }
    if let Some(ffprobe) = resolve_ffprobe(&paths.base) {
        if let Ok(duration) = probe_duration_ffprobe(&ffprobe, url) {
            return Ok(UrlProbeInfo {
                duration_secs: duration,
                title: None,
            });
        }
    }
    if resolve_yt_dlp(&paths.base).is_none() && is_streaming_url(url) {
        bail!(
            "YouTube et sites similaires nécessitent yt-dlp. Placez yt-dlp.exe dans tools/yt-dlp/ (voir tools/README.md)."
        );
    }
    if let Some(e) = yt_dlp_err {
        bail!("{e:#}");
    }
    bail!(
        "Impossible de lire cette URL. Installez yt-dlp (YouTube) ou vérifiez le lien direct."
    );
}

fn is_streaming_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    [
        "youtube.com",
        "youtu.be",
        "vimeo.com",
        "twitch.tv",
        "soundcloud.com",
    ]
    .iter()
    .any(|host| lower.contains(host))
}

fn append_yt_dlp_ffmpeg_location(cmd: &mut Command, base: &Path) {
    if let Some(dir) = ffmpeg_location_dir(base) {
        cmd.arg("--ffmpeg-location").arg(dir);
    }
}

fn probe_url_with_yt_dlp(base: &Path, yt_dlp: &Path, url: &str) -> Result<UrlProbeInfo> {
    let mut cmd = Command::new(yt_dlp);
    append_yt_dlp_ffmpeg_location(&mut cmd, base);
    cmd.args([
        "--no-playlist",
        "--no-warnings",
        "-q",
        "--print",
        "duration",
        "--print",
        "title",
        url,
    ]);
    configure_subprocess(&mut cmd);
    let output = cmd.output().context("yt-dlp probe")?;
    if !output.status.success() {
        let stderr = trim_tool_stderr(&output.stderr);
        if stderr.is_empty() {
            bail!("yt-dlp n'a pas pu analyser l'URL");
        }
        bail!("yt-dlp : {stderr}");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let duration_line = lines.next().context("durée absente")?;
    let duration_secs: f64 = duration_line
        .parse()
        .with_context(|| format!("durée invalide : {duration_line}"))?;
    let title = lines.next().map(|t| t.to_string());
    Ok(UrlProbeInfo {
        duration_secs,
        title,
    })
}

fn probe_duration_ffprobe(ffprobe: &Path, url: &str) -> Result<f64> {
    let mut cmd = Command::new(ffprobe);
    cmd.args([
        "-hide_banner",
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        url,
    ]);
    configure_subprocess(&mut cmd);
    let output = cmd.output().context("ffprobe url")?;
    if !output.status.success() {
        bail!("ffprobe échoué");
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = line.trim().parse().context("durée ffprobe invalide")?;
    Ok(duration)
}

/// Télécharge l’audio complet de l’URL dans un fichier temporaire (aperçu / découpe).
pub fn download_url_audio_full(paths: &AppPaths, url: &str) -> Result<PathBuf> {
    let stem = paths.temp.join(format!("url_preview_{}", Uuid::new_v4()));
    let out_template = format!("{}.%(ext)s", stem.to_string_lossy());

    if let Some(yt_dlp) = resolve_yt_dlp(&paths.base) {
        let mut cmd = Command::new(&yt_dlp);
        append_yt_dlp_ffmpeg_location(&mut cmd, &paths.base);
        cmd.args([
            "--no-playlist",
            "--no-progress",
            "--no-warnings",
            "-q",
            "-x",
            "--audio-format",
            "wav",
            "-o",
            &out_template,
            url,
        ]);
        configure_subprocess(&mut cmd);
        let output = cmd.output().context("yt-dlp téléchargement")?;
        if output.status.success() {
            if let Some(path) = find_downloaded_audio(&stem) {
                return Ok(path);
            }
        }
        if is_streaming_url(url) {
            let stderr = trim_tool_stderr(&output.stderr);
            if stderr.is_empty() {
                bail!(
                    "Téléchargement YouTube échoué. Vérifiez tools/ffmpeg/ et tools/yt-dlp/ (voir tools/README.md)."
                );
            }
            bail!("Téléchargement YouTube échoué : {stderr}");
        }
    }

    let ffmpeg = resolve_ffmpeg(&paths.base).context(
        "ffmpeg ou yt-dlp requis pour l'extraction URL (voir PACKAGING.md)",
    )?;
    let out_wav = stem.with_extension("wav");
    let mut cmd = Command::new(&ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostats",
        "-y",
        "-i",
        url,
        "-ac",
        "2",
        "-ar",
        "44100",
        out_wav.to_str().unwrap(),
    ]);
    configure_subprocess(&mut cmd);
    let status = cmd.status().context("ffmpeg url")?;
    if !status.success() || !out_wav.exists() {
        bail!("Téléchargement URL échoué. Vérifiez l'URL et installez yt-dlp pour YouTube.");
    }
    Ok(out_wav)
}

fn find_downloaded_audio(stem: &Path) -> Option<PathBuf> {
    let wav = stem.with_extension("wav");
    if wav.exists() {
        return Some(wav);
    }
    let parent = stem.parent()?;
    let prefix = stem.file_name()?.to_str()?;
    std::fs::read_dir(parent)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix))
                .unwrap_or(false)
        })
}

pub fn extract_url_clip(
    paths: &AppPaths,
    url: &str,
    start: &str,
    end: &str,
    _target_lufs: f32,
) -> Result<PathBuf> {
    let temp_raw = paths.temp.join(format!("url_{}.wav", Uuid::new_v4()));

    if let Some(yt_dlp) = resolve_yt_dlp(&paths.base) {
        let section = format!("*{start}-{end}");
        let mut cmd = Command::new(&yt_dlp);
        append_yt_dlp_ffmpeg_location(&mut cmd, &paths.base);
        cmd.args([
            "--no-playlist",
            "--no-progress",
            "--no-warnings",
            "-q",
            "--download-sections",
            &section,
            "-x",
            "--audio-format",
            "wav",
            "-o",
            temp_raw.to_str().unwrap(),
            url,
        ]);
        configure_subprocess(&mut cmd);
        let status = cmd.status().context("yt-dlp")?;
        if status.success() && temp_raw.exists() {
            return Ok(temp_raw);
        }
    }

    let ffmpeg = resolve_ffmpeg(&paths.base).context(
        "ffmpeg ou yt-dlp requis pour l'extraction URL (voir PACKAGING.md)",
    )?;
    let mut cmd = Command::new(&ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostats",
        "-y",
        "-ss",
        start,
        "-to",
        end,
        "-i",
        url,
        "-ac",
        "2",
        "-ar",
        "44100",
        temp_raw.to_str().unwrap(),
    ]);
    configure_subprocess(&mut cmd);
    let status = cmd.status().context("ffmpeg url")?;
    if !status.success() {
        bail!("Extraction URL échouée. Vérifiez l'URL et installez yt-dlp.");
    }
    Ok(temp_raw)
}

fn trim_tool_stderr(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("WARNING:")
                && !line.starts_with("WARNING ")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(280)
        .collect()
}
