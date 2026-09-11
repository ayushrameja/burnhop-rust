# 03 — First playable combat loop

Implemented 2026-09-11 in `burnhop-native`; ready for review. The player can aim, fire a pistol or M416, reload, kill the stationary bot, take return fire, die and respawn. Both actors reset with F5. Approved movement tuning and the existing arena are unchanged.

The user reports completing the earlier Mac movement playtest and approving movement/controls with placeholder graphics. This **user-reported approval** is recorded in the movement handoff; it is not a claim that this agent performed a human playtest. Human combat feel, Windows compilation/hardware and GitHub CI remain unverified.

## Subsequent user-reported approval — 2026-09-11

The Agent 4 brief states that the user personally playtested and approved both movement and combat feel on Mac. This is **user-reported evidence**, separate from the original automated/native UI checks below. No individual held-mouse, Shift, cursor-exit, or focus subcase result was supplied, so those detailed physical checks are not inferred. Approved movement and weapon tuning are preserved in milestone 4. See [04-multiplayer.md](04-multiplayer.md) for current work; the original evidence below is historical.

## What changed

| File | Responsibility |
| --- | --- |
| `crates/gameplay-core/src/combat.rs` | Plain weapons/ammunition/health/life state, bot policy, rays, damage and practice step |
| `crates/gameplay-core/src/lib.rs` | Adds aim/fire/reload/selection fields to the command and exports combat types; movement rules remain unchanged |
| `crates/gameplay-core/tests/combat.rs` | 22 combat, lifecycle, collision, replay and movement-preservation tests |
| `crates/client/src/adapter.rs` | Buffers mouse/keyboard edges, aim point, reload/selection and cancellation |
| `crates/client/src/main.rs` | Device capture, focus pause, fixed ticks, lifecycle input clearing, camera/player/HUD integration |
| `crates/client/src/combat_view.rs` | Actual Bevy cursor projection, UI reticle, barrels, bot, health bar, confirmed tracers and hit flashes |
| `crates/client/src/playtest.rs` | Existing movement route plus seven-checkpoint combat route |
| `crates/client/src/tests.rs` | Native-message adapters, cursor projection, clearing and timing tests |

README controls, roadmap, reference notes and project context were updated. Existing uncommitted foundation/movement work, CI, Rust 1.98.1, Bevy 0.19.1 and Cargo.lock were preserved. No new dependency, map, external asset, audio, networking, account, service or engine was added. The browser working tree remains clean. No agents were spawned; nothing was committed, pushed, deployed or purchased.

## Shared-core contract

`step_practice(&mut World, &mut CombatState, InputCommand, &Arena)` advances one 60 Hz practice tick. A complete practice snapshot consists of **both** `World` and `CombatState`. These are plain, copyable Rust data. The core has zero dependencies, no Bevy types, device APIs, wall clocks or networking.

The existing `World`, `Player`, movement-only `step`, arena and movement constants retain their behavior. One existing test command literal gained `..Default::default()` to initialize the new fields; no movement assertions were weakened. A new 1,500-command test compares the practice wrapper's movement against the original step while the bot is inactive.

`InputCommand` adds `aim_at: Option<Vec2>`, `fire_held`, `reload_pressed`, and `select_weapon: Option<WeaponId>`. It retains tick, movement/jet/jump, reset and release-input fields. It now derives `PartialEq` instead of `Eq` because aim coordinates are floating point. `None`, non-finite or zero-length aim cannot shoot; the last valid direction is retained for facing. Coordinates remain f64 browser pixels, top-left rectangles, X right/Y down. The client feeds world aim points, not screen coordinates or Bevy vectors.

The step first checks the expected tick; a mismatch leaves **both** states untouched. Reset consumes one monotonically increasing tick, restores both actors, ammunition, fuel, counters and timers, ignores other commands, and requires neutral input before acting again. Normal ordering is life timers, living player movement, weapon timers/commands, both shot decisions, then damage. Simultaneous lethal shots are valid: neither actor gets an accidental first-in-loop advantage.

Death sets health to zero, cancels reload and movement intent, zeros velocity, and ignores movement, fire, reload and selection. Dead actors do not absorb or fire shots; corpses are dim cosmetic rectangles. After 180 ticks (3 s), the actor returns to its authored spawn with 100 HP and fresh loadout; the player also restores fuel/movement state. Respawn ignores that tick's input and requires a neutral/release-input tick before fresh actions. The client additionally clears observed/held/pending controls on death, respawn and F5, and snaps interpolation/camera on player respawn/reset.

## Weapon tuning and deliberate differences

[REFERENCE_GAMEPLAY.md](../REFERENCE_GAMEPLAY.md#combat-reference-for-milestone-3) records the actual browser values and source paths at commit `7a398d4abefa8144fa8949998de4cd76e5dacf8a`.

| Native rule | Pistol | M416 |
| --- | --- | --- |
| Trigger | Repeat while held | Repeat while held |
| Magazine + initial reserve | 12 + 48 | 30 + 120 |
| Fire interval | 12 ticks / 0.20 s | 7 ticks / ~0.117 s |
| Reload | 72 ticks / 1.2 s | 114 ticks / 1.9 s |
| Base whole-body damage | 18 | 23 |
| Falloff start → end / minimum factor | 300 → 800 / 0.4 | 800 → 1600 / 0.8 |
| Range | 1000 pixels | 1900 pixels |

Both browser weapons actually repeat on hold, including the pistol. No guessed semi-automatic behavior was introduced. Damage keeps the browser's rounded linear falloff. A spawn-to-spawn pistol hit does 14 damage; an M416 hit does 23. The bot uses the same pistol rules with unlimited reserve and a slower attack policy.

Timers decrement once per simulation tick before processing actions. A reload starts with its full duration, transfers only available reserve at completion, and can fire on that completion tick when held. Empty magazines never auto-reload for the player. Repeated reload edges cannot restart or chain a reload while the same press remains active. Switching cancels the old reload without granting ammunition; it preserves each weapon's ammunition/cooldown and adds an 18-tick (0.3 s) equip delay. Off-slot cooldowns keep counting down. Re-selecting the same slot does not reset its delay; toggling slots cannot bypass cooldowns.

Intentional simplifications:

- Player reserves are finite to make magazine/reserve/reload behavior visible; both reference weapons have unlimited reserves. Player respawn or F5 refills both weapons.
- Exact aim without recoil, spread, bloom or stance modifiers. The reference values are documented for later evidence-led tuning. This is not recoil-feel parity.
- One 36 × 68 body hitbox with no head/leg multipliers. No dual wield, melee, pickups, inventory instances, weapon-dependent camera zoom or collision between actors.
- Rays start at body center, about 3.14 pixels below the browser's standing weapon origin. Barrel graphics are directional indicators; they do not define hits.
- Both native respawns use 180 ticks; the browser practice target uses 120 ticks and does not attack.

## Hits, cover and opponent behavior

These weapons are **hitscan** in the reference: their entire ray is resolved in the firing tick. There is no authoritative traveling bullet or projectile-speed setting. Yellow player and red bot tracers are brief cosmetic feedback. A whole-segment slab test finds the closest terrain/body intersection, so even a ray crossing a million pixels cannot jump over a 0.01-pixel wall. This is long-ray coverage, not a claim to have implemented or tested a moving-projectile system.

Cover wins equal or near-equal distances (1e-8 tolerance); bodies must be strictly closer. Body ties use stable actor identity. The shooter is explicitly excluded. Ray tests start inside the shooter's body, not at the end of the barrel, so protruding barrels cannot skip nearby walls. Damage and confirmed hit positions come only from the core. Tracer lifetime and flash colors cannot change them.

The bot stands at the existing target spawn (910,1152); player spawn is (390,1152). It aims at the player's body center and attempts at most one pistol shot every 60 ticks, only with a living target, line of sight and sufficient range. It waits 180 ticks after reset or either respawn. It reloads an empty magazine using the same weapon timer, delaying fire until ready. It has no movement, pathfinding, tactics, random behavior or match system. Spawn grace restrains the bot's attacks; it is not general invulnerability. The practice bot spawn is specific to this arena; arbitrary map loading/spawn validation remains deferred.

## Native input, aim and presentation

Left mouse fires, R reloads, 1/2 select pistol/M416, and **F5 replaces the old R reset**. A/D, Space and both Shift keys retain their approved controls. The ordered input buffer ignores duplicate states and OS keyboard repeats, quantizes short clicks to one held tick, and consumes reload/selection edges once across catch-up ticks.

Aim uses Bevy's actual `Camera::viewport_to_world_2d` and camera `GlobalTransform` from the last displayed frame; it converts the resulting Y-up position to the core's Y-down convention. Logical window coordinates handle Retina scaling. See the [official Bevy projection API](https://docs.rs/bevy/0.19.1/bevy/camera/struct.Camera.html#method.viewport_to_world_2d) and bundled 0.19.1 source. There is no hardcoded 1280 × 720 pointer mapping. If the window size and cached viewport disagree during resize, firing is temporarily canceled until Bevy refreshes the projection (normally the next frame); click again afterward.

The reticle is a small screen-space cross, so it stays under the pointer while the camera follows. The weapon direction follows the core aim; body presentation retains existing interpolation. Native cursor exit clears firing and pending fire edges without interrupting keyboard movement. Re-entry requires a fresh mouse press. Focus loss pauses simulation/accumulation and clears all input; on return, a release-input tick cancels core intent before fresh controls. In-progress reload/cooldown timers remain paused with the rest of the encounter.

The HUD shows player health, weapon, magazine/reserve, reload/equip/empty status, fuel, death/respawn, bot health/grace/respawn and kill/death counters. Bot health bar, dim corpses, short tracers and white hit/impact flashes are placeholders. No screen shake or audio. Small windows wrap the HUD; it occupies more of the view at the 480 × 320 minimum.

## Verification and its limits

Host: Apple M1 Pro, Apple Silicon macOS 26.6.2, native `aarch64-apple-darwin`. Both test wrappers reported the Apple M1 Pro **Metal** renderer. The earlier handoff contains hardware details; no new performance claim is made.

| Check | Observed result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 17 preserved movement + 22 combat + 15 client tests = **54** |
| `cargo build --workspace --locked` | Passed; Mach-O arm64 executable |
| `cargo check -p burnhop-gameplay-core --locked` / dependency tree | Passed independently; zero dependencies |
| Repeatable replay | 10,000 combat commands produce equal world/combat/events on this machine; existing 10,000-command movement replay also passes |
| Cursor conversion | Actual Bevy orthographic projection + inverse and round-trip, translated cameras, four window/aspect sizes, 1×/2× scale, invalid cursor and stale resize viewport |
| Device adapter | Actual Bevy keyboard/mouse/focus messages verify Shift, held left fire, R, repeat rejection and clearing; ordered-buffer tests cover taps, selection, reset, cursor exit and respawn |
| Native UI actions | Mouse clicks aimed and hit with both guns, R reload, 1/2 selection, F5 reset, A/D/Space input, aiming during a jump, bot kills, incoming damage, player death and both respawns |
| Native resize | Actual window inspected at 1280 × 720, about 800 × 530, and 480 × 320 client area; HUD wraps and fresh mouse shots hit after resize |
| Native camera follow | Observed the camera follow native D taps; moving fire is additionally covered by the rendered scripted route. Human sustained moving-aim feel is pending |
| Native focus recovery | Finder took focus; game visibly displayed PAUSED. Loss/return logs retained identical ticks 10677 and 11141. Fresh firing and R worked after return; no elapsed combat during pause |
| Rendered combat route | All **seven** checkpoints passed in the actual Metal-rendered client, using the same input buffer/core as ordinary play |
| Historical movement route | Its original eight-checkpoint test still passes headlessly; no claim of a new physical Shift playtest |
| Native close | Closed both test windows; an unsandboxed `pgrep` check found no remaining game process |
| Human combat playtest / long physical holds | **Not performed**. Native UI automation is not a human feel verdict |
| Windows compilation / Windows hardware | **Not performed** |
| GitHub CI | **Not run**. Existing workflow preserved; no commit, push or dispatch |

The seven native route checkpoints were pistol held-fire kill, pistol reload, bot respawn, M416 held-fire kill while moving, M416 reload, player death/respawn from bot fire, and complete reset. It injects tick-aligned aim and button state, not fixture damage or teleportation. Run `cargo run -p burnhop-client --locked -- --combat-playtest` and keep the window focused for about 22 seconds. It prints `COMBAT PLAYTEST COMPLETE`, then returns device control. Real keys/clicks or focus loss cancel the route. The old `--movement-playtest` suppresses combat only while its movement route is active.

Native mouse clicks and keyboard presses came through the UI automation tool. The tool provides no separate mouse-down/up or controlled-duration hold API; a short drag produced only one sampled fire tick. Thus sustained automatic fire is proven by core tests and the rendered route, **not** by a claimed physical mouse hold. Cursor exit while held, app switching while holding mouse/Shift, and human aiming feel still need the manual checklist.

Ignored wrappers `target/smoke/Burnhop Combat Check.app` and `Burnhop Combat Route.app` run the ordinary built binary with `--combat-diagnostics` or `--combat-playtest`; evidence is in `target/smoke/combat-check.log` and `combat-route.log`. They are local inspection conveniences, not signed distribution packaging. Normal close emitted the same existing Bevy/Winit `Skipped event Destroyed for unknown winit Window Id` warning; no gameplay/render failure was observed. No performance, networking-latency or cross-platform determinism claim follows from these checks.

## User playtest checklist — about five minutes

1. Run `cargo run -p burnhop-client --locked`. Aim at the red bot and hold left mouse with pistol, then M416. Both repeat; the rifle is faster. Move/jump/jet while tracking the bot and check that the reticle and shots feel aligned.
2. Fire part of a magazine; press R and try firing during reload. Switch 1/2 mid-reload, switch back, and verify no free ammunition or instant shot. Empty a magazine and reload manually.
3. Shoot a platform or use it as cover. Check that impacts stop at the surface and the bot cannot hit through it. Stand still to take damage, die and respawn; confirm fresh health, fuel and ammunition.
4. Hold movement/fire through death and through F5. Nothing should resume until released and pressed again. Resize, move the cursor outside while firing, then return and click afresh. Switch apps while holding fire or Shift; release outside and return. Check for stale fire/thrust or aim jumps.
5. Report one concrete action and observation, especially aim feel or bot pressure. Movement is already approved; tune combat from that evidence before adding more weapons or multiplayer.

Next: human combat feedback, then Windows build/hardware and CI validation as separate evidence. The shared core is prepared for later reuse; no networking transport, server overload policy, serialized snapshot protocol or match architecture was selected here.
