use crate::audio::volume::effective_volume;
use crate::db::AudioPolicy;
use crate::paths::AppPaths;
use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
struct PlayingSink {
    sink: Sink,
}

pub struct AudioEngine {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sinks: Mutex<Vec<PlayingSink>>,
    policy: AudioPolicy,
    max_channels: u32,
    global_volume: f32,
    paths: AppPaths,
}

impl AudioEngine {
    pub fn new(paths: AppPaths, policy: AudioPolicy, max_channels: u32, global_volume: f32) -> Result<Self> {
        let (stream, handle) =
            OutputStream::try_default().context("open default audio output")?;
        Ok(Self {
            _stream: stream,
            handle,
            sinks: Mutex::new(Vec::new()),
            policy,
            max_channels,
            global_volume,
            paths,
        })
    }

    pub fn set_global_volume(&mut self, v: f32) {
        self.global_volume = v.clamp(0.0, 1.0);
    }

    pub fn set_policy(&mut self, policy: AudioPolicy, max_channels: u32) {
        self.policy = policy;
        self.max_channels = max_channels.max(1);
    }

    pub fn stop_all(&self) {
        let mut sinks = self.sinks.lock().unwrap();
        for s in sinks.drain(..) {
            s.sink.stop();
        }
    }

    fn prune_finished(sinks: &mut Vec<PlayingSink>) {
        sinks.retain(|s| !s.sink.empty());
    }

    pub fn play_file(
        &self,
        file_path: &str,
        sound_vol: f32,
        loudness_gain_db: f32,
        slot_vol: f32,
    ) -> Result<()> {
        let path = if Path::new(file_path).is_absolute() {
            Path::new(file_path).to_path_buf()
        } else {
            self.paths.sound_file(file_path)
        };
        let file = File::open(&path).with_context(|| format!("open sound {}", path.display()))?;
        let source = Decoder::new(BufReader::new(file)).context("decode audio")?;
        let vol = effective_volume(self.global_volume, sound_vol, loudness_gain_db, slot_vol);

        let mut sinks = self.sinks.lock().unwrap();
        Self::prune_finished(&mut sinks);

        match self.policy {
            AudioPolicy::StopPrevious => {
                for s in sinks.drain(..) {
                    s.sink.stop();
                }
            }
            AudioPolicy::Limited => {
                while sinks.len() >= self.max_channels as usize {
                    if let Some(old) = sinks.first() {
                        let _ = old.sink.stop();
                    }
                    sinks.remove(0);
                }
            }
            AudioPolicy::Overlap => {}
        }

        let sink = Sink::try_new(&self.handle).context("create sink")?;
        sink.set_volume(vol);
        sink.append(source);
        sinks.push(PlayingSink { sink });
        Ok(())
    }

    pub fn play_file_segment(&self, path: &Path, start_sec: f64, end_sec: f64) -> Result<()> {
        let start_sec = start_sec.max(0.0);
        let end_sec = end_sec.max(start_sec + 0.05);
        let file = File::open(path).with_context(|| format!("open clip {}", path.display()))?;
        let source = Decoder::new(BufReader::new(file)).context("decode clip preview")?;
        let clipped = source
            .skip_duration(Duration::from_secs_f64(start_sec))
            .take_duration(Duration::from_secs_f64(end_sec - start_sec));

        self.stop_all();
        let sink = Sink::try_new(&self.handle).context("create clip preview sink")?;
        sink.set_volume(self.global_volume);
        sink.append(clipped);
        self.sinks.lock().unwrap().push(PlayingSink { sink });
        Ok(())
    }
}

pub type SharedAudioEngine = Arc<Mutex<AudioEngine>>;
