# Roadmap

## Completed

- [x] Create a separate project folder and local Git repository.
- [x] Record the agreed direction and agent continuity instructions.
- [x] Commit and normally push the reviewed foundation-through-multiplayer implementation and test-only CI portability fix to the existing repository on `main`.

## 1. Native foundation and movement

- [x] Inspect the reference movement, camera, collision, and map data; see `docs/REFERENCE_GAMEPLAY.md`.
- [x] Verify and pin Rust 1.98.1 / Bevy 0.19.1; create native client and gameplay-core workspace crates.
- [x] Establish a dependency-free gameplay-core boundary; simulation itself is deferred.
- [x] Add formatting, Clippy, locked builds, and macOS ARM64 / Windows x64 CI configuration (execution results are in the checkpoint handoff).
- [x] Compile, visually inspect the native placeholder window, and verify close on Apple Silicon macOS (M1 Pro, macOS 26.6.2; local inspection wrapper).
- [ ] Build and open the game on Windows hardware; CI is not a hardware playtest.
- [x] Add the reference practice arena, controllable character, jump/ledge grace/buffer, direct jet/fuel, swept rectangular collision, interpolation and bounded following camera.
- [x] Add 17 shared-core and 10 client input/timing/route tests; local formatting, strict Clippy, tests and locked build pass.
- [x] Run the actual Mac renderer through eight scripted movement checkpoints; exercise native A/D, Space, R, resize, focus pause/resume and close. See the movement handoff for the physical Shift testing limitation.
- [x] Record user-reported Mac movement/control approval with placeholder graphics (2026-09-11); see the movement handoff addendum. Individual physical Shift/focus subcase results were not supplied; agent verification remains separately described.
- [x] Run GitHub CI and verify Windows compilation separately from Windows hardware playtesting; both targets pass at `de70455`, with exact run links in [05-checkpoint.md](handoffs/05-checkpoint.md).

## 2. Small combat loop

- [x] Add pistol/M416 held fire, tick timers, finite ammunition, reload/switching, authoritative rays/damage and both actors' death/respawn.
- [x] Add camera-projected mouse aim, reticle/barrels, health/ammo/fuel/reload/life HUD, confirmed shot/impact flashes, stationary attacking bot and F5 reset.
- [x] Verify 54 tests, formatting, strict Clippy, locked Mac build and zero-dependency core; preserve all 17 movement tests and movement tuning.
- [x] Run seven scripted combat checkpoints in the actual Mac Metal renderer; exercise native clicks, R, 1/2, F5, movement, damage/death/respawn, resize and focus recovery.
- [x] Record user-reported Mac combat-feel approval from the Agent 4 brief; see [03-combat.md](handoffs/03-combat.md). Individual physical held-input/focus subcases were not supplied and remain distinct from this approval.
- [x] Validate Windows compilation and GitHub CI with an actual passing Windows runner job at `de70455`; no success is inferred from Mac checks.
- [ ] Validate combat on Windows hardware/GPU.

## 3. Authoritative multiplayer

- [x] Select and pin Renet 2.0.0 / renet_netcode 2.0.0 after official documentation and source review.
- [x] Generalize dependency-free actor rules and preserve offline practice through an adapter and pre-refactor trace check.
- [x] Run one headless Rust server at 60 Hz with bounded overload, two assigned actors, shared combat and no bot.
- [x] Connect two native clients with movement prediction/reconciliation, authoritative combat, remote interpolation and explicit connection states.
- [x] Validate ownership, compatibility, malformed/stale/duplicate input, bounds, missing input, lifecycle, effects and interpolation; exercise real localhost UDP plus injected message delay/jitter/loss/reordering.
- [x] Exercise two actual Mac Metal clients with injected movement/fire and both kill/death/respawn cycles; separately check native clicks, reload, F5 exclusion, focus loss, departure, fresh native join and lost-server UI.
- [x] Preserve offline practice and run its actual native combat regression route; see [04-multiplayer.md](handoffs/04-multiplayer.md).
- [x] Record user-reported participation in a successful two-player multiplayer playtest and milestone approval (2026-09-11); network conditions, platforms and individual checklist results were not supplied.
- [ ] Document physical held mouse/Shift, cursor-exit and focus subcases; general approval does not establish each result.
- [x] Validate GitHub CI and Windows compilation for exact implementation/fix commit `de704556391dae3f58ffa83aba17160691535342`; both jobs pass all 80 tests and locked builds. Later commits require their own CI results.
- [ ] Validate Windows hardware/GPU and an actual Mac-to-Windows match.
- [ ] Measure internet latency/loss and frame/server timing on real connections. Eight-player scaling is a later milestone.

## 4. Hosting and Go services

- [ ] Add player-hosted matches and explicit host-departure behavior.
- [ ] Implement internet connection setup and relay fallback as needed.
- [ ] Build one small Go room/invite service.
- [ ] Deploy official match hosting in India when ready and authorized.
- [ ] Test player hosting from Canadian users' actual connections.

## 5. Expand from playtest evidence

- [ ] Refine menus, map traversal, spawn fairness, visuals, sound, and animation.
- [ ] Add additional content and customization incrementally.
- [ ] Add accounts and persistence when required by a concrete feature.
- [ ] Establish distribution, updates, and platform packaging.

## Latest handoff

2026-09-11: Reviewed foundation, movement, combat and two-player multiplayer are saved in the existing repository. User-reported Mac movement/combat approval and successful multiplayer participation/approval are recorded without inferring multiplayer network conditions or unreported physical-input cases. Local formatting, strict Clippy, all 80 tests and locked Mac builds pass. The first Windows CI run exposed a Mac-specific practice trace assumption; a test-only fix now compares every tick against the preserved approved source on each platform, retaining the Mac golden and leaving runtime rules/tuning/dependencies unchanged. Both macOS ARM64 and Windows x64 CI pass all 80 tests and locked executable builds for `de704556391dae3f58ffa83aba17160691535342`. See [05-checkpoint.md](handoffs/05-checkpoint.md) for exact run links, failure/fix history and evidence boundaries. Check each later commit's own CI before sign-off. Remaining work is Windows hardware/GPU, a documented Mac-to-Windows match, physical input subcases and real internet/performance measurements. No hosting, deployment, purchase or agent delegation occurred.
