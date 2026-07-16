// Generate lang/fr/LC_MESSAGES/streamdeck.po from @tr() in Slint files.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn main() {
    let root = Path::new(r"C:\Users\hvernet\Documents\02 - Projets\soundboard\streamdeck");
    let mut msgs = BTreeMap::new();
    for rel in ["ui/app-window.slint", "ui/modal-footer.slint", "ui/clip-timeline.slint"] {
        let text = fs::read_to_string(root.join(rel)).expect(rel);
        extract_tr(&text, &mut msgs);
    }

    let fr: BTreeMap<&str, &str> = [
        ("Close", "Fermer"),
        ("Delete", "Supprimer"),
        ("Save", "Sauver"),
        ("Drag the green cursors on the timeline", "Glissez les curseurs verts sur la ligne"),
        ("Tools", "Outils"),
        ("Volume", "Volume"),
        ("Configure button", "Configurer le bouton"),
        ("Button type", "Type de bouton"),
        ("Empty", "Vide"),
        ("Sound", "Son"),
        ("Folder", "Dossier"),
        ("Alarm", "Alarme"),
        ("Button name", "Nom de la touche"),
        ("Optional — e.g. Applause, Intro", "Optionnel — ex. Applaudissements, Intro"),
        ("Keyboard shortcut", "Raccourci clavier"),
        ("Change", "Changer"),
        ("Default", "Defaut"),
        ("Default grid: A-Z-E-R-T / Q-S-D-F-G / W-X-C-V-B (physical AZERTY positions).", "Defaut grille : A-Z-E-R-T / Q-S-D-F-G / W-X-C-V-B (position physique AZERTY)."),
        ("For a folder, this name also creates the path /home/name.", "Pour un dossier, ce nom crée aussi le chemin /accueil/nom."),
        ("Black key, no icon (no action on click)", "Touche noire, sans icone (aucune action au clic)"),
        ("Button appearance", "Apparence de la touche"),
        ("Color + icon", "Couleur + icone"),
        ("Image", "Image"),
        ("Color", "Couleur"),
        ("Icon", "Icône"),
        ("File", "Fichier"),
        ("Library", "Bibliothèque"),
        ("Clear", "Effacer"),
        ("https://... (Unsplash, direct link, …)", "https://... (Unsplash, lien direct, …)"),
        ("Download URL", "Télécharger URL"),
        ("Interpreter", "Interpréteur"),
        ("Script path (e.g. C:\\scripts\\action.ps1)", "Chemin du script (ex: C:\\scripts\\action.ps1)"),
        ("Browse", "Parcourir"),
        ("The script runs with the selected interpreter. Custom paths in Settings.", "Le script est lancé avec l interpréteur choisi. Chemins personnalisés dans Réglages."),
        ("Alarm mode", "Mode alarme"),
        ("Clock", "Heure"),
        ("Timer", "Minuteur"),
        ("Trigger time", "Heure de déclenchement"),
        ("Presets 5·10·15·30 min or custom duration below", "Raccourcis 5·10·15·30 min ou duree personnalisee ci-dessous"),
        ("Other duration in minutes (e.g. 7, 25, 90)", "Autre duree en minutes (ex: 7, 25, 90)"),
        ("Sound / music", "Son / musique"),
        ("Play", "Écouter"),
        ("Test", "Tester"),
        ("Sound library", "Bibliothèque sons"),
        ("+ File", "+ Fichier"),
        ("Select", "Choisir"),
        ("Image library", "Bibliothèque images"),
        ("Image URL", "URL image"),
        ("Name", "Nom"),
        ("Settings", "Paramètres"),
        ("Sounds", "Sons"),
        ("Images", "Images"),
        ("Application", "Application"),
        ("Application settings", "Réglages application"),
        ("Audio playback", "Lecture audio"),
        ("Policy", "Politique"),
        ("stop_previous = stop current sound; overlap = layer sounds; limited = cap concurrent sounds.", "stop_previous = arrête le son en cours ; overlap = superpose les sons ; limited = limite le nombre de sons simultanés."),
        ("Max channels (e.g. 3)", "Canaux max (ex. 3)"),
        ("Maximum sounds playing at once. Used only in « limited » mode. 3 = at most three concurrent sounds.", "Nombre maximum de sons joués en même temps. Utilisé uniquement en mode « limited ». 3 = trois sons simultanés au plus."),
        ("Target loudness LUFS (e.g. -16)", "Volume cible LUFS (ex. -16)"),
        ("Target loudness on import (file, URL, capture). -16 = loud (broadcast); -23 = quieter. ffmpeg normalizes to this value (ffmpeg required).", "Niveau sonore cible à l'import (fichier, URL, capture). -16 = fort (broadcast) ; -23 = plus doux. ffmpeg normalise vers cette valeur (ffmpeg requis)."),
        ("Max capture duration (seconds, e.g. 60)", "Durée max capture (secondes, ex. 60)"),
        ("Maximum length of a mic or PC recording. 60 = one minute max, then auto-stop.", "Durée maximale d'un enregistrement micro ou son PC. 60 = une minute maximum, puis arrêt automatique."),
        ("Script interpreters (empty = auto-detect)", "Interpréteurs script (vide = détection auto)"),
        ("PowerShell (e.g. C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe)", "PowerShell (ex: C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe)"),
        ("CMD (e.g. C:\\Windows\\System32\\cmd.exe)", "CMD (ex: C:\\Windows\\System32\\cmd.exe)"),
        ("Bash (e.g. /bin/bash or C:\\Program Files\\Git\\bin\\bash.exe)", "Bash (ex: /bin/bash ou C:\\Program Files\\Git\\bin\\bash.exe)"),
        ("Python (e.g. python, python3, C:\\Python312\\python.exe)", "Python (ex: python, python3, C:\\Python312\\python.exe)"),
        ("Window", "Fenêtre"),
        ("Enable grid shortcuts", "Activer raccourcis grille"),
        ("Row 1: A Z E R T — Row 2: Q S D F G — Row 3: W X C V B (AZERTY positions).", "Ligne 1 : A Z E R T — Ligne 2 : Q S D F G — Ligne 3 : W X C V B (positions AZERTY)."),
        ("Keep Streamdeck on top", "Garder Streamdeck au premier plan"),
        ("Language", "Langue"),
        ("Audio recording", "Enregistrement audio"),
        ("Source", "Source"),
        ("Mic", "Micro"),
        ("Name (e.g. Applause)", "Nom (ex: Applaudissements)"),
        ("Cancel", "Annuler"),
        ("Local trim", "Découpe locale"),
        ("Source audio file", "Fichier audio source"),
        ("Browse an audio file — the waveform appears here with two cursors.", "Parcourir un fichier audio — la ligne d onde apparait ici avec deux curseurs."),
        ("Play selection", "Ecouter selection"),
        ("Sound name (e.g. Chorus, Intro)", "Nom du son (ex: Refrain, Intro)"),
        ("Move the green cursors on the timeline, listen, then Save.", "Deplacez les curseurs verts sur la ligne, ecoutez, puis Sauver."),
        ("Move the green cursors, listen, then Save.", "Deplacez les curseurs verts, ecoutez, puis Sauver."),
        ("URL extract", "Extrait URL"),
        ("https://... (YouTube, direct audio link)", "https://... (YouTube, lien direct audio)"),
        ("Allow videos longer than 5 min", "Autoriser les videos de plus de 5 min"),
        ("Warning: beyond 5 min, loading may take several minutes.", "Attention : au-dela de 5 min, le chargement peut prendre plusieurs minutes."),
        ("Default limit: videos up to 5 min to stay responsive.", "Limite par defaut : videos de 5 min maximum pour rester reactif."),
        ("Download in progress — please wait…", "Telechargement en cours — patientez…"),
        ("Paste a URL then Load — the waveform with two cursors appears here.", "Collez une URL puis Charger — la ligne d onde avec deux curseurs apparait ici."),
        ("Sound name (e.g. Applause, Jingle)", "Nom du son (ex: Applaudissements, Jingle)"),
        ("Button color", "Couleur de la touche"),
        ("Apply", "Valider"),
        ("Choose an icon", "Choisir une icône"),
        ("No icon", "Aucune icone"),
        ("ALARM", "ALARME"),
        ("Duration: —", "Durée : —"),
        ("Press a key…", "Appuyez sur une touche…"),
        ("Key: {}", "Touche : {}"),
        ("No sound selected", "Aucun son sélectionné"),
        ("1) Click a sound  2) Select  3) Save in the editor", "1) Cliquez un son  2) Choisir  3) Sauver dans l editeur"),
        ("Manage your sounds — import, play, delete.", "Gérez vos sons — importez, écoutez, supprimez."),
        ("1) Click an image  2) Select  3) Save in the editor", "1) Cliquez une image  2) Choisir  3) Sauver dans l editeur"),
        ("Shared images — import here, then assign in edit mode.", "Images partagees — importez ici, puis assignez en edition."),
        ("Load", "Charger"),
        ("Music, games and browser audio are recorded (speaker output).", "Musique, jeux et sons du navigateur sont enregistrés (sortie haut-parleurs)."),
        ("The microphone records your voice or a sound near the mic.", "Le microphone enregistre votre voix ou un son proche du micro."),
        ("● Recording", "● En cours"),
        ("✓ Done", "✓ Terminé"),
        ("Press REC", "Appuyez sur REC"),
    ]
    .iter()
    .copied()
    .collect();

    let mut out = String::from(
        "msgid \"\"\nmsgstr \"\"\n\"Content-Type: text/plain; charset=UTF-8\\n\"\n\"Language: fr\\n\"\n\n",
    );
    let mut missing = Vec::new();
    for en in msgs.keys() {
        let translation = fr.get(en.as_str()).copied().unwrap_or_else(|| {
            missing.push(en.clone());
            en.as_str()
        });
        out.push_str(&format!(
            "msgid \"{}\"\nmsgstr \"{}\"\n\n",
            escape_po(en),
            escape_po(translation)
        ));
    }
    let po = root.join("lang/fr/LC_MESSAGES/streamdeck.po");
    fs::create_dir_all(po.parent().unwrap()).unwrap();
    fs::write(&po, out).unwrap();
    println!("Wrote {} ({} entries, {} missing FR)", po.display(), msgs.len(), missing.len());
    for m in missing {
        println!("MISSING FR: {m}");
    }
}

fn escape_po(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn extract_tr(text: &str, msgs: &mut BTreeMap<String, ()>) {
    let mut rest = text;
    while let Some(idx) = rest.find("@tr(") {
        rest = &rest[idx + 4..];
        rest = rest.trim_start();
        if !rest.starts_with('"') {
            continue;
        }
        rest = &rest[1..];
        let mut s = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(n) = chars.next() {
                    s.push(n);
                }
                continue;
            }
            if c == '"' {
                break;
            }
            s.push(c);
        }
        msgs.insert(s, ());
        rest = chars.as_str();
    }
}
