# Contrôles du jeu

## Modes de jeu

Le jeu a **deux modes** :

### 🎮 Mode UI (Menu)
- Curseur visible ✓
- Peut interagir avec les menus et interfaces
- **Clic gauche** pour entrer en mode Jeu
- **ESC** pour quitter l'application

### 🎯 Mode Jeu (FPS)
- Curseur verrouillé et invisible
- Rotation de caméra avec la souris
- **ESC** pour revenir au mode UI

---

## Contrôles en mode Jeu

### Déplacement
| Touche | Action |
|--------|--------|
| **W** | Avancer |
| **S** | Reculer |
| **A** | Aller à gauche |
| **D** | Aller à droite |
| **Espace** | Monter ⬆️ |
| **C** | Descendre ⬇️ |

### Caméra
| Action | Effet |
|--------|-------|
| **Souris** | Rotation libre (infinie) |
| **Molette** | Zoom |

### Options
| Touche | Action |
|--------|--------|
| **ESC** | Retour au mode UI / Menu |
| **F11** | Basculer plein écran |
| **V** | Toggle VSync ON/OFF |
| **R** | Toggle rotation auto (debug) |

---

## Au démarrage

1. **La fenêtre s'ouvre** avec le curseur visible (mode UI)
2. **Cliquez n'importe où** dans la fenêtre pour commencer à jouer
3. Le curseur disparaît et vous pouvez contrôler la caméra
4. **Appuyez sur ESC** pour revenir au menu

---

## Architecture technique

### Mode UI
- Curseur: `Visible` + `CursorGrabMode::None`
- Utilisé pour: Menus, inventaire, paramètres, pause
- Rotation caméra: Désactivée

### Mode Jeu  
- Curseur: `Invisible` + `CursorGrabMode::Locked`
- Utilisé pour: Gameplay FPS
- Rotation caméra: `DeviceEvent::MouseMotion` (mouvement relatif)

### Plein écran
- **F11** pour activer/désactiver
- Mode: `Borderless(None)` (fullscreen sans bordures)
- Fonctionne dans les deux modes (UI et Jeu)
