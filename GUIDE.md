# Guide d'utilisation

## Grille

```
[  ] [  ] [  ] [  ] [  ]
[  ] [  ] [⌂ ] [  ] [  ]
[  ] [  ] [  ] [  ] [← ]
```

| Touche | Rôle |
|--------|------|
| **Centre** | Accueil (page racine) |
| **Bas droite** | Retour (sauf à l’accueil) |
| Autres | Son, dossier, script, alarme, ou vide |

## Barre du haut

| Zone | Action |
|------|--------|
| Gauche | Réglages · Édition |
| Centre | Chemin de page |
| Droite | Fermer |

**Réglages** → Sons · Images · Outils (Capture / Clip / URL) · Application.

## Configurer une touche

1. Activer **Édition**
2. Cliquer une case (hors Accueil / Retour)
3. Choisir le type, renseigner les options, **Sauver**
4. Quitter l’édition

### Types

| Type | Effet |
|------|--------|
| Son | Lit un fichier de la bibliothèque |
| Dossier | Ouvre un sous-menu |
| Script | Lance PowerShell / CMD / Bash / Python |
| Alarme | Heure fixe ou minuteur |
| Vide | Inutilisée |

Apparence : couleur + icône, ou image (bibliothèque Images).

### Sous-menu

Édition → case vide → **Dossier** → **Créer sous-menu** (ou Sauver).  
Navigation : dossier pour entrer, **Retour** / **Accueil** pour remonter.

### Son

Édition → **Son** → choisir dans la bibliothèque → **Sauver**.  
Importer : Réglages → **Sons** → **+ Fichier**.

## Raccourcis clavier

Activés via Réglages → Application. Mapping physique (AZERTY / QWERTY) :

| Ligne | Touches |
|-------|---------|
| 1 | A Z E R T |
| 2 | Q S D F G |
| 3 | W X C V B |

Personnalisation : Édition → bouton → **Changer** / **Défaut**. Inactifs en édition ou si une modale est ouverte.

## Données

À côté de l’exe (ou racine projet) :

| Chemin | Contenu |
|--------|---------|
| `soundboard.db` | Pages, touches, réglages |
| `assets/sounds/` | Audio |
| `assets/images/` | Images |
| `logs/`, `temp/` | Journaux, temporaires |

## Dépannage

| Problème | Solution |
|----------|----------|
| Dossier sans effet | Édition → Dossier → Créer sous-menu |
| Pas de son | Assigner un fichier puis Sauver |
| Import / normalisation | Installer ffmpeg dans `tools/ffmpeg/` |
