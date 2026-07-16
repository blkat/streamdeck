use std::sync::RwLock;

static LANG: RwLock<Lang> = RwLock::new(Lang::En);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Fr,
}

impl Lang {
    pub fn from_setting(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "fr" | "fra" | "french" | "français" | "francais" => Lang::Fr,
            _ => Lang::En,
        }
    }

    pub fn as_setting(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Fr => "fr",
        }
    }

    /// Empty / "en" = Slint source language (English).
    pub fn slint_locale(self) -> &'static str {
        match self {
            Lang::En => "",
            Lang::Fr => "fr",
        }
    }
}

pub fn current() -> Lang {
    *LANG.read().unwrap_or_else(|e| e.into_inner())
}

pub fn set_current(lang: Lang) {
    if let Ok(mut g) = LANG.write() {
        *g = lang;
    }
}

pub fn apply_slint(lang: Lang) {
    set_current(lang);
    let _ = slint::select_bundled_translation(lang.slint_locale());
}

fn pick<'a>(en: &'a str, fr: &'a str) -> &'a str {
    match current() {
        Lang::En => en,
        Lang::Fr => fr,
    }
}

pub fn default_kind_label(kind: &str) -> String {
    match kind {
        "folder" => pick("Folder", "Dossier").into(),
        "sound" => pick("Sound", "Son").into(),
        "script" => pick("Script", "Script").into(),
        "alarm" => pick("Alarm", "Alarme").into(),
        _ => String::new(),
    }
}

pub fn duration_placeholder() -> String {
    pick("Duration: —", "Durée : —").into()
}

pub fn duration_label(secs: &str) -> String {
    match current() {
        Lang::En => format!("Duration: {secs}"),
        Lang::Fr => format!("Durée : {secs}"),
    }
}

pub fn alarm_default_label() -> String {
    pick("Alarm", "Alarme").into()
}

pub fn shortcut_already_used(other: &str) -> String {
    match current() {
        Lang::En => format!("Already used by key {other}"),
        Lang::Fr => format!("Déjà utilisé par la touche {other}"),
    }
}

pub fn rfd_add_sound() -> &'static str {
    pick("Add sound", "Ajouter un son")
}

pub fn rfd_audio_files() -> &'static str {
    pick("Audio files", "Fichiers audio")
}

pub fn rfd_pick_script() -> &'static str {
    pick("Choose a script", "Choisir un script")
}

pub fn rfd_add_image() -> &'static str {
    pick("Add image", "Ajouter une image")
}

pub fn rfd_image_files() -> &'static str {
    pick("Images", "Images")
}

pub fn rfd_clip_source() -> &'static str {
    pick("Audio file to trim", "Fichier audio à découper")
}

pub fn capture_mic_hint() -> String {
    pick("Microphone", "Micro").into()
}

pub fn capture_pc_hint() -> String {
    pick("PC output", "Sortie PC").into()
}

pub fn url_status_loading() -> String {
    pick("Downloading…", "Téléchargement…").into()
}

pub fn url_status_ready() -> String {
    pick("Ready — adjust the selection then Save.", "Prêt — ajustez la sélection puis Sauver.").into()
}

pub fn url_status_error(err: &str) -> String {
    match current() {
        Lang::En => format!("Error: {err}"),
        Lang::Fr => format!("Erreur : {err}"),
    }
}

pub fn clip_status_error(err: &str) -> String {
    url_status_error(err)
}

pub fn rfd_slot_image() -> &'static str {
    pick("Image for button", "Image pour la touche")
}

pub fn url_need_url() -> String {
    pick("Enter a URL.", "Indiquez une URL.").into()
}

pub fn url_downloading() -> String {
    pick("Downloading audio…", "Téléchargement de l'audio…").into()
}

pub fn url_downloading_long(info: &str) -> String {
    match current() {
        Lang::En => format!("Downloading ({info}) — this may take several minutes…"),
        Lang::Fr => format!("Téléchargement en cours ({info}) — cela peut prendre plusieurs minutes…"),
    }
}

pub fn duration_zero() -> String {
    pick("Duration: 00:00", "Durée : 00:00").into()
}

pub fn url_too_long(time: &str) -> String {
    match current() {
        Lang::En => format!(
            "Video too long ({time}) — max 5 min. Check « Allow > 5 min » to continue."
        ),
        Lang::Fr => format!(
            "Vidéo trop longue ({time}) — max 5 min. Cochez « Autoriser > 5 min » pour continuer."
        ),
    }
}

pub fn root_page_name() -> &'static str {
    pick("Home", "Accueil")
}
