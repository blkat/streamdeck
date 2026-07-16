use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::CapturedAudio;

pub struct MicSession {
    active: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    _stream: cpal::Stream,
}

impl MicSession {
    pub fn start(device_name: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let device = if let Some(name) = device_name.filter(|s| !s.is_empty()) {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                .context("périphérique micro introuvable")?
        } else {
            host.default_input_device().context("aucun micro")?
        };
        Self::start_on_device(device)
    }

    pub fn start_on_device(device: cpal::Device) -> Result<Self> {
        let config = device.default_input_config().context("config entrée audio")?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicBool::new(true));
        let samples_clone = samples.clone();
        let active_clone = active.clone();
        let config: cpal::StreamConfig = config.into();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                if active_clone.load(Ordering::Relaxed) {
                    if let Ok(mut buf) = samples_clone.lock() {
                        buf.extend_from_slice(data);
                    }
                }
            },
            |e| tracing::error!("capture entrée: {e}"),
            None,
        )?;
        stream.play().context("démarrer la capture")?;

        Ok(Self {
            active,
            samples,
            sample_rate,
            channels,
            _stream: stream,
        })
    }

    pub fn level_rms(&self) -> f32 {
        let buf = self.samples.lock().unwrap();
        if buf.is_empty() {
            return 0.0;
        }
        let tail = buf.len().saturating_sub(2048);
        let slice = &buf[tail..];
        let sum: f32 = slice.iter().map(|s| s * s).sum();
        (sum / slice.len() as f32).sqrt().min(1.0)
    }

    pub fn stop(self) -> Result<CapturedAudio> {
        self.active.store(false, Ordering::Relaxed);
        drop(self._stream);
        let samples = self.samples.lock().unwrap().clone();
        Ok(CapturedAudio {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let mut names = Vec::new();
    for dev in host.input_devices()? {
        if let Ok(name) = dev.name() {
            names.push(name);
        }
    }
    Ok(names)
}

pub fn default_input_device_label() -> String {
    let host = cpal::default_host();
    match host.default_input_device().and_then(|d| d.name().ok()) {
        Some(n) => match crate::i18n::current() {
            crate::i18n::Lang::En => format!("Microphone: {n}"),
            crate::i18n::Lang::Fr => format!("Microphone : {n}"),
        },
        None => match crate::i18n::current() {
            crate::i18n::Lang::En => "System default microphone".into(),
            crate::i18n::Lang::Fr => "Microphone par défaut du système".into(),
        },
    }
}
