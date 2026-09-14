# Roadmap

Current continuation (2026-09-14): save the reviewed, visually approved offline character-customization milestone to `origin/main`; see [checkpoint 17](handoffs/17-character-customization-checkpoint.md). **New remote CI is pending; the user monitors the exact final SHA on macOS ARM64 and Windows x64.** Locate the run briefly, then hand off without completion waits, repeated polling, watchers or additional tasks. Windows hardware/LAN/internet remain separate.

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
- [x] Checkpoint CI reported passing on macOS ARM64 and Windows x64 for approved SHA `7e9610a3d2398e058bfee74bf7fd684b8f0d13fe` in [run 34740619832](https://github.com/ayushrameja/burnhop-rust/actions/runs/34740619832), per the inventory task brief; not independently queried during this audit.
- [ ] Windows hardware/GPU checks. Windows CI success is reported for this checkpoint; hardware playtesting remains separate.
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


## Historical checkpoint 11 handoff — 2026-09-13

[11-menu-hosting-checkpoint.md](handoffs/11-menu-hosting-checkpoint.md) supersedes the historical next-action statements above: human menu/hosting review passed all six checks, and the user now authorizes commit and normal push to `origin/main`. Fresh formatting, strict Clippy, 111 tests, locked host/Mac ARM64 builds and dependency boundaries pass; no implementation fix was needed. All approval notes and prior evidence are preserved. Remote macOS ARM64 and Windows x64 CI remains **pending** for the final pushed SHA. The user will monitor the exact Actions run; stop after handoff without polling for completion. This is not a fully validated checkpoint until its own CI results are known. Windows hardware, deferred second-machine LAN, Mac-to-Windows, real internet and eight-human testing remain separate. The later feature inventory is outside this save task.


## Current inventory and migration sequence — 2026-09-13

[Handoff 12](handoffs/12-web-to-native-inventory.md) and [WEB_TO_NATIVE_INVENTORY.md](WEB_TO_NATIVE_INVENTORY.md) supersede historical next-action statements above. Both trees were initially clean; native HEAD is the approved `7e9610a3d2398e058bfee74bf7fd684b8f0d13fe`. The brief reports Mac ARM64/Windows x64 CI success for that checkpoint; no live CI check, new build or gameplay test was performed in this documentation task.

- [x] Inventory actual active web maps, characters, weapons, camera, audio, every settings/input option and backend boundaries with source/asset references, native gaps, reuse constraints and acceptance checks.
- [x] Record full repository SHAs/status, asset provenance gaps, stale-reference discrepancies and source-versus-runtime evidence limits.
- [x] Confirm first map with the user: **Outpost**, the active web multiplayer map. Practice range remains the existing native baseline.
- [x] Define a bounded proposed first implementation task, affected systems, compatibility considerations and regression/human checks in inventory section 11.
- [ ] **Superseded inventory proposal; do not implement:** Proposed milestone 13: offline Outpost with all authored collision contours, safe player/bot spawns, polygon movement/rays, required tunnel crouch, open-floor recovery and map-aware camera. Preserve approved range/standing/weapon/menu behavior; introduce basic local framing preference alongside camera. Offline-first scope and exact camera/crouch/recovery recommendations need to be adopted in the implementation brief.
- [ ] Extend Outpost to current native direct-connect Host/Join with explicit map/stance compatibility, complete prediction/snapshots, map-scoped spawns and a deliberate online fall policy. A room service is not needed for this step.
- [ ] Character detail and customization in small playable slices; active look persistence first, complete catalog/outfits/named looks and online appearance after.
- [ ] Weapon batches: Revolver/AK-47, UZI/UMP, Sniper; then practice racks, dual transfer/reload and punch in separately reviewable slices. Preserve approved pistol/M416 behavior; decide additional web combat mechanics explicitly.
- [ ] Audio with mute/channel controls on introduction; check provenance and actual native playback before asset parity claims.
- [ ] Complete settings/remapping/reduced motion/graphics and robust versioned local persistence, expanding the basic preferences introduced with earlier features.
- [ ] Rust ready/countdown/timed-round/results/rematch flow; authoritative pickup/supply schedules once weapon and match-clock dependencies exist.
- [ ] Backend room/invite integration: evaluate existing service adapter versus small Go supporting service, secure admission and reachable Rust match endpoints. Keep Rust movement/combat authority; accounts/database only for concrete later needs. Existing hosting is user-reported Germany; official India deployment remains future separately authorized work.
- [ ] Distribution/signing/updates and clean-machine Mac/Windows checks, then broader hardware, second-machine LAN, cross-platform and real internet validation. No completion inferred from local Mac approval or CI.

This sequence advances crouch only because Outpost's tunnels require a shorter collider. Basic settings accompany each feature. Full catalog, weapon handling, audio, cloud profiles and internet hosting are not part of the first map milestone. The inventory documents asset/license unknowns and deliberate native differences; do not treat them as permission to change approved direction. No code, asset, dependency, deployment, commit or push was performed or authorized by this audit.


## Revised first-map milestone — original geometry (2026-09-13)

- [x] Record the user's original-geometry decision: no MM permission obtained, no reuse/extension of Outpost layout; broad GTA/Apex atmosphere/gameplay references only.
- [x] Record approved resource/score-preserving fall recovery, released-input gate and 180-tick bot grace. No further recovery/provenance approval question for this design.
- [x] Complete v1 (geometry superseded below) [Ember Relay original-map brief](ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md) and [editable SVG layout](design/ember-relay-layout.svg): 3200×1900, three connected areas, 17 solids, eight candidate spawns/fixed bot, staged jet climbs, short crouch alternatives, original evening visuals and bounded implementation/acceptance scope. Verify source values, coordinates/links and rendered SVG readability; movement numbers are estimates, not runtime tests.
- [x] Mark the Outpost brief and inventory's geometry-import direction historical; preserve their reference findings and all approved native work.
- [ ] When implementation is requested: add v2 original static geometry, nine convex-ramp queries, offline stance/spawn/recovery state and simple original native visuals; retain dependency-free core and immutable range fixtures.
- [ ] Add the offline map action and map-aware camera/aim/resize handling; preserve range defaults, fixed network codec, online range and all Host/Join behavior.
- [ ] Run meaningful collision/traversal/clearance/recovery/aim/switching regressions, appropriate Mac/Windows checks and the brief's native human route. Record observations before tuning original geometry or considering expansion.

Current first-map scope supersedes the older Outpost-first entries above, including their tentative Wide/settings work. No gameplay code, dependency change, runtime asset, build, commit, push, deployment, extra agent or online map implementation is part of this design completion. Later catalog/backend milestones remain deferred recommendations, not authorization; replace any old Outpost online-plan assumption with a separately scoped original-map decision after offline acceptance.


## Ember Relay v2 — geometry revision before implementation (2026-09-13)

- [x] Record user feedback: v1 too flat/orderly/safe; add broad elevation/partial cover, short exposed floating platforms, meaningful lower routing and real fuel-related falls. Preserve original layout authorship and simple evening visuals.
- [x] Revise [brief](ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md) and [SVG](design/ember-relay-layout.svg):35 solids (26 rectangles/nine convex ramps), unchanged 3200×1900 bounds; nine 100–160-wide floats; two lower chambers,140/120 jump gaps and 200-wide rest island; two brief crouch shortcuts/four upper connections; unobstructed central recovery voids.
- [x] Recalculate native jump/jet/fuel/landing estimates, supported S0/B0 and candidate placements, updated deepest-floor camera extent and selected sightlines; verify document links and rendered SVG. These are design/static checks, not gameplay validation or spawn-fairness proof.
- [ ] User design review of v2. No unresolved material question; geometry and route feel remain recommendations awaiting review.
- [ ] On subsequent implementation request, follow the brief’s bounded shared-core/offline plan and exact geometry. Preserve airborne fuel regeneration, resource/score retention, release gate and 180 eligible bot-grace ticks, plus range/Host/Join/fixed codec/frozen fixtures.
- [ ] Native traversal/aim/recovery/switching tests and short human route: pay particular attention to narrow-platform braking, rising-slope hops, shaft undersides, underfuel shortcuts and lower-route usefulness. Adjust original geometry if needed before approved tuning.

V2 supersedes v1’s 17-solid counts and old route estimates. No builds, gameplay implementation, generated runtime assets, new weapons/audio/backend/maps, agents, commits, pushes or deployment in this revision. Review readiness does not imply approval or proven gameplay quality.


## Milestone 13 — offline Ember Relay implementation (2026-09-13)

This supersedes the earlier design-only next actions while retaining their history.

- [x] User-approved v2 geometry implemented exactly: 35 solids, S0/B0, convex collision/rays, full-height voids; no tuning or geometry deviations.
- [x] Offline stance/clearance, active-map reset/respawn, preserved-resource recovery/neutral gate/eligible grace, camera/aim lifecycle and original simple native visuals.
- [x] Ember Relay second menu action; range defaults, frozen fixtures, fixed online codec, and Host/Join preserved.
- [x] 135 tests, formatting/strict Clippy, locked host/Mac ARM64 builds, dependency boundaries and unchanged frozen hashes; 26 stages at 60/full fuel, both sleeves and voids, low-fuel cases and extreme sweeps.
- [x] Actual native injected traversal, desktop/800×524/480×320 presentation, ordinary map/reset/pause/fire/resize interaction and two-window local hosting rechecked; inspected framebuffer evidence indexed in [screenshots](screenshots/13-ember-relay/README.md).
- [x] User reports completing the requested offline playtest and says “it is working great” (2026-09-13). Offline-experience approval; no additional Windows/LAN/internet or per-subcase physical-input evidence is inferred.
- [x] Independent review finds no blocking defects; 135 tests, formatting, strict Clippy, locked Mac builds, frozen regression and targeted probes pass. See [handoff 14](handoffs/14-ember-relay-review.md).
- [ ] Windows compilation/CI and hardware, second-machine LAN, Mac-to-Windows and real internet remain unverified for these changes. No inferred save or online map authorization.

No collision-geometry or approved tuning correction was necessary. User playtest approval and independent review are complete; next is a separately authorized save/CI checkpoint. Screenshots and injected stages do not independently prove feel. No commits, pushes, deployment or CI actions were performed in the review.


## Milestone 14 — independent offline review (2026-09-13)

- [x] Record user-reported offline approval and review actual geometry, collision/stance, recovery/eligible grace, camera/aim, session cleanup and fixed online range behavior; no blocking defects.
- [x] Independently rerun 135 tests, formatting/strict Clippy, locked host/Mac ARM64 builds, dependency and frozen-fixture checks; add temporary seam/corner/recovery and ephemeral-packet probes without editing runtime/tests.
- [x] Inspect representative native desktop/compact screenshots and preserve evidence limits. Separate range-script attempt remains canceled; the ten undersized packets have a reproducible pre-handshake mechanism and plausible existing-test source, not proven historical attribution.
- [x] Both P3 fixes: U04 decorative accent follows its authored slope; the ten-cancel test uses an owned ephemeral endpoint and verifies destination/state/socket cleanup. Focused checks, all 135 tests, formatting/strict Clippy and locked host/Mac ARM64 builds pass. Desktop/480×320 native captures retained in [handoff 14](handoffs/14-ember-relay-review.md); no additional human playtest required. Historical packet-source attribution remains qualified.
- [ ] Separately authorized save and change-specific Mac/Windows CI checkpoint. No CI started or monitored by this review; Windows hardware, LAN, cross-platform and internet checks stay outstanding. No repeat Mac offline playtest required before save on current evidence.


## Milestone 15 — Ember Relay save and manual CI handoff (2026-09-13)

- [x] User authorizes commit and normal push of the approved milestone; no-blocker independent review and both P3 fixes recorded.
- [x] Review branch/remote and candidate files; no newer remote commits at pre-save fetch. Preserve source/tests, inventory/design history, handoffs, 22 native screenshots and bounded logs; exclude temporary/configuration/build output.
- [x] Reuse validation for unchanged code: 135 tests, frozen 12,000-command regression, strict Clippy and locked host/explicit Mac ARM64 builds. Fresh formatting, source/documentation diff and evidence-integrity checks pass; two original console captures retain harmless final blank lines, recorded in handoff 15. No new runtime changes or long test reruns.
- [x] Record saved scope and manual CI handoff in [handoff 15](handoffs/15-ember-relay-checkpoint.md); the normal push starts the existing Native checks workflow. Final task response verifies pushed SHA, live remote equality and exact run association.
- [ ] User to monitor both `aarch64-apple-darwin` and `x86_64-pc-windows-msvc` for the final SHA. Remote CI is pending in this commit; both jobs must pass before marking it cleared.
- [ ] Windows hardware/GPU, second-machine LAN, Mac-to-Windows, real internet and eight-human validation remain separate. No repeat Mac offline human playtest required for this save.


## Milestone 16 — detailed pilot and local Character screen (2026-09-14)

- [x] Inspect active web renderer/creator/catalog usage; record exact subset and Base-versus-active-outfit distinction.
- [x] Add procedural head/hair/face, clothing/equipment and limb detail, preserving approved original preset, articulated rig and gameplay dimensions.
- [x] Native Character menu, large shared-pose preview, mouse/keyboard controls and compact layout; Apply/Cancel/Restore Defaults with independent draft state.
- [x] Per-user versioned bounded configuration, catalog/color validation and safe replacement; failures preserve saved choices.
- [x] Offline range/Ember integration through lifecycle/reset/menu/restart; bots and all eight online pilots remain baseline. No protocol/server/core changes.
- [x] 142 tests, unchanged 12,000-command regression, strict Clippy, formatting, locked host and explicit Mac ARM64 builds. Inspected ordinary native UI/restart/Host/Join and eight-slot synthetic-peer rendering; debug pose fixtures separately identified. No completed native combat-script claim after focus cancellation.
- [x] [Handoff 16](handoffs/16-character-customization.md), [screenshot index](screenshots/16-character-customization/README.md), supported catalog and human checklist.
- [x] User visual approval: “looks good” (2026-09-14). This does not establish separate completion of persistence, lifecycle, online or physical-input checklist items. Checkpoint 17 authorizes commit/normal push and brief CI run lookup; Windows hardware and current-change CI remain separate.
- [ ] Remaining web catalog, named looks, Heavy, body/face editor, online appearance metadata and other backend work remain out of scope.

Baseline update from the current user brief: both Mac ARM64 and Windows x64 checkpoint CI jobs passed 135 tests for `8a13d129226dbaab80cb2b5775bf8caa78bc39f7`. This is reported checkpoint evidence, not newly monitored CI or CI for milestone 16.


## Milestone 17 — character customization save and manual CI handoff (2026-09-14)

- [x] Record user visual approval and manager-reported independent 142-test, formatting, strict Clippy and locked Mac-build passes. Preserve handoff 16 and all evidence qualifications.
- [x] Fetch and inspect `main`: live remote and local baseline both `8a13d129226dbaab80cb2b5775bf8caa78bc39f7`, with no incoming work. Review source/tests/docs, 24 useful PNGs and bounded logs; exclude personal appearance, secrets, temporary and build output.
- [x] Reuse unchanged-code validation; fresh formatting, whitespace, protected-boundary and screenshot-integrity checks pass. No runtime fix or gameplay/network rerun needed.
- [x] Prepare [checkpoint 17](handoffs/17-character-customization-checkpoint.md) for authorized normal commit/push. Final task response verifies saved SHA, live remote equality, working-tree status and exact Actions run association.
- [ ] New remote CI: user to monitor `aarch64-apple-darwin` and `x86_64-pc-windows-msvc` for the exact final SHA. Both must pass before clearing this gate; no Windows success is assumed.
- [ ] Windows hardware/GPU, second-machine LAN, Mac-to-Windows, real internet and unreported human checklist items remain separate.
