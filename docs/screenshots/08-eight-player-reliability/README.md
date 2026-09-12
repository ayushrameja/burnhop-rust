# Native eight-player review — 2026-09-12

Actual Bevy 0.19.1 / Metal rendering on Apple M1 Pro, macOS 26.6.2. PNGs are lossless conversions of native framebuffer PPM captures; no composition or generated imagery. Desktop captures are 1280 ×720 logical / 2560 ×1440 framebuffer; compact is 480 ×320 logical / 960 ×640 framebuffer. Milestone 08 reported that images were opened and visually inspected. Checkpoint 09 independently inspected the desktop and compact scoreboards and found the supplied image 01 was entirely black; the correction below supersedes its original evidence claim.

One real native client rendered its local P1 and seven synthetic UDP peers from the shared approved pilot rig. The companion drove ordinary inputs to gather pilots, fire jets, depart/rejoin, then fight. It did not teleport bodies or alter gameplay. Native input was neutral when unfocused; this is not eight-human play. `BURNHOP_REVIEW_TAB=1` was used for the persistent scoreboard captures. A Bevy system test separately verifies actual Tab pressed/released, focus loss, disconnect and preservation of movement intent.

- 01 — excluded at checkpoint review: the supplied `01-eight-pilots.png` decoded to an entirely black framebuffer. Its original bytes were preserved outside Git at `/tmp/burnhop-checkpoint09-excluded/01-eight-pilots.png`; it is not rendering evidence. Image 02 independently shows all eight pilots with stable identities.
- [02 — desktop scoreboard](02-scoreboard-desktop.png): eight authoritative identity/kill/death rows and ordinary edge HUD.
- [03 — minimum-size scoreboard](03-scoreboard-compact.png): all eight rows readable with essential HP/fuel/ammo still at edges.
- [04 — remote jets](04-remote-jets.png): multiple pairs of cyan exhaust cones; eight pilots remain identifiable.
- [05 — departure](05-departure.png): P2 disappears, seven rows and 7/8 population; no leftover pilot at its former position.
- [06 — replacement handshake](06-replacement.png): eight rows return with fresh zero scores. The replacement is initially off-screen at its selected distant spawn; this image alone does not show its rig.
- [07 — combat death](07-death.png): authoritative death count, folded/fading rigs, inactive weapon and three-second return UI.
- [08 — impact](08-impact.png): confirmed tracer, origin flash and terrain endpoint sparks, bounded pools.
- [09 — respawn](09-respawn.png): restored local rig/health at selected spawn, updated camera and other combatants' weapons/effects.
- [10 — offline rifle fire](10-offline-rifle-fire.png) and [11 — offline respawn](11-offline-respawn.png): approved practice rendering and bot behavior. Its native scripted combat route passed all seven checkpoints.
- [12 — reused pilot visible](12-reused-pilot-visible.png): a separate repeat assigned the departed/reused slot P3, now back on-screen with fresh health/pose and zero scores. Generation changes are logged, not printed as HUD implementation details. It is walking back through the lineup; no actor collision was added.

Slot assignment follows handshake order; the seventh synthetic process is not guaranteed P8. The departure/replacement screenshot pairs and repeat logs explicitly reflect this. Native captures check presentation; real UDP lifecycle tests also replace every slot and verify fresh identity/state, while per-actor tests cover interpolation, effects and skipped death cycles. The source review aid captures only a bounded set of transition names plus twenty manual F9 images. See the [handoff](../../handoffs/08-eight-player-reliability.md) and [measurements](../../measurements/08-eight-player/README.md).
