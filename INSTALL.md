# Installation (Windows)

## Utilisateur final

Pas besoin de Rust. Décompressez le package (`sd-rs.exe` + `tools/`), lancez l’exe.  
Utilisation : [GUIDE.md](GUIDE.md).

## Développeur

### 1. Rust

```powershell
winget install Rustlang.Rustup
```

Ou https://rustup.rs — puis **rouvrir le terminal** :

```powershell
rustup default stable
cargo --version
```

### 2. Build Tools (MSVC)

Obligatoire pour Slint (`link.exe`).

- UI : [Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) → charge **Développement Desktop en C++**
- Ou (admin) :

```powershell
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### 3. ffmpeg

Placer `ffmpeg.exe` et `ffprobe.exe` dans `tools/ffmpeg/` — [tools/README.md](tools/README.md).  
Optionnel : `tools/yt-dlp/yt-dlp.exe`.

### 4. Lancer

```powershell
cd streamdeck   # dossier contenant Cargo.toml
cargo run
```

Premier build : plusieurs minutes. Release : `cargo build --release`.

### Dépannage

| Symptôme | Action |
|----------|--------|
| `cargo` introuvable | Rouvrir le terminal |
| `link.exe` not found | Installer Build Tools C++ |
| `ffmpeg: absent` | Copier les binaires dans `tools/ffmpeg/` |

## Données

Créées au 1er lancement à côté de l’exe (ou racine projet en `cargo run`) :  
`soundboard.db`, `assets/`, `logs/`, `temp/`.

Package complet : [PACKAGING.md](PACKAGING.md).

Traductions UI : fichiers `lang/fr/LC_MESSAGES/*.po` (anglais = source Slint `@tr`).
