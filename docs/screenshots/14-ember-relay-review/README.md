# Ember Relay P3 correction — native evidence

2026-09-13. Actual rebuilt native client on Apple M1 Pro / Metal, using the existing opt-in `--ember-playtest` route in a separate temporary application wrapper. Only the U04 decorative accent changed in rendering. These are native framebuffer PPM captures converted losslessly to PNG; no crop, compositing or generated artwork. Both PNGs were visually inspected.

| Capture | Logical viewport | Scene |
| --- | --- | --- |
| [West lip desktop](01-west-lip-desktop.png) | 1280×720 | `lower-island-west`, tick 1882; the pilot stands just left of the corrected amber strip, which follows U04's slope into V1. |
| [West lip compact](02-west-lip-compact.png) | 480×320 | Same injected route fixture, tick 1879; the accent stays on the slope, with the gap open and essential HUD legible. |

The original native logs are retained byte-for-byte as text evidence (to remain eligible for the save checkpoint) in [desktop](native-desktop.txt) and [compact](native-compact.txt). They identify Metal, viewport sizes and capture names/ticks. Preview runs were stopped after the relevant lower-route captures, rather than completing the full 26-stage suite. The desktop route was canceled with device input after stage 12, followed by ordinary movement and one fall recovery; the compact preview was paused after stage 13, then closed. Neither is claimed as a complete native traversal rerun. Both temporary previews were closed. Each log contains the previously observed Winit unknown-window Destroyed warning at close; no error or panic is logged.

The route injects initial fixtures and disables the bot during traversal, hence “Bot respawning” in the HUD. Screenshots demonstrate presentation, not a new human feel/balance approval. The user's earlier offline approval is preserved. See [review/fix handoff 14](../../handoffs/14-ember-relay-review.md).
