# -*- coding: utf-8 -*-
from pathlib import Path
import re

root = Path(r"C:\Users\hvernet\Documents\02 - Projets\soundboard\streamdeck")
path = root / "ui" / "app-window.slint"
text = path.read_text(encoding="utf-8")

pairs = [
    ("Durée : —", "Duration: —"),
    ("Optionnel — ex. Applaudissements, Intro", "Optional — e.g. Applause, Intro"),
    ("Pour un dossier, ce nom crée aussi le chemin /accueil/nom.", "For a folder, this name also creates the path /home/name."),
    ("Icône", "Icon"),
    ("https://... (Unsplash, lien direct, …)", "https://... (Unsplash, direct link, …)"),
    ("Télécharger URL", "Download URL"),
    ("Interpréteur", "Interpreter"),
    ("Chemin du script (ex: C:\\scripts\\action.ps1)", "Script path (e.g. C:\\scripts\\action.ps1)"),
    ("Le script est lancé avec l interpréteur choisi. Chemins personnalisés dans Réglages.", "The script runs with the selected interpreter. Custom paths in Settings."),
    ("Heure de déclenchement", "Trigger time"),
    ("Raccourcis 5·10·15·30 min ou duree personnalisee ci-dessous", "Presets 5·10·15·30 min or custom duration below"),
    ("Bibliothèque", "Library"),
    ("Écouter", "Play"),
    ("Bibliothèque sons", "Sound library"),
    ("Bibliothèque images", "Image library"),
    ("Paramètres", "Settings"),
    ("Réglages application", "Application settings"),
    ("stop_previous = arrête le son en cours ; overlap = superpose les sons ; limited = limite le nombre de sons simultanés.", "stop_previous = stop current sound; overlap = layer sounds; limited = cap concurrent sounds."),
    ("Nombre maximum de sons joués en même temps. Utilisé uniquement en mode « limited ». 3 = trois sons simultanés au plus.", "Maximum sounds playing at once. Used only in « limited » mode. 3 = at most three concurrent sounds."),
    ("Niveau sonore cible à l'import (fichier, URL, capture). -16 = fort (broadcast) ; -23 = plus doux. ffmpeg normalise vers cette valeur (ffmpeg requis).", "Target loudness on import (file, URL, capture). -16 = loud (broadcast); -23 = quieter. ffmpeg normalizes to this value (ffmpeg required)."),
    ("Durée max capture (secondes, ex. 60)", "Max capture duration (seconds, e.g. 60)"),
    ("Durée maximale d'un enregistrement micro ou son PC. 60 = une minute maximum, puis arrêt automatique.", "Maximum length of a mic or PC recording. 60 = one minute max, then auto-stop."),
    ("Interpréteurs script (vide = détection auto)", "Script interpreters (empty = auto-detect)"),
    ("PowerShell (ex: C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe)", "PowerShell (e.g. C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe)"),
    ("CMD (ex: C:\\Windows\\System32\\cmd.exe)", "CMD (e.g. C:\\Windows\\System32\\cmd.exe)"),
    ("Bash (ex: /bin/bash ou C:\\Program Files\\Git\\bin\\bash.exe)", "Bash (e.g. /bin/bash or C:\\Program Files\\Git\\bin\\bash.exe)"),
    ("Python (ex: python, python3, C:\\Python312\\python.exe)", "Python (e.g. python, python3, C:\\Python312\\python.exe)"),
    ("Fenêtre", "Window"),
    ("Ligne 1 : A Z E R T — Ligne 2 : Q S D F G — Ligne 3 : W X C V B (positions AZERTY).", "Row 1: A Z E R T — Row 2: Q S D F G — Row 3: W X C V B (AZERTY positions)."),
    ("Découpe locale", "Local trim"),
    ("Parcourir un fichier audio — la ligne d onde apparait ici avec deux curseurs.", "Browse an audio file — the waveform appears here with two cursors."),
    ("Telechargement en cours — patientez…", "Download in progress — please wait…"),
    ("Collez une URL puis Charger — la ligne d onde avec deux curseurs apparait ici.", "Paste a URL then Load — the waveform with two cursors appears here."),
    ("Choisir une icône", "Choose an icon"),
]

pairs = sorted(pairs, key=lambda x: len(x[0]), reverse=True)
for fr, en in pairs:
    old = f'"{fr}"'
    new = f'@tr("{en}")'
    if old in text:
        text = text.replace(old, new)
        print("OK", fr[:60])
    else:
        print("MISS", fr[:60])

path.write_text(text, encoding="utf-8")

left = re.findall(r'(?:text|title|label|placeholder-text):\s*"([^"]+)"', text)
skip = {"Streamdeck", "STOP", "0.0s", "A", "10", "3", "-16", "60", "PS", "CMD", "Bash", "Py", "H", "min", "R", "V", "B", "PC", "URL", "Capture", "Clip", "Script"}
fr_left = [s for s in left if s not in skip and not s.startswith("http") and any(
    c in s for c in "éèêàùçôîÉÈÀ«»…·—"
) or any(w in s.lower() for w in ["dossier", "fichier", "durée", "réglage", "param", "biblioth", "écouter", "accueil", "arrête", "nombre", "niveau", "durée", "interprét", "fenêtre", "ligne 1", "découpe", "parcourir", "telecharg", "collez", "choisir une", "optionnel", "pour un", "chemin du", "le script", "heure de", "raccourcis 5", "télécharg"])]
print("LEFT:")
for s in left:
    if s in skip:
        continue
    if s.startswith("@tr"):
        continue
    # show non-ascii or french-ish
    if any(ord(c) > 127 for c in s) or any(w in s.lower() for w in ["arrête", "nombre", "niveau", "durée", "interprét", "fenêtre", "accueil", "dossier", "fichier", "réglage", "param", "biblioth", "écouter", "parcourir", "collez", "telecharg", "optionnel", "chemin", "heure", "raccourcis", "choisir", "découpe", "ligne 1", "stop_previous ="]):
        print(" ", s)
