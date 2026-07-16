# Convert French UI literals to @tr("English") and emit fr.po entries.
$ErrorActionPreference = "Stop"
$root = "C:\Users\hvernet\Documents\02 - Projets\soundboard\streamdeck"

# FR => EN (exact string matches inside "...")
$pairs = @(
    @('Fermer', 'Close'),
    @('Supprimer', 'Delete'),
    @('Sauver', 'Save'),
    @('Glissez les curseurs verts sur la ligne', 'Drag the green cursors on the timeline'),
    @('Outils', 'Tools'),
    @('Volume', 'Volume'),
    @('Configurer le bouton', 'Configure button'),
    @('Type de bouton', 'Button type'),
    @('Vide', 'Empty'),
    @('Son', 'Sound'),
    @('Dossier', 'Folder'),
    @('Alarme', 'Alarm'),
    @('Nom de la touche', 'Button name'),
    @('Optionnel — ex. Applaudissements, Intro', 'Optional — e.g. Applause, Intro'),
    @('Raccourci clavier', 'Keyboard shortcut'),
    @('Changer', 'Change'),
    @('Defaut', 'Default'),
    @('Defaut grille : A-Z-E-R-T / Q-S-D-F-G / W-X-C-V-B (position physique AZERTY).', 'Default grid: A-Z-E-R-T / Q-S-D-F-G / W-X-C-V-B (physical AZERTY positions).'),
    @('Pour un dossier, ce nom crée aussi le chemin /accueil/nom.', 'For a folder, this name also creates the path /home/name.'),
    @('Touche noire, sans icone (aucune action au clic)', 'Black key, no icon (no action on click)'),
    @('Apparence de la touche', 'Button appearance'),
    @('Couleur + icone', 'Color + icon'),
    @('Image', 'Image'),
    @('Couleur', 'Color'),
    @('Icône', 'Icon'),
    @('Fichier', 'File'),
    @('Bibliotheque', 'Library'),
    @('Effacer', 'Clear'),
    @('https://... (Unsplash, lien direct, …)', 'https://... (Unsplash, direct link, …)'),
    @('Télécharger URL', 'Download URL'),
    @('Interpréteur', 'Interpreter'),
    @('Chemin du script (ex: C:\scripts\action.ps1)', 'Script path (e.g. C:\scripts\action.ps1)'),
    @('Parcourir', 'Browse'),
    @('Le script est lancé avec l interpréteur choisi. Chemins personnalisés dans Réglages.', 'The script runs with the selected interpreter. Custom paths in Settings.'),
    @('Mode alarme', 'Alarm mode'),
    @('Heure', 'Clock'),
    @('Minuteur', 'Timer'),
    @('Heure de déclenchement', 'Trigger time'),
    @('Raccourcis 5·10·15·30 min ou duree personnalisee ci-dessous', 'Presets 5·10·15·30 min or custom duration below'),
    @('Autre duree en minutes (ex: 7, 25, 90)', 'Other duration in minutes (e.g. 7, 25, 90)'),
    @('Son / musique', 'Sound / music'),
    @('Bibliothèque', 'Library'),
    @('Écouter', 'Play'),
    @('Tester', 'Test'),
    @('Bibliothèque sons', 'Sound library'),
    @('+ Fichier', '+ File'),
    @('Choisir', 'Select'),
    @('Bibliothèque images', 'Image library'),
    @('URL image', 'Image URL'),
    @('Nom', 'Name'),
    @('Paramètres', 'Settings'),
    @('Sons', 'Sounds'),
    @('Images', 'Images'),
    @('Application', 'Application'),
    @('Réglages application', 'Application settings'),
    @('Lecture audio', 'Audio playback'),
    @('Politique', 'Policy'),
    @('stop_previous = arrête le son en cours ; overlap = superpose les sons ; limited = limite le nombre de sons simultanés.', 'stop_previous = stop current sound; overlap = layer sounds; limited = cap concurrent sounds.'),
    @('Canaux max (ex. 3)', 'Max channels (e.g. 3)'),
    @('Nombre maximum de sons joués en même temps. Utilisé uniquement en mode « limited ». 3 = trois sons simultanés au plus.', 'Maximum sounds playing at once. Used only in « limited » mode. 3 = at most three concurrent sounds.'),
    @('Volume cible LUFS (ex. -16)', 'Target loudness LUFS (e.g. -16)'),
    @("Niveau sonore cible à l'import (fichier, URL, capture). -16 = fort (broadcast) ; -23 = plus doux. ffmpeg normalise vers cette valeur (ffmpeg requis).", 'Target loudness on import (file, URL, capture). -16 = loud (broadcast); -23 = quieter. ffmpeg normalizes to this value (ffmpeg required).'),
    @('Durée max capture (secondes, ex. 60)', 'Max capture duration (seconds, e.g. 60)'),
    @("Durée maximale d'un enregistrement micro ou son PC. 60 = une minute maximum, puis arrêt automatique.", 'Maximum length of a mic or PC recording. 60 = one minute max, then auto-stop.'),
    @('Interpréteurs script (vide = détection auto)', 'Script interpreters (empty = auto-detect)'),
    @('PowerShell (ex: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe)', 'PowerShell (e.g. C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe)'),
    @('CMD (ex: C:\Windows\System32\cmd.exe)', 'CMD (e.g. C:\Windows\System32\cmd.exe)'),
    @('Bash (ex: /bin/bash ou C:\Program Files\Git\bin\bash.exe)', 'Bash (e.g. /bin/bash or C:\Program Files\Git\bin\bash.exe)'),
    @('Python (ex: python, python3, C:\Python312\python.exe)', 'Python (e.g. python, python3, C:\Python312\python.exe)'),
    @('Fenêtre', 'Window'),
    @('Activer raccourcis grille', 'Enable grid shortcuts'),
    @('Ligne 1 : A Z E R T — Ligne 2 : Q S D F G — Ligne 3 : W X C V B (positions AZERTY).', 'Row 1: A Z E R T — Row 2: Q S D F G — Row 3: W X C V B (AZERTY positions).'),
    @('Garder Streamdeck au premier plan', 'Keep Streamdeck on top'),
    @('Enregistrement audio', 'Audio recording'),
    @('Source', 'Source'),
    @('Micro', 'Mic'),
    @('Nom (ex: Applaudissements)', 'Name (e.g. Applause)'),
    @('Annuler', 'Cancel'),
    @('Découpe locale', 'Local trim'),
    @('Fichier audio source', 'Source audio file'),
    @('Parcourir un fichier audio — la ligne d onde apparait ici avec deux curseurs.', 'Browse an audio file — the waveform appears here with two cursors.'),
    @('Ecouter selection', 'Play selection'),
    @('Nom du son (ex: Refrain, Intro)', 'Sound name (e.g. Chorus, Intro)'),
    @('Deplacez les curseurs verts sur la ligne, ecoutez, puis Sauver.', 'Move the green cursors, listen, then Save.'),
    @('Extrait URL', 'URL extract'),
    @('https://... (YouTube, lien direct audio)', 'https://... (YouTube, direct audio link)'),
    @('Autoriser les videos de plus de 5 min', 'Allow videos longer than 5 min'),
    @('Attention : au-dela de 5 min, le chargement peut prendre plusieurs minutes.', 'Warning: beyond 5 min, loading may take several minutes.'),
    @('Limite par defaut : videos de 5 min maximum pour rester reactif.', 'Default limit: videos up to 5 min to stay responsive.'),
    @('Telechargement en cours — patientez…', 'Download in progress — please wait…'),
    @('Collez une URL puis Charger — la ligne d onde avec deux curseurs apparait ici.', 'Paste a URL then Load — the waveform with two cursors appears here.'),
    @('Nom du son (ex: Applaudissements, Jingle)', 'Sound name (e.g. Applause, Jingle)'),
    @('Deplacez les curseurs verts, ecoutez, puis Sauver.', 'Move the green cursors, listen, then Save.'),
    @('Couleur de la touche', 'Button color'),
    @('Valider', 'Apply'),
    @('Choisir une icône', 'Choose an icon'),
    @('Aucune icone', 'No icon'),
    @('ALARME', 'ALARM'),
    @('Durée : —', 'Duration: —'),
    @('Language', 'Language'),
    @('English', 'English'),
    @('Français', 'Français')
)

# Sort by FR length descending
$pairs = $pairs | Sort-Object { $_[0].Length } -Descending

function Escape-Po([string]$s) {
    return (($s -replace '\\', '\\') -replace '"', '\"')
}

foreach ($rel in @('ui\modal-footer.slint', 'ui\clip-timeline.slint', 'ui\app-window.slint')) {
    $path = Join-Path $root $rel
    $text = [System.IO.File]::ReadAllText($path)
    foreach ($p in $pairs) {
        $fr = $p[0]
        $en = $p[1]
        $from = '"' + $fr + '"'
        $to = '@tr("' + $en + '")'
        if ($text.Contains($from)) {
            $text = $text.Replace($from, $to)
        }
    }
    [System.IO.File]::WriteAllText($path, $text, [System.Text.UTF8Encoding]::new($false))
    Write-Host "Updated $rel"
}

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine('msgid ""')
[void]$sb.AppendLine('msgstr ""')
[void]$sb.AppendLine('"Content-Type: text/plain; charset=UTF-8\n"')
[void]$sb.AppendLine('"Language: fr\n"')
[void]$sb.AppendLine('')
$seen = @{}
foreach ($p in $pairs) {
    $fr = $p[0]; $en = $p[1]
    if ($seen.ContainsKey($en)) { continue }
    $seen[$en] = $true
    [void]$sb.AppendLine(('msgid "{0}"' -f (Escape-Po $en)))
    [void]$sb.AppendLine(('msgstr "{0}"' -f (Escape-Po $fr)))
    [void]$sb.AppendLine('')
}
$poPath = Join-Path $root 'lang\fr\LC_MESSAGES\streamdeck.po'
New-Item -ItemType Directory -Force -Path (Split-Path $poPath) | Out-Null
[System.IO.File]::WriteAllText($poPath, $sb.ToString(), [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote $poPath ($($seen.Count) entries)"
