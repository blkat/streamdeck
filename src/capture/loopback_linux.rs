//! Capture de la sortie audio via source « monitor » PulseAudio / PipeWire (Linux).

use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};

use super::mic::MicSession;

fn is_monitor_device_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("monitor")
        || n.contains("loopback")
        || (n.contains("alsa_output") && n.contains(".monitor"))
        || n.starts_with("monitor of")
}

pub fn loopback_available() -> bool {
    find_monitor_device().is_ok()
}

pub fn find_monitor_device() -> Result<cpal::Device> {
    let host = cpal::default_host();
    let mut fallback = None;
    for dev in host.input_devices().context("lister les entrées audio")? {
        let Ok(name) = dev.name() else {
            continue;
        };
        if is_monitor_device_name(&name) {
            if name.to_lowercase().contains("monitor of") || name.contains(".monitor") {
                return Ok(dev);
            }
            if fallback.is_none() {
                fallback = Some(dev);
            }
        }
    }
    if let Some(dev) = fallback {
        return Ok(dev);
    }
    bail!(
        "Aucune source « monitor » trouvée. Sous PipeWire/PulseAudio, vérifiez que du son sort des haut-parleurs (pavucontrol → onglet Enregistrement)."
    )
}

pub fn default_loopback_label() -> String {
    find_monitor_device()
        .ok()
        .and_then(|d| d.name().ok())
        .map(|n| format!("Sortie PC (monitor) : {n}"))
        .unwrap_or_else(|| {
            "Sortie PC — source monitor introuvable (PulseAudio / PipeWire)".to_string()
        })
}

pub fn start() -> Result<MicSession> {
    let device = find_monitor_device()?;
    MicSession::start_on_device(device)
}
