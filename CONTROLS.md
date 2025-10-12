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
| **F** | Toggle WireFrame
| **P** | Spectator camera