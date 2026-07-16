mod mic;

#[cfg(windows)]
mod loopback_windows;

#[cfg(target_os = "linux")]
mod loopback_linux;

use anyhow::{Context, Result};
use std::time::Instant;

pub use mic::{default_input_device_label, list_input_devices};

#[cfg(windows)]
pub use loopback_windows::default_loopback_label;

#[cfg(target_os = "linux")]
pub use loopback_linux::default_loopback_label;

/// Source de capture : micro ou sortie systeme (loopback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSource {
    Microphone,
    SystemLoopback,
}

impl CaptureSource {
    pub fn from_setting(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "loopback" | "system" | "pc" | "sortie" => Self::SystemLoopback,
            _ => Self::Microphone,
        }
    }

    pub fn as_setting(&self) -> &'static str {
        match self {
            Self::Microphone => "microphone",
            Self::SystemLoopback => "loopback",
        }
    }

    pub fn loopback_available() -> bool {
        #[cfg(windows)]
        {
            true
        }
        #[cfg(target_os = "linux")]
        {
            loopback_linux::loopback_available()
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            false
        }
    }
}

enum RecordingBackend {
    Mic(mic::MicSession),
    #[cfg(windows)]
    Loopback(loopback_windows::LoopbackSession),
}

pub struct RecordingSession {
    backend: RecordingBackend,
    started: Instant,
}

impl RecordingSession {
    pub fn start(source: CaptureSource, device_name: Option<&str>) -> Result<Self> {
        let backend = match source {
            CaptureSource::Microphone => RecordingBackend::Mic(mic::MicSession::start(device_name)?),
            CaptureSource::SystemLoopback => {
                #[cfg(windows)]
                {
                    RecordingBackend::Loopback(loopback_windows::LoopbackSession::start()?)
                }
                #[cfg(target_os = "linux")]
                {
                    RecordingBackend::Mic(loopback_linux::start()?)
                }
                #[cfg(not(any(windows, target_os = "linux")))]
                {
                    anyhow::bail!(
                        "Capture sortie PC : Windows (WASAPI) ou Linux (monitor Pulse/PipeWire) uniquement."
                    );
                }
            }
        };
        Ok(Self {
            backend,
            started: Instant::now(),
        })
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    pub fn level_rms(&self) -> f32 {
        match &self.backend {
            RecordingBackend::Mic(s) => s.level_rms(),
            #[cfg(windows)]
            RecordingBackend::Loopback(s) => s.level_rms(),
        }
    }

    pub fn stop(self) -> Result<CapturedAudio> {
        match self.backend {
            RecordingBackend::Mic(s) => s.stop(),
            #[cfg(windows)]
            RecordingBackend::Loopback(s) => s.stop(),
        }
    }
}

pub struct CapturedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl CapturedAudio {
    pub fn duration_ms(&self) -> u64 {
        if self.channels == 0 || self.sample_rate == 0 {
            return 0;
        }
        let frames = self.samples.len() as u64 / self.channels as u64;
        frames * 1000 / self.sample_rate as u64
    }

    pub fn write_wav(&self, path: &std::path::Path) -> Result<()> {
        let spec = hound::WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).context("create wav")?;
        for &s in &self.samples {
            let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(v)?;
        }
        writer.finalize()?;
        Ok(())
    }
}

pub fn capture_source_label(source: CaptureSource) -> String {
    match source {
        CaptureSource::Microphone => default_input_device_label(),
        CaptureSource::SystemLoopback => {
            #[cfg(windows)]
            {
                default_loopback_label()
            }
            #[cfg(target_os = "linux")]
            {
                default_loopback_label()
            }
            #[cfg(not(any(windows, target_os = "linux")))]
            {
                "Sortie PC — non disponible sur cette plateforme".to_string()
            }
        }
    }
}
