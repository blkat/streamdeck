$ErrorActionPreference = "Stop"
$path = "C:\Users\hvernet\Documents\02 - Projets\soundboard\streamdeck\ui\app-window.slint"
$text = [System.IO.File]::ReadAllText($path, [System.Text.UTF8Encoding]::new($false))

$pairs = @(
    ,@("Duration: —", $null) # skip if already EN
)

# Build replacements: quoted FR -> @tr(EN)
$repl = [System.Collections.Generic.List[object]]::new()

function Add-Pair([string]$fr, [string]$en) {
    $script:repl.Add([tuple]::Create($fr, $en)) | Out-Null
}

Add-Pair "Durée : —" "Duration: —"
Add-Pair "Optionnel — ex. Applaudissements, Intro" "Optional — e.g. Applause, Intro"
Add-Pair "Pour un dossier, ce nom crée aussi le chemin /accueil/nom." "For a folder, this name also creates the path /home/name."
Add-Pair "Icône" "Icon"
Add-Pair "https://... (Unsplash, lien direct, …)" "https://... (Unsplash, direct link, …)"
Add-Pair "Télécharger URL" "Download URL"
Add-Pair "Interpréteur" "Interpreter"
Add-Pair 'Chemin du script (ex: C:\scripts\action.ps1)' 'Script path (e.g. C:\scripts\action.ps1)'
Add-Pair "Le script est lancé avec l interpréteur choisi. Chemins personnalisés dans Réglages." "The script runs with the selected interpreter. Custom paths in Settings."
Add-Pair "Heure de déclenchement" "Trigger time"
Add-Pair "Raccourcis 5·10·15·30 min ou duree personnalisee ci-dessous" "Presets 5·10·15·30 min or custom duration below"
Add-Pair "Bibliothèque" "Library"
Add-Pair "Écouter" "Play"
Add-Pair "Bibliothèque sons" "Sound library"
Add-Pair "Bibliothèque images" "Image library"
Add-Pair "Paramètres" "Settings"
Add-Pair "Réglages application" "Application settings"
Add-Pair "stop_previous = arrête le son en cours ; overlap = superpose les sons ; limited = limite le nombre de sons simultanés." "stop_previous = stop current sound; overlap = layer sounds; limited = cap concurrent sounds."
Add-Pair "Nombre maximum de sons joués en même temps. Utilisé uniquement en mode « limited ». 3 = trois sons simultanés au plus." "Maximum sounds playing at once. Used only in « limited » mode. 3 = at most three concurrent sounds."
Add-Pair "Niveau sonore cible à l'import (fichier, URL, capture). -16 = fort (broadcast) ; -23 = plus doux. ffmpeg normalise vers cette valeur (ffmpeg requis)." "Target loudness on import (file, URL, capture). -16 = loud (broadcast); -23 = quieter. ffmpeg normalizes to this value (ffmpeg required)."
Add-Pair "Durée max capture (secondes, ex. 60)" "Max capture duration (seconds, e.g. 60)"
Add-Pair "Durée maximale d'un enregistrement micro ou son PC. 60 = une minute maximum, puis arrêt automatique." "Maximum length of a mic or PC recording. 60 = one minute max, then auto-stop."
Add-Pair "Interpréteurs script (vide = détection auto)" "Script interpreters (empty = auto-detect)"
Add-Pair 'PowerShell (ex: C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe)' 'PowerShell (e.g. C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe)'
Add-Pair 'CMD (ex: C:\Windows\System32\cmd.exe)' 'CMD (e.g. C:\Windows\System32\cmd.exe)'
Add-Pair 'Bash (ex: /bin/bash ou C:\Program Files\Git\bin\bash.exe)' 'Bash (e.g. /bin/bash or C:\Program Files\Git\bin\bash.exe)'
Add-Pair 'Python (ex: python, python3, C:\Python312\python.exe)' 'Python (e.g. python, python3, C:\Python312\python.exe)'
Add-Pair "Fenêtre" "Window"
Add-Pair "Ligne 1 : A Z E R T — Ligne 2 : Q S D F G — Ligne 3 : W X C V B (positions AZERTY)." "Row 1: A Z E R T — Row 2: Q S D F G — Row 3: W X C V B (AZERTY positions)."
Add-Pair "Découpe locale" "Local trim"
Add-Pair "Parcourir un fichier audio — la ligne d onde apparait ici avec deux curseurs." "Browse an audio file — the waveform appears here with two cursors."
Add-Pair "Telechargement en cours — patientez…" "Download in progress — please wait…"
Add-Pair "Collez une URL puis Charger — la ligne d onde avec deux curseurs apparait ici." "Paste a URL then Load — the waveform with two cursors appears here."
Add-Pair "Choisir une icône" "Choose an icon"

$sorted = $repl | Sort-Object { $_.Item1.Length } -Descending
$ok = 0; $miss = 0
foreach ($t in $sorted) {
    $fr = $t.Item1; $en = $t.Item2
    $from = '"' + $fr + '"'
    $to = '@tr("' + $en + '")'
    if ($text.Contains($from)) {
        $text = $text.Replace($from, $to)
        $ok++
        Write-Host "OK: $($fr.Substring(0, [Math]::Min(50,$fr.Length)))"
    } else {
        $miss++
        Write-Host "MISS: $($fr.Substring(0, [Math]::Min(50,$fr.Length)))"
    }
}

[System.IO.File]::WriteAllText($path, $text, [System.Text.UTF8Encoding]::new($false))
Write-Host "Done ok=$ok miss=$miss"

# Show remaining quoted non-@tr user strings with non-ascii
$rx = [regex]'(?:text|title|label|placeholder-text):\s*"([^"]+)"'
foreach ($m in $rx.Matches($text)) {
    $s = $m.Groups[1].Value
    if ($s -match '[^\x00-\x7F]' -or $s -match 'arrête|Param|Régl|Bibli|Écouter|Durée|Interpr|Fenêt|Découpe|Optionnel|accueil|Nombre|Niveau') {
        Write-Host "LEFT: $s"
    }
}
