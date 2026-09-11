# 02 — Movement playground handoff

Implemented 2026-09-11 in `burnhop-native`. One arena is playable and ready for review. No combat or multiplayer was added. At the original handoff, human movement-feel validation, physical Shift-key testing, Windows compilation/hardware and GitHub CI were outstanding. See the subsequent user approval below.

## Subsequent user-reported approval — 2026-09-11

The user reports completing the requested Mac playtest and confirms that movement and controls feel correct with placeholder graphics. This is **user-reported approval**, recorded from the Agent 3 brief; it is separate from the agent's earlier scripted/native UI checks. The brief does not enumerate every physical Shift/focus subcase, so no more detailed hardware claim is inferred. Movement tuning is preserved for combat. The historical evidence below remains unchanged; the current reset key is **F5**, since combat assigns R to reload. Human combat playtesting, Windows compilation/hardware and GitHub CI remain separate pending work.

## What works

- The practice range retains the browser's 2400 × 1350 dimensions, floor at Y=1220, four rectangular platforms and player top-left spawn (390,1152). The 36 × 68 player starts supported on the floor.
- A/D accelerate and brake; Space jumps with ledge grace and buffered landings; either Shift directly powers the jet from ground or air. Fuel tapers thrust, drains, regenerates after a delay, and requires a new press after exhaustion or landing.
- Continuous rectangular collision stops floors, walls, ceilings and corners, slides along free axes, and checks support again after leaving an edge.
- The resizable native client has interpolation, bounded following camera, controls/fuel HUD, thrust indicator, spawn mark, R reset, and focus pause/resume with input clearing.

## Architecture and simulation contract

| File | Responsibility |
| --- | --- |
| `crates/gameplay-core/src/lib.rs` | Arena, plain player/world state, explicit commands, tuning and one-tick movement rules |
| `crates/gameplay-core/src/collision.rs` | Swept rectangle contact and final standing support |
| `crates/gameplay-core/tests/movement.rs` | 17 behavior and replay tests |
| `crates/client/src/adapter.rs` | Ordered device-edge queue, bounded frame clock and camera math |
| `crates/client/src/main.rs` | Bevy keyboard/focus adapter, simulation scheduling, sprite/UI rendering and camera |
| `crates/client/src/tests.rs` | Input/frame/camera tests, including actual Bevy keyboard/focus messages |
| `crates/client/src/playtest.rs` | Opt-in rendered route and its checkpoint test |

The core has **zero dependencies**. It contains no Bevy types, wall clocks, device input, file loading or network transport. Rendering and collision both consume `PRACTICE_ARENA.solids`, including the browser's boundary rectangles; decorative edge highlights do not define a second collision map.

Coordinates and units are explicit: top-left rectangles, X right, Y down, world pixels, pixels/second, pixels/second², and named integer tick timers. Simulation numbers are `f64`, matching JavaScript's number precision. The client converts a rectangle to sprite center `(x + width/2, -(y + height/2))` and casts only presentation values to `f32`. No claim of cross-platform floating-point determinism follows from this choice.

`step(&mut World, InputCommand, &Arena)` advances exactly 1/60 second. `InputCommand.tick` must equal `World.tick`; a mismatch returns `TickMismatch` without mutation. Input uses a left/idle/right enum, a jump edge, jet edge/hold, reset, and release-input cancellation. There is only one player, so actor routing is deferred. Reset consumes its tick, restores every player field and ignores other commands in that tick; the world tick stays monotonic. State includes velocity, support, fuel, grace/buffer/delay counters and the thrust latch. Returned flags report jump, landing and reset.

The arena is compiled constant data for this milestone. Collision requires finite, positive-sized rectangles and a non-overlapping starting body. Spawn is validated by tests. Arbitrary map import/validation and recovery from externally injected overlapping states are not implemented.

## Movement choices and reference fidelity

Source reviewed at browser commit `7a398d4abefa8144fa8949998de4cd76e5dacf8a`: `src/game/simulation.ts`, `collision.ts`, `timing.ts`, `input.ts`, `controls.ts`, relevant movement/input tests, and `public/assets/arena.json`. The browser working tree stayed clean.

| Rule | Retained value/behavior |
| --- | --- |
| Horizontal cap | 320 pixels/s |
| Ground/air acceleration | 3800 / 2300 pixels/s² |
| Ground/air braking | 4200 / 320 pixels/s² |
| Gravity / jump impulse | 1500 pixels/s² / -520 pixels/s |
| Sustained jet rise / fall caps | -480 / +740 pixels/s |
| Ledge grace / landing buffer | 8 / 9 tick counters, preserving the reference update order |
| Fuel | 100 capacity; drain 40/1.4 per second; regen 30 per second after 24 ticks |
| Jet force | 3600 at full fuel, tapering below 90 fuel toward 1800; sampled at the tick's fuel midpoint |

Both browser control modes were inspected. In combined mode, the first grounded Space press jumps without becoming thrust; a fresh airborne press can latch thrust; a missed buffered landing may become thrust while held. **This client implements the browser's current default separate hold mode** (`defaultControls()`): Space only jumps/buffers; Shift takes off directly. A failed/missed Space buffer never becomes jet thrust. Combined mode, toggles, rebinding and crouch are deferred.

The landing buffer probes nine actual horizontal/vertical swept movements; being near a platform is insufficient. Releasing Space preserves a queued hop. Ledge grace is consumed by a jump or direct jet takeoff. Holding Space never autojumps. A simultaneous jump+jet press retains the initial jump impulse before the sustained-flight cap applies on later ticks. Landing cancels jet intent. Exhaustion uses only the remaining fraction of a tick's fuel, then clears the latch; holding Shift through regeneration does not restart thrust. Releasing Shift stops force immediately on its simulation release tick but preserves momentum.

The jump apex is 85.8333 pixels above spawn under the reference semi-implicit 60 Hz integration (first vertical speed -495). Platforms are two-sided solids; jump alone cannot reach the first platform 260 pixels above the floor. Jet around its edge to land on top.

## Why collision does not tunnel

Each movement is swept through its entire displacement against every rectangle. The solver finds the earliest impact, moves to it, blocks impacted axes and spends the remainder sliding along free axes. Equal-time normals from all solids are combined so a floor/wall corner is independent of solid ordering. Each impact removes at least one moving axis; three passes cover two blocked axes plus any free remainder. Touching tangentially and instantaneous corner grazes are legal. A final strict horizontal support check prevents staying grounded after moving beyond a platform edge. Tests sweep 10,000 pixels through a one-pixel platform from all four sides, far beyond normal movement caps.

No polygon/slope solver, one-way platforms, crouch clearance or Outpost artwork was brought into this milestone.

## Client timing, input and presentation

- Raw Bevy `KeyboardInput` messages feed an ordered edge queue; OS repeats are ignored. A press/release pair between simulation ticks lasts at least one tick, including movement and jet taps. Releases remain queued in order; a press is not repeated across catch-up ticks. Both Shift keys form one aggregate hold. Queue overflow clears intent rather than growing without bound.
- The local frame clock admits at most 0.1 seconds, executes at most five 60 Hz ticks, then drops excess whole-tick backlog. This deliberately slows local simulation after a stall instead of attempting unbounded catch-up. It is **not** a future authoritative-server overload policy.
- Focus transitions discard pending/held input. While unfocused, the simulation and accumulator pause. On return, an explicit release-input tick clears core buffers/latches before normal controls resume. The player keeps physical momentum; it is not teleported or reset. Inputs arriving in the focus-transition frame are deliberately discarded to avoid synthetic/stale key state. Press controls again after clicking back.
- R restores spawn/fuel/timers, clears held/queued controls and snaps previous/current rendering and camera together, preventing an interpolated trail across the arena.
- Rendering interpolates previous/current positions, adding up to one tick of presentation delay. Camera follows this position with reference exponential rates 20/24 and lag caps 24/32, and presentation time capped at 0.06 seconds.
- Normal framing is 1280 × 720 world units. Resize preserves aspect ratio and bounds the view to the arena; camera Y bounds extend to `floor_y + 95`, as in the reference. The minimum client area is 480 × 320. HUD text wraps at smaller widths. Weapon/view-tier zoom is deferred.

Intentional differences: only separate controls are offered; sub-tick taps are explicitly quantized to one tick (including a very short jet tap); there is one fixed camera framing; R means reset rather than the browser's combat reload action. No artwork or runtime JSON files were copied.

Rust remains **1.98.1** and Bevy remains **0.19.1**. Bevy UI/rendering and its bundled default font were enabled for a legible HUD, adding 49 locked transitive packages. Existing pinned versions were preserved, and no third-party physics dependency was added.

## Verification performed

Host: Apple M1 Pro, macOS 26.6.2 (25G83), `aarch64-apple-darwin`. The actual renderer reported the Apple M1 Pro Metal adapter. This is the same host as the foundation handoff, not a new hardware performance benchmark.

| Check | Observed result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings |
| `cargo test --workspace --locked` | Passed: 17 core + 10 client tests (27 total); no window required |
| `cargo build --workspace --locked` | Passed; resulting executable is Mach-O arm64 |
| `cargo check -p burnhop-gameplay-core --locked` | Passed independently |
| `cargo tree -p burnhop-gameplay-core --locked` | Only the core package; zero dependencies |
| Same-machine replay | 10,000 identical tick commands produce exactly equal state/events, with arena bounds/interior-overlap checks each tick |
| Presentation schedules | Equal commands/state at 30/60/144 Hz and jittered frames for event times aligned to shared frame boundaries |
| Actual native renderer | Opened and visually inspected arena, player, camera movement and controls/fuel HUD |
| Rendered scripted route | All eight checkpoints passed in the actual running Metal-rendered client; details below |
| Native keyboard taps | A/D moved; Space produced first-tick Y=1143.75, VY=-495 and a later floor landing; R restored (390,1152), fuel=100 and all latches/timers |
| Resize | Visually inspected around 800 × 560 and the 480 × 320 minimum client area; text wrapped and arena/player remained visible |
| Focus loss/return | Native window visibly changed to PAUSED; loss/return logs had identical tick 6960 and position (390,1152); release-input command on resume; fresh A/Space worked and landed afterward |
| Native close | Clicked the close button and confirmed no game process remained |
| Physical Shift hold/release | **Not verified with real hardware input.** UI automation rejects standalone modifier presses; its Shift+A chord emitted A events but no Shift keyboard event. Bevy-message adapter test and rendered scripted thrust passed separately |
| Human feel / live side-by-side browser comparison | **Not performed**; source tuning and numerical trajectory were checked |
| Windows compilation | **Not performed** |
| Windows hardware launch/playtest | **Not performed** |
| GitHub CI | **Not run**; foundation workflow preserved; no commit/push/dispatch |

The eight rendered checkpoints were: platform underside (Y=1002), floor landing (Y=1152), arena ceiling plus empty fuel (Y=0, fuel=0), held jet remaining off after regeneration (fuel=18.5), fresh-press reactivation, right wall (X=2364, VX=0), first-platform landing (Y=892), and complete reset to spawn. The route feeds the same client `InputBuffer`; it does not teleport the actor to contact fixtures. Automated logs establish simulation outcomes in the running renderer; screenshots do not constitute human perception of every intermediate frame.

Native inspection used ignored local app wrappers under `target/smoke/Burnhop Movement*.app`, pointing at the ordinary built binary. The check wrapper passes `--movement-playtest` and logs to `target/smoke/movement-check.log`. This is an inspection convenience, not signing, distribution packaging or a runtime dependency. The supplied route is opt-in, takes about 21 focused seconds, leaves keyboard control available afterward, and cancels safely on real input or focus loss. Default runs do not log scripted diagnostics. After a final review fix to drain all focus events in a frame (covered by the Bevy-message regression test), the rebuilt default client was reopened, visually checked, exercised with D/Space and closed. Process absence was confirmed after closure.

The schedule tests deliberately use events at common frame boundaries. Arbitrary OS-event delivery can quantize to different next ticks at different refresh rates, and dropped long-gap time intentionally differs. None of these same-machine tests establishes Mac/Windows determinism, GPU performance, networking latency or human movement feel.

## Manual feel checklist (about five minutes)

1. Run `cargo run -p burnhop-client --locked`. Walk and reverse with A/D. Release: ground stopping should be quick; air momentum should last longer.
2. Tap then hold Space. There should be one roughly 86-pixel hop and no automatic thrust/repeated landing jump. Try a late jump just after walking off a platform.
3. Hold Shift from the floor, then release in flight. Thrust should cut while momentum continues. Fly around the first platform edge, hit its underside/side, then land on top; inspect for snagging or sinking.
4. In the gap between the second and third platforms, fly into the arena ceiling. Drain the tank fully, keep holding Shift as fuel returns, then release/repress. It should restart only on the fresh press.
5. While descending toward a platform, tap Space just before touchdown. Expect a queued hop; steer clear of the edge and confirm a missed landing does not become a jet.
6. Press R while moving/thrusting. Expect a clean spawn reset. Resize the window. Switch apps **while holding A/D or Shift**, release outside the game, return, and check for stuck motion/thrust. Fresh controls should work.

Record what feels wrong as an action and observation (for example, “air braking takes too long after releasing D”), then tune one constant at a time. This is the original checklist. The subsequent user approval above supersedes the earlier general movement-feel pending status; individual subcase results were not supplied.

## Remaining limits and next steps

Normal close of the final default client emitted one Bevy/Winit warning, `Skipped event Destroyed for unknown winit Window Id`, then exited; process absence was confirmed. A similar shutdown warning was recorded during foundation validation. No startup/render failure or lingering game process was observed; investigate if later platform playtests show a shutdown problem.

The user subsequently approved movement and requested the combat milestone; see [03-combat.md](03-combat.md). Windows build checks, Windows hardware playtesting and GitHub CI still require separate evidence. Future map import must validate whole-body spawn clearance and add polygon support deliberately; future networking must decide serialization/transport and server timing independently of the local frame-gap policy.

The browser project was left untouched. Pre-existing uncommitted foundation files, lockfile lineage, CI and tooling were preserved. README, roadmap and project context now describe movement; the foundation handoff remains historical. No agents were spawned; nothing was committed, pushed, deployed or purchased.
