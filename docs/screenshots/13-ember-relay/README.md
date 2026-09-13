# Ember Relay native screenshot evidence

2026-09-13, Apple M1 Pro / Metal. **20 nonblank, visually inspected native framebuffer captures**, lossless PPM→PNG conversion. No generated art, compositing, or gameplay-state inference from image pixels. See [handoff 13](../../handoffs/13-ember-relay-offline.md).

Images marked injected come from explicit initial fixtures followed by normal core commands in the real renderer. Traversal fixtures temporarily disable bot damage and are labeled “Bot respawning” in the HUD. They demonstrate rendered states and executed stages, not a human-controlled circuit or gameplay-feel approval. Desktop area captures preceded only the final help line-wrap and pause-heading polish; final-build compact captures repeat the routes with those corrections. Geometry/tuning is identical.

| Image | Logical viewport | Evidence |
| --- | --- | --- |
| [01-desktop-entry](01-desktop-entry.png) | 1280×720 | Ordinary final-build Ember play, S0/B0 and blocked initial sightline. |
| [02-crown](02-crown.png) | 1280×720 | Injected F08 crown landing and braking. |
| [03-central-steps](03-central-steps.png) | 1280×720 | Injected central ladder landing; independent stage. |
| [04-west-crouch](04-west-crouch.png) | 1280×720 | Injected held crouch beneath T01, west chamber. |
| [05-east-crouch](05-east-crouch.png) | 1280×720 | Injected held crouch beneath T02, east chamber. |
| [06-lower-gaps](06-lower-gaps.png) | 1280×720 | Injected lower east landing; U05 and genuine gaps visible. |
| [07-west-exit](07-west-exit.png) | 1280×720 | Injected M1 shaft return via U12. |
| [08-east-exit](08-east-exit.png) | 1280×720 | Injected M3 return via U11 to the east slope. |
| [09-outer-exit](09-outer-exit.png) | 1280×720 | Injected optional outer exit M4. |
| [10-recovered](10-recovered.png) | 1280×720 | Injected genuine fall recovered at S0: HP 57, ammo 7/48, kills 2; fuel includes normal regeneration. |
| [11-compact-menu](11-compact-menu.png) | 480×320 | Ordinary menu: five usable actions, 36px buttons, range backdrop. |
| [12-compact-help](12-compact-help.png) | 480×320 | Final-build ordinary Ember controls/help; center remains clear. |
| [13-compact-west-crouch](13-compact-west-crouch.png) | 480×320 | Final-build injected west sleeve traversal. |
| [14-compact-east-crouch](14-compact-east-crouch.png) | 480×320 | Final-build injected east sleeve traversal. |
| [15-compact-central](15-compact-central.png) | 480×320 | Final-build injected lower island stop between voids. |
| [16-compact-recovery](16-compact-recovery.png) | 480×320 | Final-build injected V1 recovery with retained HP/ammo. |
| [17-desktop-pause](17-desktop-pause.png) | 1280×720 | Final-build ordinary pause with corrected Ember heading. |
| [18-medium-pause](18-medium-pause.png) | 800×524 | Same paused Ember tick 1204 after resizing. |
| [19-host-confirmation](19-host-confirmation.png) | 480×320 | Ordinary range host with two actual clients: Keep Playing defaults. |
| [20-guest-disconnect](20-guest-disconnect.png) | 1280×720 | Actual native guest after confirmed host stop. |

The first attempted desktop-entry capture raced the pause action; it was replaced with a settled ordinary gameplay capture, not relabeled as gameplay. Every retained image is nonblank and checked visually. A temporary contact sheet was used for QA only; final files retain the framebuffer pixels.

## Logs

- [Native desktop traversal and subsequent local host interaction](evidence/native-desktop.log): 26 stage passes plus two recoveries; later teardown/rehost diagnostics retained.
- [Native final-build compact traversal](evidence/native-compact.log): 26 passes plus two recoveries, 480×320 after initial resize.
- [Ordinary final-build menu/input/resize](evidence/native-menu.log), [native guest](evidence/native-guest.log).
- [Measured 60/full-fuel routes](evidence/routes.txt), [135 workspace tests](evidence/workspace-tests.txt), [strict Clippy](evidence/clippy.txt), [host build](evidence/host-build.txt), [explicit Mac ARM64 build](evidence/mac-arm64-build.txt).
- [Core dependency tree](evidence/core-tree.txt), [server dependency tree](evidence/server-tree.txt).
- [Separate historical range combat-route attempt](evidence/range-route-canceled.log): canceled on focus loss; not a completed new seven-checkpoint run. Frozen range regression passes unchanged and ordinary range/Host/Join were directly rechecked.

Desktop hosting log includes ten rejected “packet is too small” datagrams and Winit window-destroy warnings during temporary-wrapper teardown/relaunch. The sender of those datagrams was not captured. The pinned transport rejects packets shorter than 18 bytes before decoding; observed guest departure/rejoin, stop/disconnection, and same-port rehosting succeeded. No transport fix, long soak, or claim of error-free networking is made. Final compact traversal log has no errors/warnings.

Human controls/feel, Windows compilation/hardware, second-machine LAN, cross-platform and real internet remain separate checks. See the handoff’s short human checklist.
