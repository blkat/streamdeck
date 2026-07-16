use anyhow::{bail, Context, Result};
use crate::external_tools::{configure_subprocess, resolve_ffmpeg, resolve_ffprobe};
use crate::paths::AppPaths;
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub const WAVEFORM_BARS: usize = 64;

pub struct ClipSourceInfo {
    pub duration_secs: f64,
    pub peaks: Vec<f32>,
}

pub fn analyze_clip_source(paths: &AppPaths, path: &Path) -> Result<ClipSourceInfo> {
    let ffprobe = resolve_ffprobe(&paths.base);
    if let Ok(info) = analyze_clip_file(path, ffprobe.as_deref()) {
        if info.duration_secs > 0.0 {
            return Ok(info);
        }
    }
    if let Some(ffmpeg) = resolve_ffmpeg(&paths.base) {
        let wav = paths.temp.join(format!("analyze_{}.wav", Uuid::new_v4()));
        let mut cmd = Command::new(&ffmpeg);
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostats",
            "-y",
            "-i",
            path.to_str().context("chemin source")?,
            "-ac",
            "1",
            "-ar",
            "44100",
            wav.to_str().unwrap(),
        ]);
        configure_subprocess(&mut cmd);
        let status = cmd.status().context("ffmpeg analyse")?;
        if status.success() && wav.exists() {
            let info = analyze_clip_file(&wav, ffprobe.as_deref())?;
            let _ = std::fs::remove_file(&wav);
            if info.duration_secs > 0.0 {
                return Ok(info);
            }
        }
    }
    bail!(
        "Impossible d'analyser l'audio. Installez ffmpeg dans tools/ffmpeg (voir PACKAGING.md)."
    )
}

fn analyze_clip_file(path: &Path, ffprobe: Option<&Path>) -> Result<ClipSourceInfo> {
    let file = File::open(path).with_context(|| format!("ouverture {}", path.display()))?;
    let source = Decoder::new(BufReader::new(file)).context("décodage audio")?;
    let mut duration_secs = source
        .total_duration()
        .map(|d| d.as_secs_f64())
        .filter(|d| *d > 0.0)
        .unwrap_or_else(|| probe_duration_fallback(path).unwrap_or(0.0));

    if duration_secs <= 0.0 {
        if let Some(probe) = ffprobe {
            if let Ok(d) = probe_media_duration(probe, path) {
                duration_secs = d;
            }
        }
    }

    let sample_rate = source.sample_rate().max(1);
    let channels = source.channels().max(1) as u32;
    let mut peaks = vec![0.05f32; WAVEFORM_BARS];

    if duration_secs > 0.0 {
        let total_samples = duration_secs * sample_rate as f64 * channels as f64;
        let samples_per_bar = (total_samples / WAVEFORM_BARS as f64).max(1.0);
        let mut bar = 0usize;
        let mut count_in_bar = 0.0f64;
        let mut max_in_bar = 0.0f32;

        for sample in source.convert_samples::<f32>() {
            max_in_bar = max_in_bar.max(sample.abs());
            count_in_bar += 1.0;
            if count_in_bar >= samples_per_bar {
                if bar < WAVEFORM_BARS {
                    peaks[bar] = max_in_bar;
                }
                bar += 1;
                count_in_bar = 0.0;
                max_in_bar = 0.0;
            }
        }
        if bar < WAVEFORM_BARS {
            peaks[bar] = max_in_bar.max(peaks[bar]);
        }
    }

    if duration_secs <= 0.0 {
        bail!("Durée audio introuvable");
    }

    let max_peak = peaks.iter().cloned().fold(0.01f32, f32::max);
    for peak in &mut peaks {
        *peak = (*peak / max_peak).clamp(0.08, 1.0);
    }

    Ok(ClipSourceInfo {
        duration_secs,
        peaks,
    })
}

pub fn probe_media_duration(ffprobe: &Path, source: &Path) -> Result<f64> {
    let mut cmd = Command::new(ffprobe);
    cmd.args([
        "-hide_banner",
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        source.to_str().context("chemin media")?,
    ]);
    configure_subprocess(&mut cmd);
    let output = cmd.output().context("ffprobe")?;
    if !output.status.success() {
        bail!("ffprobe échoué");
    }
    let line = String::from_utf8_lossy(&output.stdout);
    line.trim()
        .parse::<f64>()
        .context("durée ffprobe invalide")
}

fn probe_duration_fallback(path: &Path) -> Option<f64> {
    if path.extension()?.to_str()?.eq_ignore_ascii_case("wav") {
        let reader = hound::WavReader::open(path).ok()?;
        let rate = reader.spec().sample_rate.max(1);
        return Some(reader.duration() as f64 / rate as f64);
    }
    None
}

pub fn format_clip_time(secs: f64) -> String {
    let total = secs.max(0.0);
    let m = (total as u64) / 60;
    let s = total - (m as f64 * 60.0);
    if m > 0 {
        format!("{m}:{s:04.1}")
    } else {
        format!("{s:.1}s")
    }
}

pub fn range_to_seconds(duration: f64, start_ratio: f32, end_ratio: f32) -> (f64, f64) {
    let start = (start_ratio as f64 * duration).clamp(0.0, duration);
    let mut end = (end_ratio as f64 * duration).clamp(0.0, duration);
    if end <= start {
        end = (start + 0.1).min(duration.max(start + 0.1));
    }
    if duration > 0.0 && end > duration {
        end = duration;
    }
    (start, end)
}
