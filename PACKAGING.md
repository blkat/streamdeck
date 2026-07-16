# Distribution

Livrer `sd-rs` avec ffmpeg embarqué (sans dépendre du PATH utilisateur).

## Contenu attendu

```
Streamdeck/
  sd-rs.exe
  tools/ffmpeg/   (ffmpeg.exe, ffprobe.exe)
  tools/yt-dlp/   (optionnel)
  assets/sounds/
  assets/images/
  GUIDE.md
  THIRD_PARTY.md
```

## Assemblage (Windows)

1. Placer ffmpeg (et yt-dlp si besoin) — [tools/README.md](tools/README.md)
2. Optionnel : `.\scripts\reset-user-data.ps1` (nettoyer les données de test)
3. Assembler :

```powershell
.\scripts\prepare-release.ps1
.\scripts\prepare-release.ps1 -Version auto
.\scripts\prepare-release.ps1 -SkipBuild
```

Sortie : `dist\Streamdeck\`. Tester : `.\sd-rs.exe` — statut `ffmpeg: OK`.

## Installeur

Inno Setup : modèle [scripts/streamdeck-installer.iss.example](scripts/streamdeck-installer.iss.example).  
Source : `dist\Streamdeck\*`.

## Notes

- Préférer `tools/` au PATH système.
- Linux : même structure, ou dépendance paquet `ffmpeg`.
- Licences : [THIRD_PARTY.md](THIRD_PARTY.md).

| Symptôme | Action |
|----------|--------|
| `ffmpeg: absent` | Vérifier `tools/ffmpeg/` |
| Clip URL échoue | Ajouter yt-dlp + ffmpeg |
| Données au mauvais endroit | Lancer depuis le dossier d’installation |
