use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Page {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    pub volume_linear: f32,
    pub loudness_gain_db: f32,
    pub peak_db: Option<f32>,
    pub duration_ms: Option<i32>,
    pub source_kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Sound,
    Folder,
    Script,
    Alarm,
    Empty,
}

impl SlotKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SlotKind::Sound => "sound",
            SlotKind::Folder => "folder",
            SlotKind::Script => "script",
            SlotKind::Alarm => "alarm",
            SlotKind::Empty => "empty",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "sound" => SlotKind::Sound,
            "folder" => SlotKind::Folder,
            "script" => SlotKind::Script,
            "alarm" => SlotKind::Alarm,
            _ => SlotKind::Empty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub page_id: i64,
    pub row: i32,
    pub col: i32,
    pub kind: SlotKind,
    pub label: Option<String>,
    pub image_path: Option<String>,
    /// `color` = fond couleur + icône catalogue optionnelle ; `image` = photo (fichier dans assets/images/)
    pub slot_appearance: String,
    pub sound_id: Option<i64>,
    pub child_page_id: Option<i64>,
    pub slot_volume: f32,
    pub color_hex: Option<String>,
    pub script_command: Option<String>,
    /// `powershell` | `cmd` | `bash` | `python`
    pub script_shell: Option<String>,
    pub alarm_time: Option<String>,
    /// `clock` = heure fixe (HH:MM), `timer` = compte à rebours activable sur la touche
    pub alarm_mode: String,
    pub alarm_minutes: Option<i32>,
    pub alarm_armed: bool,
    pub alarm_armed_at_ms: Option<i64>,
    /// Code touche physique winit (`KeyQ`, …) ; `None` = raccourci par défaut de la cellule.
    pub shortcut_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioPolicy {
    #[serde(rename = "stop_previous")]
    StopPrevious,
    #[serde(rename = "overlap")]
    Overlap,
    #[serde(rename = "limited")]
    Limited,
}

impl AudioPolicy {
    pub fn from_setting(s: &str) -> Self {
        match s {
            "overlap" => AudioPolicy::Overlap,
            "limited" => AudioPolicy::Limited,
            _ => AudioPolicy::StopPrevious,
        }
    }

    pub fn to_setting(&self) -> &'static str {
        match self {
            AudioPolicy::StopPrevious => "stop_previous",
            AudioPolicy::Overlap => "overlap",
            AudioPolicy::Limited => "limited",
        }
    }
}
