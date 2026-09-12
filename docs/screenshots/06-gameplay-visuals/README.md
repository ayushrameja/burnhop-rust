# Gameplay visuals — native Metal captures

2026-09-12, Apple M1 Pro / macOS 26.6.2, 2× Retina. Actual native framebuffer captures, not mockups. PPM framebuffers were losslessly encoded as PNG; no image-generation assets or altered scene content. The original six images total about 545 KiB; updated compact-HUD review evidence is below. See [the handoff](../../handoffs/06-gameplay-visuals.md) for methods and limitations.

| Capture | Evidence |
| --- | --- |
| [01 — Field range](01-field-range.png) | Final practice spawn, pilot, terrain and HUD; 1280 × 720 logical. |
| [02 — Online jets](02-online-jets.png) | Player Two is local/cyan; remote One is ochre. Both boots show network-driven exhaust. |
| [03 — Rifle reload](03-rifle-reload.png) | Native mouse shot and R reload; magazine/reserve and authoritative reload feedback. |
| [04 — Confirmed rifle hit](04-confirmed-rifle-hit.png) | Online Player Two firing left, confirmed tracer and opponent hit tint. |
| [05 — Death / respawn](05-death-respawn.png) | Practice death pose, zero health and respawn countdown. |
| [06 — Minimum viewport](06-minimum-window.png) | Historical 480 × 320 capture that exposed the compact-HUD obstruction; superseded by 07–14. |

## Compact-HUD follow-up — 2026-09-12

Sixteen additional native Metal framebuffer captures at 2× Retina. These retain original scene pixels and are lossless PPM-to-PNG conversions. Exact logical sizes and ordinary input routes were set in a temporary inspection copy; no camera, simulation, artwork or image content was changed. The original six files remain untouched. See the [handoff follow-up](../../handoffs/06-gameplay-visuals.md#compact-hud-review-follow-up--2026-09-12) for native checks, automated checks and limits.

| Capture | Evidence |
| --- | --- |
| [07-minimum-platform](07-minimum-platform.png) | 480 × 320: platform landing, both pilots visible; replaces 06 as current compact evidence. |
| [08-minimum-left-jet](08-minimum-left-jet.png) | 480 × 320: native jet along the left edge, x=0 near y=699. |
| [09-minimum-right-jet](09-minimum-right-jet.png) | 480 × 320: native jet along the right edge, x=2364 near y=699. |
| [10-minimum-ceiling-empty](10-minimum-ceiling-empty.png) | 480 × 320: ceiling traversal and explicit empty fuel; middle aiming lane stays open. |
| [11-minimum-controls](11-minimum-controls.png) | 480 × 320: F1 guide explicitly revealed; defaults closed and F1 hides it. |
| [12-minimum-reload](12-minimum-reload.png) | 480 × 320: M416 magazine/reserve and reload countdown. |
| [13-minimum-death](13-minimum-death.png) | 480 × 320: HP 0 DOWN, inactive weapon, death pose and respawn countdown. |
| [14-minimum-connection-error](14-minimum-connection-error.png) | 480 × 320: actual five-second connection failure with complete wrapped reason and restart guidance. |
| [15-medium-platform](15-medium-platform.png) | 800 × 524: platform landing with compact HUD and clear pilots. |
| [16-medium-reload](16-medium-reload.png) | 800 × 524: real rifle reload feedback. |
| [17-medium-connection-error](17-medium-connection-error.png) | 800 × 524: complete actual connection failure. |
| [18-desktop-platform](18-desktop-platform.png) | 1280 × 720: platform landing, existing desktop layout retained. |
| [19-desktop-reload](19-desktop-reload.png) | 1280 × 720: rifle reload in the unchanged desktop HUD. |
| [20-desktop-connection-error](20-desktop-connection-error.png) | 1280 × 720: full reason and restart guidance in the existing desktop layout. |
| [21-medium-left-jet](21-medium-left-jet.png) | 800 × 524: native jet along the left screen edge. |
| [22-desktop-right-jet](22-desktop-right-jet.png) | 1280 × 720: native jet along the right screen edge. |

New captures: 920 KiB. Total: 22 PNGs.
