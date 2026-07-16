# Outils embarqués

Résolution : `tools/` d’abord, puis PATH.

```
tools/
  ffmpeg/ffmpeg.exe
  ffmpeg/ffprobe.exe
  yt-dlp/yt-dlp.exe   # optionnel
```

Linux / macOS : mêmes noms sans `.exe`.

## Téléchargement (Windows)

**ffmpeg** — https://www.gyan.dev/ffmpeg/builds/ (`ffmpeg-release-essentials.zip`)  
→ extraire `bin/ffmpeg.exe` et `bin/ffprobe.exe` dans `tools/ffmpeg/`.

**yt-dlp** — https://github.com/yt-dlp/yt-dlp/releases → `tools/yt-dlp/`.  
Nécessite ffmpeg pour la conversion audio.

Vérification dans l’app : `ffmpeg: OK (…) | yt-dlp: OK (…)`.

Licences : [THIRD_PARTY.md](../THIRD_PARTY.md). Assemblage : `scripts/prepare-release.ps1`.
