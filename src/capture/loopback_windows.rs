//! Capture de la sortie audio système (WASAPI loopback) — Windows uniquement.

use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use wasapi::{
    get_default_device, initialize_mta, Direction, SampleType, StreamMode, WaveFormat,
};

use super::CapturedAudio;

pub fn default_loopback_label() -> String {
    get_default_device(&Direction::Render)
        .ok()
        .and_then(|d| d.get_friendlyname().ok())
        .map(|n| match crate::i18n::current() {
            crate::i18n::Lang::En => format!("PC output (loopback): {n}"),
            crate::i18n::Lang::Fr => format!("Sortie PC (loopback) : {n}"),
        })
        .unwrap_or_else(|| match crate::i18n::current() {
            crate::i18n::Lang::En => "PC output — WASAPI loopback (default speakers)".into(),
            crate::i18n::Lang::Fr => "Sortie PC — WASAPI loopback (haut-parleurs par défaut)".into(),
        })
}

pub struct LoopbackSession {
    samples: Arc<Mutex<Vec<f32>>>,
    active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u16,
    thread: Option<JoinHandle<Result<()>>>,
}

impl LoopbackSession {
    pub fn start() -> Result<Self> {
        let _ = initialize_mta();

        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicBool::new(true));
        let stop = Arc::new(AtomicBool::new(false));

        let samples_t = samples.clone();
        let active_t = active.clone();
        let stop_t = stop.clone();

        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<(u32, u16)>>(1);

        let thread = std::thread::Builder::new()
            .name("wasapi-loopback".into())
            .spawn(move || capture_thread(samples_t, active_t, stop_t, ready_tx))
            .context("démarrer le thread loopback")?;

        let (sample_rate, channels) = ready_rx
            .recv()
            .context("thread loopback")?
            .context("initialisation loopback")?;

        Ok(Self {
            samples,
            active,
            stop,
            sample_rate,
            channels,
            thread: Some(thread),
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

    pub fn stop(mut self) -> Result<CapturedAudio> {
        self.active.store(false, Ordering::Relaxed);
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        let samples = self.samples.lock().unwrap().clone();
        Ok(CapturedAudio {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

fn capture_thread(
    samples: Arc<Mutex<Vec<f32>>>,
    active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    ready: std::sync::mpsc::SyncSender<Result<(u32, u16)>>,
) -> Result<()> {
    let device =
        get_default_device(&Direction::Render).context("aucune sortie audio par défaut")?;
    let mut audio_client = device
        .get_iaudioclient()
        .context("ouvrir le client WASAPI")?;

    let wavefmt = audio_client
        .get_mixformat()
        .context("format mixage de la sortie")?;
    let sample_rate = wavefmt.get_samplespersec();
    let channels = wavefmt.get_nchannels();
    let _ = ready.send(Ok((sample_rate, channels)));

    let (_, min_time) = audio_client.get_device_period().context("période device")?;
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: min_time,
    };
    audio_client
        .initialize_client(&wavefmt, &Direction::Capture, &mode)
        .context("initialiser capture loopback")?;

    let h_event = audio_client
        .set_get_eventhandle()
        .context("événement WASAPI")?;
    let capture_client = audio_client
        .get_audiocaptureclient()
        .context("client capture")?;

    let blockalign = wavefmt.get_blockalign() as usize;
    let mut byte_queue: VecDeque<u8> = VecDeque::new();

    audio_client.start_stream().context("démarrer loopback")?;

    while active.load(Ordering::Relaxed) && !stop.load(Ordering::Relaxed) {
        capture_client
            .read_from_device_to_deque(&mut byte_queue)
            .context("lire loopback")?;

        if byte_queue.len() >= blockalign {
            let take = byte_queue.len() - (byte_queue.len() % blockalign);
            let mut chunk = vec![0u8; take];
            for b in chunk.iter_mut() {
                *b = byte_queue.pop_front().unwrap_or(0);
            }
            if let Ok(mut buf) = samples.lock() {
                let _ = append_pcm_bytes(&mut buf, &chunk, &wavefmt);
            }
        }

        if h_event.wait_for_event(200).is_err() && stop.load(Ordering::Relaxed) {
            break;
        }
    }

    let _ = audio_client.stop_stream();
    Ok(())
}

fn append_pcm_bytes(out: &mut Vec<f32>, bytes: &[u8], fmt: &WaveFormat) -> Result<()> {
    let block_align = fmt.get_blockalign() as usize;
    if block_align == 0 {
        return Ok(());
    }
    let channels = fmt.get_nchannels() as usize;
    let sample_type = fmt.get_subformat().context("format PCM")?;
    let bytes_per_sample = (fmt.get_bitspersample() / 8) as usize;

    for frame in bytes.chunks(block_align) {
        if frame.len() < block_align {
            break;
        }
        for ch in 0..channels {
            let offset = ch * bytes_per_sample;
            if offset + bytes_per_sample > frame.len() {
                break;
            }
            let v = match sample_type {
                SampleType::Float => {
                    let bytes_s = &frame[offset..offset + bytes_per_sample];
                    if bytes_per_sample == 4 {
                        f32::from_le_bytes(bytes_s.try_into().context("float32")?)
                    } else {
                        continue;
                    }
                }
                SampleType::Int => match bytes_per_sample {
                    2 => {
                        let s = i16::from_le_bytes(frame[offset..offset + 2].try_into()?);
                        s as f32 / i16::MAX as f32
                    }
                    4 => {
                        let s = i32::from_le_bytes(frame[offset..offset + 4].try_into()?);
                        s as f32 / i32::MAX as f32
                    }
                    _ => continue,
                },
            };
            out.push(v.clamp(-1.0, 1.0));
        }
    }
    Ok(())
}
