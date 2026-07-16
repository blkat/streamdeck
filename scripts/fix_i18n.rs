// One-shot UTF-8 string rewriter for Slint i18n. Compile: rustc fix_i18n.rs && fix_i18n.exe
use std::fs;
use std::path::Path;

fn main() {
    let path = Path::new(r"C:\Users\hvernet\Documents\02 - Projets\soundboard\streamdeck\ui\app-window.slint");
    let mut text = fs::read_to_string(path).expect("read");
    let pairs: &[(&str, &str)] = &[
        ("Durée : —", "Duration: —"),
        ("Optionnel — ex. Applaudissements, Intro", "Optional — e.g. Applause, Intro"),
        ("Pour un dossier, ce nom crée aussi le chemin /accueil/nom.", "For a folder, this name also creates the path /home/name."),
        ("Icône", "Icon"),
        ("https://... (Unsplash, lien direct, …)", "https://... (Unsplash, direct link, …)"),
        ("Télécharger URL", "Download URL"),
        ("Interpréteur", "Interpreter"),
        ("Chemin du script (ex: C:\\\\scripts\\\\action.ps1)", "Script path (e.g. C:\\\\scripts\\\\action.ps1)"),
        ("Le script est lancé avec l interpréteur choisi. Chemins personnalisés dans Réglages.", "The script runs with the selected interpreter. Custom paths in Settings."),
        ("Heure de déclenchement", "Trigger time"),
        ("Raccourcis 5·10·15·30 min ou duree personnalisee ci-dessous", "Presets 5·10·15·30 min or custom duration below"),
        ("Bibliothèque sons", "Sound library"),
        ("Bibliothèque images", "Image library"),
        ("Bibliothèque", "Library"),
        ("Écouter", "Play"),
        ("Paramètres", "Settings"),
        ("Réglages application", "Application settings"),
        ("stop_previous = arrête le son en cours ; overlap = superpose les sons ; limited = limite le nombre de sons simultanés.", "stop_previous = stop current sound; overlap = layer sounds; limited = cap concurrent sounds."),
        ("Nombre maximum de sons joués en même temps. Utilisé uniquement en mode « limited ». 3 = trois sons simultanés au plus.", "Maximum sounds playing at once. Used only in « limited » mode. 3 = at most three concurrent sounds."),
        ("Niveau sonore cible à l'import (fichier, URL, capture). -16 = fort (broadcast) ; -23 = plus doux. ffmpeg normalise vers cette valeur (ffmpeg requis).", "Target loudness on import (file, URL, capture). -16 = loud (broadcast); -23 = quieter. ffmpeg normalizes to this value (ffmpeg required)."),
        ("Durée max capture (secondes, ex. 60)", "Max capture duration (seconds, e.g. 60)"),
        ("Durée maximale d'un enregistrement micro ou son PC. 60 = une minute maximum, puis arrêt automatique.", "Maximum length of a mic or PC recording. 60 = one minute max, then auto-stop."),
        ("Interpréteurs script (vide = détection auto)", "Script interpreters (empty = auto-detect)"),
        ("PowerShell (ex: C:\\\\Windows\\\\System32\\\\WindowsPowerShell\\\\v1.0\\\\powershell.exe)", "PowerShell (e.g. C:\\\\Windows\\\\System32\\\\WindowsPowerShell\\\\v1.0\\\\powershell.exe)"),
        ("CMD (ex: C:\\\\Windows\\\\System32\\\\cmd.exe)", "CMD (e.g. C:\\\\Windows\\\\System32\\\\cmd.exe)"),
        ("Bash (ex: /bin/bash ou C:\\\\Program Files\\\\Git\\\\bin\\\\bash.exe)", "Bash (e.g. /bin/bash or C:\\\\Program Files\\\\Git\\\\bin\\\\bash.exe)"),
        ("Python (ex: python, python3, C:\\\\Python312\\\\python.exe)", "Python (e.g. python, python3, C:\\\\Python312\\\\python.exe)"),
        ("Fenêtre", "Window"),
        ("Ligne 1 : A Z E R T — Ligne 2 : Q S D F G — Ligne 3 : W X C V B (positions AZERTY).", "Row 1: A Z E R T — Row 2: Q S D F G — Row 3: W X C V B (AZERTY positions)."),
        ("Découpe locale", "Local trim"),
        ("Parcourir un fichier audio — la ligne d onde apparait ici avec deux curseurs.", "Browse an audio file — the waveform appears here with two cursors."),
        ("Telechargement en cours — patientez…", "Download in progress — please wait…"),
        ("Collez une URL puis Charger — la ligne d onde avec deux curseurs apparait ici.", "Paste a URL then Load — the waveform with two cursors appears here."),
        ("Choisir une icône", "Choose an icon"),
    ];
    let mut pairs: Vec<_> = pairs.to_vec();
    pairs.sort_by_key(|(fr, _)| std::cmp::Reverse(fr.len()));
    for (fr, en) in pairs {
        let from = format!("\"{fr}\"");
        let to = format!("@tr(\"{en}\")");
        if text.contains(&from) {
            text = text.replace(&from, &to);
            println!("OK {}", &fr[..fr.len().min(50)]);
        } else {
            println!("MISS {}", &fr[..fr.len().min(50)]);
        }
    }
    fs::write(path, text).expect("write");
}
