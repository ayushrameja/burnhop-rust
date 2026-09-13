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
- [ ] Measure internet latency/loss and frame/server timing on real connections. Controlled eight-player results are recorded in milestone 08; real internet validation remains pending.

## 4. Gameplay visuals — moved ahead of eight-player expansion

User decision, 2026-09-12: polish the playable scene now; preserve approved movement, combat and two-player networking.

- [x] Inspect actual browser references and the native baseline; record `VISUAL_DIRECTION.md`.
- [x] Add one shared illustrated pilot rig with idle, run, airborne, jet, aiming, reload, hit and death/respawn presentation.
- [x] Draw distinct pistol/M416 silhouettes and cosmetic slide/bolt feedback; preserve center-origin rays and clip nearby-cover artwork.
- [x] Style all existing arena solids with consistent surfaces, landscape depth and an initial-spawn range sign.
- [x] Pool bounded confirmed effects and clean up presentation across reset, lifecycle, joins and departure.
- [x] Replace the debug slab with a responsive native combat HUD and useful connection/errors.
- [x] Verify Mac native rendering, original combat/movement routes, both online roles, resize and focus recovery; retain representative screenshots.
- [x] Record user-reported completion of the visual playtest and approval of the result (2026-09-12); see the milestone 06 addendum.
- [x] Address the compact-HUD review finding: short edge readouts, collapsed F1 guidance, and native upper-platform/edge-flight, combat and connection-error checks at minimum, medium and desktop sizes. See milestone 06 follow-up.
- [x] Compact-HUD fix passed final review, as recorded in the checkpoint brief (2026-09-12); user visual approval, screenshot review and automated checks remain separate evidence.
- [ ] Windows hardware/GPU review.
- [x] Save approved visuals as `bf34b95bf0f9953b85635b8ec48a645ccfba45a9`; macOS ARM64 and Windows x64 CI passed all 90 tests and locked builds. See [07-visual-checkpoint.md](handoffs/07-visual-checkpoint.md). Later commits require their own CI verification.

## 5. Eight-player multiplayer and reliability

- [x] Record baseline, implementation plan and pass criteria before refactoring from `0a7c7e3`.
- [x] Eight stable generations, validated separated spawns, shared simultaneous combat and single-credit scoring; preserve approved practice regression.
- [x] Eight owners, clear ninth-player rejection, bounded lifecycle/queues, fresh replacement state and 60 Hz overload policy.
- [x] Protocol 2 lossless 1,185-byte snapshots and measured-RTT input scheduling, justified by failed fixed-lead/fragmented experiments.
- [x] Reuse the pilot rig for all actors; add held Tab scores and bounded per-actor effects/cleanup.
- [x] Controlled raw UDP baseline, 50/100/150 ms RTT with jitter/loss, one-second client stall/recovery; document precise injection/counter semantics.
- [x] 605-second eight-client soak: 13 replacement joins, 916 deaths, 906 respawns; no overload and bounded measured resources.
- [x] Actual Mac native rendering with seven synthetic clients; eight visible pilots, effects, desktop/compact scores, departure/reuse and respawns; offline and two-client regressions.
- [x] Local 102 tests, formatting, strict Clippy, locked builds and dependency boundaries. Preserve short Mac/Windows CI; long harness stays opt-in.
- [x] User-reported two-client localhost Mac playtest: movement/combat/scores, held-input focus recovery, replacement joins and server-shutdown disconnection (2026-09-12).
- [ ] Eight-human spawn fairness, delayed aiming, and physical-input cases beyond the approved local checklist.
- [x] Manager review reported no blocking issues; fresh checkpoint checks pass all 102 tests, formatting, strict Clippy, locked builds and dependency boundaries.
- [x] Save reviewed implementation as `437b6b67db0e07e9f237b54d90e90817d5c69748`; both platform CI jobs pass 102 tests and locked executable builds in [run 34699835294](https://github.com/ayushrameja/burnhop-rust/actions/runs/34699835294).
- Final checkpoint response must independently verify this documentation commit's exact SHA on both targets; implementation CI is not evidence for later commits.
- [ ] Windows hardware/GPU and an actual Mac-to-Windows match.
- [ ] Measure real internet latency/loss and a broader range of hardware before setting capacity/performance expectations.

## 6. Native menu and owned hosting

- [x] No-argument native main menu with Practice, Host Game, Join Game and Quit; mouse/keyboard states and readable 480 × 320 / desktop layouts.
- [x] Offline Escape pause/resume/return and preserved F5; online Escape menu with neutral input and continuing simulation.
- [x] Own the existing authoritative server in one worker thread, connect the host over UDP, preserve standalone server and explicit launch/review flags.
- [x] Numeric address editing/validation, local-only vs explicit LAN binding, connecting/cancel, full/compatibility/timeout/disconnect errors and fresh retry.
- [x] Guest leave, confirmed Stop Hosting, partial-startup/shutdown cleanup, repeated sessions and port reuse without touching external servers.
- [x] Agent-operated actual Mac host/guest windows: leave/rejoin/stop/rehost, errors/retry, focus/menu navigation, compact/desktop, IPv6 loopback; preserved offline native combat route passes seven checkpoints.
- [x] Final 111 tests, formatting, strict Clippy, locked host/Mac ARM64 builds, zero-dependency core, headless boundary, unchanged frozen fixtures and fourteen inspected PNGs. See [handoff 10](handoffs/10-menu-and-hosting.md).
- [x] Human menu/hosting review: all six checks explicitly approved on 2026-09-13; approval notes below are preserved.
- [x] Checkpoint save authorized; fresh local validation and reviewed commit contents recorded in [handoff 11](handoffs/11-menu-hosting-checkpoint.md).
- [ ] Exact-final-SHA remote CI completion: pending, handed to the user for manual monitoring after the authorized push.
- [ ] Windows compilation of this checkpoint and Windows hardware/GPU checks. Remote CI is pending; historical checkpoint CI is separate.
- [ ] Second-computer/LAN review is explicitly deferred; internet reachability remains unverified.

## 7. Hosting and Go services

- [x] Local/interface-bound player hosting and explicit host-departure behavior; milestone 10. Internet setup remains separate.
- [ ] Implement internet connection setup and relay fallback as needed.
- [ ] Build one small Go room/invite service.
- [ ] Deploy official match hosting in India when ready and authorized.
- [ ] Test player hosting from Canadian users' actual connections.

## 8. Expand from playtest evidence

- [ ] Refine menus, map traversal and spawn fairness from playtests; add audio in a later milestone.
- [ ] Iterate the first visual scene from user and Windows hardware feedback.
- [ ] Add additional content and customization incrementally.
- [ ] Add accounts and persistence when required by a concrete feature.
- [ ] Establish distribution, updates, and platform packaging.

## Latest handoff

2026-09-12: [09-eight-player-checkpoint.md](handoffs/09-eight-player-checkpoint.md) records the authorized reviewed save, fresh local checks, evidence correction and outstanding validation. [08-eight-player-reliability.md](handoffs/08-eight-player-reliability.md) records the eight-player implementation, protocol/scheduling decisions, controlled packet tests, 605-second soak, actual native review and remaining limits. Manager-reviewed implementation `437b6b6` is normally pushed and green on both CI targets. This documentation commit needs its own exact-SHA check in the final checkpoint response. [07-visual-checkpoint.md](handoffs/07-visual-checkpoint.md) preserves the saved visual/CI baseline; [06-gameplay-visuals.md](handoffs/06-gameplay-visuals.md) preserves the approved art direction and earlier native evidence. Windows hardware, real internet and eight-human feel remain outstanding; final documentation-SHA CI must be verified separately.


Latest approval: the user passed the two-client Mac localhost checklist. Final saved SHA `b64d213b1bdcfaa272848926ae9716fdff56c042` was independently verified green on both CI targets in [run 34701237678](https://github.com/ayushrameja/burnhop-rust/actions/runs/34701237678). See the dated approval addendum in [checkpoint 09](handoffs/09-eight-player-checkpoint.md). Next validation: a second physical machine on LAN; Windows hardware and real internet remain separate pending checks.


Latest implementation: [10-menu-and-hosting.md](handoffs/10-menu-and-hosting.md), 2026-09-12. The native menu/local hosting milestone is ready for local review with 111 passing tests, actual two-window Mac interaction, preserved native offline route, and fourteen inspected screenshots. No commit/push or new CI run was performed. Preserve the earlier user-approved gameplay/localhost baseline. Next: human menu/hosting review and separately authorized save/CI; second-machine LAN remains deferred.


## Menu and hosting human approval — 2026-09-13

The user explicitly reports all six manager-requested checks passed on the local Mac setup: Practice pause/resume; Host Game and Join Game; input recovery across menus/focus; guest leave/rejoin; Stop Hosting and same-port rehosting; and usability. This is user-reported hands-on approval of milestone 10, superseding earlier statements that its human review was pending. Preserve this approved behavior and visual baseline.

The manager independently reran 111 tests, formatting, strict Clippy and the locked Mac build successfully during review. New-change Windows CI, Windows hardware/GPU, deferred second-machine LAN, Mac-to-Windows, real internet and eight-human testing remain separate outstanding validation. Next: a separately authorized save and exact-final-SHA macOS/Windows CI checkpoint, then the web-to-native feature inventory. This approval records playtest results; it does not commit or push the changes.


## Current checkpoint and next action — 2026-09-13

[11-menu-hosting-checkpoint.md](handoffs/11-menu-hosting-checkpoint.md) supersedes the historical next-action statements above: human menu/hosting review passed all six checks, and the user now authorizes commit and normal push to `origin/main`. Fresh formatting, strict Clippy, 111 tests, locked host/Mac ARM64 builds and dependency boundaries pass; no implementation fix was needed. All approval notes and prior evidence are preserved. Remote macOS ARM64 and Windows x64 CI remains **pending** for the final pushed SHA. The user will monitor the exact Actions run; stop after handoff without polling for completion. This is not a fully validated checkpoint until its own CI results are known. Windows hardware, deferred second-machine LAN, Mac-to-Windows, real internet and eight-human testing remain separate. The later feature inventory is outside this save task.
