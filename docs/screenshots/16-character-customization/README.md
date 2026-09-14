# Character customization — inspected native evidence

2026-09-14. All PNGs are full native Bevy/Metal framebuffer captures on Apple M1 Pro, converted losslessly from P6 RGB PPM. No generated images, compositing, cropping or browser mockups. All listed frames were visually inspected.

Frames 01–10 are final debug-only injected rendering fixtures, not physical input or gameplay-route passes. Their logical sizes are 1280×720 (desktop) or 480×320 (07/08); Retina backing pixels are shown below. Frames 11–18 are ordinary mouse/keyboard preview/save/restart interaction, 19/20 use the existing synthetic-peer rendering companion, and 21–24 are ordinary native UI. Earlier interaction frames precede the final compact error-hint fix and outfit identity-preservation refinement; 01–10 and 23/24 reflect the final implementation. Saved choices were restored to approved defaults after verification.

| Capture | Backing pixels | Inspection / evidence type |
| --- | --- | --- |
| [01-field-right-aim.png](01-field-right-aim.png) | 2560×1440 | Injected native fixture: field right aim. |
| [02-field-left-crouch.png](02-field-left-crouch.png) | 2560×1440 | Injected native fixture: field left crouch. |
| [03-scout-right-jet.png](03-scout-right-jet.png) | 2560×1440 | Injected native fixture: scout right jet. |
| [04-scout-left-reload.png](04-scout-left-reload.png) | 2560×1440 | Injected native fixture: scout left reload. |
| [05-scout-right-death.png](05-scout-right-death.png) | 2560×1440 | Injected native fixture: scout right death. |
| [06-base-left-aim.png](06-base-left-aim.png) | 2560×1440 | Injected native fixture: base left aim. |
| [07-compact-field-preview.png](07-compact-field-preview.png) | 960×640 | Injected native fixture: compact field preview. |
| [08-compact-save-error.png](08-compact-save-error.png) | 960×640 | Injected native fixture: compact save error. |
| [09-desktop-beret-preview.png](09-desktop-beret-preview.png) | 2560×1440 | Injected native fixture: desktop beret preview. |
| [10-desktop-base-preview.png](10-desktop-base-preview.png) | 2560×1440 | Injected native fixture: desktop base preview. |
| [11-base-preview.png](11-base-preview.png) | 1280×720 | Ordinary mouse selection: Base, right-facing standing preview. |
| [12-field-left-crouch.png](12-field-left-crouch.png) | 1280×720 | Ordinary Field selection, facing and crouching preview controls. |
| [13-scout-deep-compact.png](13-scout-deep-compact.png) | 480×320 | Ordinary deep-skin Scout draft at 480x320; not yet applied. |
| [14-scout-after-restart.png](14-scout-after-restart.png) | 1280×720 | Ordinary app restart reloads the previously applied deep-skin Scout. |
| [15-blond-hair-jet-preview.png](15-blond-hair-jet-preview.png) | 1280×720 | Ordinary unsaved uncovered blond swept hair / live jet preview. |
| [16-left-upward-aim-preview.png](16-left-upward-aim-preview.png) | 1280×720 | Ordinary left-facing aim preview; same gameplay rig. |
| [17-left-reload-preview.png](17-left-reload-preview.png) | 1280×720 | Ordinary live reload preview, supporting hand/magazine articulation. |
| [18-death-preview.png](18-death-preview.png) | 1280×720 | Ordinary left-facing death preview, existing fold and fade. |
| [19-eight-online-baseline.png](19-eight-online-baseline.png) | 1280×720 | Existing localhost review server + seven synthetic peers; all eight rendered pilots use approved baseline. |
| [20-online-jets.png](20-online-jets.png) | 1280×720 | Same eight-slot companion; remote jet onset. Not eight human players. |
| [21-host-confirmation.png](21-host-confirmation.png) | 1280×720 | Ordinary two-client host stop confirmation. |
| [22-guest-disconnected.png](22-guest-disconnected.png) | 1280×720 | Guest error/menu after ordinary host shutdown; connection ended safely. |
| [23-outfit-retains-skin.png](23-outfit-retains-skin.png) | 1280×720 | Final-build ordinary switch from saved Scout to Field preserves deep skin and selected hair identity. |
| [24-final-compact-main.png](24-final-compact-main.png) | 480×320 | Final-build ordinary 480x320 main menu with six actions, after saving approved defaults. |

## Verification logs

Logs below preserve the run content with ANSI color escapes removed and a single final newline. They distinguish successful local checks, ordinary UI, injected fixtures and the canceled combat-script attempt.

- [workspace-tests.txt](workspace-tests.txt)
- [strict-clippy.txt](strict-clippy.txt)
- [host-build.txt](host-build.txt)
- [mac-arm64-build.txt](mac-arm64-build.txt)
- [headless-server.txt](headless-server.txt)
- [native-initial-interaction.txt](native-initial-interaction.txt)
- [native-restart-and-host.txt](native-restart-and-host.txt)
- [native-guest.txt](native-guest.txt)
- [eight-player-companion.txt](eight-player-companion.txt)
- [native-final-fixtures.txt](native-final-fixtures.txt)
- [native-final-ui.txt](native-final-ui.txt)
- [native-combat-canceled.txt](native-combat-canceled.txt)
- [preservation.txt](preservation.txt)

The combat attempt canceled on OS focus loss after initial firing frames; it is **not** a completed seven-checkpoint native route. Simulation/regression tests passed separately. The injected save error uses a permission-denied path-result fixture; actual filesystem failures and prior-file preservation are tested in the workspace suite. No long networking soak, human approval, Windows hardware or current-change CI result is inferred. The occasional Winit Destroyed close warning is retained in native logs.

See [handoff and five-step human checklist](../../handoffs/16-character-customization.md).
