# Project context

Last updated: 2026-09-13

## Why this exists

The user and assistant agreed to explore a separate native version of Burnhop. The user wants to make a distinctive, enjoyable game and learn a new stack after boredom and burnout with TypeScript. Learning and sustained enjoyment are meaningful outcomes even if the game is not commercially successful.

The current browser game mixes Mini Militia-inspired side-view gameplay with Apex-influenced movement/combat ideas. The user reports confusing UI, map problems, and online latency issues. Their causes have not been comprehensively diagnosed. Changing language alone is not a verified fix.

## Agreed direction

- Preserve the current website/codebase and build in a separate folder and Git repository.
- Use Rust + Bevy for a native game on M-series Macs and Windows. Do not use Tauri, Bun, or a browser runtime for the native client.
- Keep a shared Rust gameplay core for movement, collision, jet fuel, weapons, damage, respawns, and match rules. Use it for the authoritative server and client prediction.
- Use Go for supporting online services: initially room discovery/invites, later accounts, matchmaking, and saved profiles as required.
- Add PostgreSQL when persistence is needed; it is not a first-playable requirement.
- Official match servers will move to India. This is the user's intended hosting direction, not an unresolved Canada-versus-India optimization exercise.
- Canadian players and players in other regions should be able to host matches on their own computers for friends.
- Retain the current visual identity where useful, while improving clarity, animation, effects, and game feel.
- Start with small playable milestones before expanding toward a complete remake. A custom engine is not currently planned.

## Existing reference project

Path: `/Users/ayushrameja/codebase/LHP/burnhop`

Remote observed: `git@github.com:ayushrameja/burnhop.git`

The existing project uses TypeScript, React, Vite, Canvas 2D, and an authoritative Colyseus server. It includes practice, an Outpost map, private multiplayer for up to eight players, movement/jet boots, weapons, customizable characters, prediction, and tests.

Useful references include `README.md`, `src/game/`, `src/multiplayer/`, `src/online/`, `src/server/`, `public/assets/`, and `docs/multiplayer-release.md`. Treat these as reference material, not a requirement to reproduce every feature or bug.

Recorded release evidence includes a passing eight-player Frankfurt capacity test; it does not establish human smoothness or the current bottleneck. Do not claim that Rust has already solved latency or that native performance is proven.

## What must be rewritten or adapted

- HTML/React/CSS menus and HUD become native Bevy UI.
- Canvas drawing code must be recreated with compatible rendering techniques such as sprites and meshes.
- TypeScript gameplay rules become Rust code, preserving useful tuning and intended behavior.
- Map data, textures, fonts, and audio can be reused or converted after format and license checks.
- Relevant existing tests become behavioral references for Rust checks.
- UI confusion and map defects should be fixed deliberately, not copied for exact feature parity.

Bevy provides rendering and engine systems, but improved art and game feel still require design, tuning, and testing.

## Networking approach

The authoritative Rust server should support both official dedicated hosting and player hosting. Share gameplay rules without making rendering a server dependency. Client prediction and correction remain necessary; shared Rust code does not guarantee cross-platform deterministic behavior automatically.

Go helps players locate and join matches. Gameplay traffic goes to the match server directly or through a relay when needed, not through the account API on every update.

Internet hosting requires connection establishment through NAT/firewalls and potentially relay infrastructure. Renet 2.0.0 plus renet_netcode 2.0.0 is selected for the direct-connect milestone. Public hosting provider, costs, secure token issuance and packaging remain deferred. The initial transport uses unsecure development authentication; this is not a public hosting security model. For the first hosting implementation, the proposed behavior is to end the match when the host leaves; seamless migration is deferred. Player-hosted matches should initially be treated as casual, not trusted competitive results.

## Scope and open decisions

- Initial direction remains a side-view game. A switch to full 3D is not agreed.
- Foundation selects Rust 1.98.1 (edition 2024) and Bevy 0.19.1, pinned in the workspace and lockfile. The gameplay core has no rendering dependencies. See `docs/handoffs/01-foundation.md` for validation evidence.
- Movement now uses a dependency-free 60 Hz core with explicit tick commands, f64 browser-pixel coordinates (top-left, X-right/Y-down), and swept rectangular collision. Bevy converts positions to centered Y-up sprites and owns clocks, input, interpolation, and camera. The current practice client uses the browser's default separate Space-jump/Shift-jet hold controls. See `docs/handoffs/02-movement.md`.
- The user reports personally playtesting and approving both Mac movement and combat feel with placeholders (recorded as user-reported evidence in `docs/handoffs/03-combat.md`). No weapon or movement tuning was changed for multiplayer. `MatchState` now contains reusable actors; `step_match` owns every present actor’s movement, weapons, rays, damage and lifecycle. `step_practice` is an adapter to the same rules with the existing stationary bot policy. The 12,000-tick pre-refactor practice command stream is preserved as an equivalence regression. After Windows CI exposed a platform-specific raw trace hash, the test now compares every tick against frozen approved source on each platform and retains the original Mac golden; runtime rules are unchanged. See `docs/handoffs/04-multiplayer.md`.
- 2–8-player direct connect now uses a headless Rust server at 60 Hz, explicit assigned actors and scheduled input sequences, bounded queues, 30 Hz authoritative snapshots, local movement prediction/reconciliation and remote interpolation. Core has zero dependencies; transport and the fixed bounded codec live in a separate protocol crate. Native focus loss sends neutral input without pausing the match. No hit rewind, automatic reconnect or world reset online. Asset representation, deployment provider and distribution/update mechanism remain to be evaluated.
- The checkpoint brief reports user participation in a successful two-player multiplayer playtest and approval of the reviewed milestone. This is user-reported evidence; network conditions, platforms and individual physical input/focus subcases were not supplied. See the dated addendum in `docs/handoffs/04-multiplayer.md`. Automated tests, CI compilation and hardware/internet validation remain separate evidence.
- The reviewed milestones and test-only portability fix are committed and pushed to `main`. GitHub CI passed formatting, strict Clippy, all 80 tests and locked executable builds on macOS ARM64 and Windows x64 for `de704556391dae3f58ffa83aba17160691535342`. See `docs/handoffs/05-checkpoint.md` for both CI run links and the original Windows regression failure. CI is not Windows hardware/GPU or live cross-platform/internet playtesting. Verify each later commit's own CI status.
- Hardware performance targets should be chosen and measured before making performance promises.
- No production deployment, server relocation, or paid service has been configured as part of this setup.
- The user created the remote repository `git@github.com:ayushrameja/burnhop-rust.git` and authorized syncing this local project to it.
- The working directory name is `burnhop-native`; the public game name need not change.

- The user moved a focused gameplay-visual milestone ahead of eight-player networking (2026-09-12). Preserve the approved simulation/protocol and existing arena; prioritize the illustrated pilot, animations, weapons, terrain, effects and native HUD. Full menus, customization, audio and additional maps remain deferred. See `VISUAL_DIRECTION.md` and `handoffs/06-gameplay-visuals.md`. No purchase, deployment, commit, push or agents are authorized in this milestone.

- The user reports completing the gameplay visual playtest and says the result looks good (2026-09-12). Record milestone 06 as user-reported visual/animation approval and preserve it as the baseline. Platform, offline/online mode and individual input/checklist results were not specified; Windows and unreported physical-input checks remain separate. See the approval addendum in `handoffs/06-gameplay-visuals.md`.

## How to resume

Read this file, the roadmap, `docs/handoffs/11-menu-hosting-checkpoint.md`, `docs/handoffs/10-menu-and-hosting.md`, `docs/handoffs/09-eight-player-checkpoint.md` and `docs/handoffs/08-eight-player-reliability.md` for current implementation and measured limits. `docs/handoffs/07-visual-checkpoint.md` records the saved visual milestone/CI checkpoint; `06-gameplay-visuals.md` preserves detailed native review evidence and `05-checkpoint.md` preserves the prior checkpoint. Inspect the existing game's movement and map boundaries as needed. The movement milestone implements one arena, a controllable character, jump/jet fuel, rectangular collision, camera and behavior tests. Read `docs/handoffs/04-multiplayer.md` for the direct-connect architecture, actual Mac checks and remaining human/platform checks; `03-combat.md` records the approved practice combat loop; `02-movement.md` records the user's movement approval and earlier evidence. Read `docs/REFERENCE_GAMEPLAY.md` before porting tuning or map data. Do not start with a complete account system or a full port of all cosmetics.

- Compact-HUD review follow-up (2026-09-12): remove the wide `top=82` compact panels, retain short corner readouts and a collapsed F1 guide, and verify actual native traversal/errors at 480 × 320, 800 × 524 and 1280 × 720. This is a HUD-only correction to the approved visual/animation baseline. Final review and separately authorized save/CI come next; no commit or push is authorized here. See the follow-up in milestone 06.

- Visual save checkpoint (2026-09-12): the compact-HUD finding passed review according to the current brief. The user now explicitly authorizes committing/pushing this approved milestone and verifying GitHub Actions, superseding earlier task-specific save restrictions. Fresh local formatting, strict Clippy, 90 tests, locked host/explicit Mac builds and dependency-boundary checks pass. Exact pushed-commit CI remains pending; hardware/internet evidence is unchanged. Preserve the approved visuals and gameplay; do not begin eight-player work as part of saving this checkpoint.

- Saved visual milestone (2026-09-12): implementation `bf34b95bf0f9953b85635b8ec48a645ccfba45a9` is normally pushed to `main`. [Native checks 34681615290](https://github.com/ayushrameja/burnhop-rust/actions/runs/34681615290) passed on macOS ARM64 and Windows x64, with 90 tests and locked executable builds on each; no fix was needed. See [07-visual-checkpoint.md](handoffs/07-visual-checkpoint.md). The handoff's documentation commit requires its own exact-SHA CI sign-off. No Windows hardware/GPU, live Mac-to-Windows or real internet validation is inferred.


## Current milestone 08 — eight players and measured reliability

2026-09-12: expanded from clean checkpoint `0a7c7e3fe0692f05a5a9cbaef8da75bd91174957`. The implementation request prohibited commit/push/deployment and additional agents. The subsequent checkpoint 09 brief reports manager review with no blocking issues and explicitly authorizes committing and normally pushing to `origin/main`, then checking exact-SHA macOS ARM64 and Windows x64 CI. This supersedes earlier task-specific save restrictions. New human playtest approval has not been reported. The browser reference remains untouched.

Eight stable slots have fresh join generations, common 60 Hz movement/combat, deterministic simultaneous-shot scoring and validated separation-based online spawns. Offline practice retains exact old spawns and its frozen 12,000-tick regression. The server allows eight gameplay clients plus two bounded pending/rejection seats. Departures clear state, inputs and effects before slot reuse.

Protocol 2 / gameplay `0x4255_524e_0008_0001` sends complete lossless snapshots in at most 1,185 bytes at 30 Hz, below the pinned Renet message-slicing threshold. Fixed six-tick input scheduling failed at 150 ms RTT/loss; measured-RTT scheduling now reserves 6–24 ticks with bounded recovery and neutral missing commands. Remote interpolation remains six ticks behind the received snapshot; no hit rewind or prediction of damage was added. No new dependency, tuning, map, service or transport reliability implementation.

All eight actors use the approved pilot rig, actual slot labels and per-generation interpolation/effects. Held Tab shows authoritative kills/deaths without expanding the ordinary HUD. Mac Metal screenshots cover eight visible pilots, jets, compact scores, departure/reuse, combat and respawns. Offline and two-native-client scripted routes pass; this does not establish physical-input feel or eight-human play.

Local checks: 102 tests, strict Clippy, formatting and locked builds. Controlled raw UDP baseline/50/100/150 ms RTT and one-second stall profiles pass; the 605-second 100 ms/1% loss soak records 13 replacements, 916 deaths, 906 respawns and no server overload. Bounded buffers and sampled process memory showed no runaway growth in that run. Exact measurements, known limits and commands are in [08-eight-player-reliability.md](handoffs/08-eight-player-reliability.md). At implementation handoff time these changes had not run in GitHub Actions. Both platform jobs are preserved; checkpoint 09 records the authorized save and subsequent CI evidence. Windows hardware, cross-platform/internet play and human spawn/aiming feel remain pending.

## Checkpoint 09 — reviewed eight-player save

2026-09-12: fresh local formatting, strict Clippy, all 102 workspace tests, locked host/explicit Mac ARM64 builds and dependency-boundary checks pass. No implementation fix or dependency change was needed. Frozen practice fixtures and movement/combat tuning remain intact. Candidate review found one entirely black PNG; its bytes were preserved outside Git and the screenshot index corrected. Eleven useful PNGs and the bounded measurement evidence are included. The recorded 605-second soak was not repeated because no runtime code changed during this checkpoint. See [09-eight-player-checkpoint.md](handoffs/09-eight-player-checkpoint.md) for save status, CI evidence and remaining validation. Manager review is separate from unreported new human approval.

Saved implementation: `437b6b67db0e07e9f237b54d90e90817d5c69748`, normally pushed to `main`. [Native checks 34699835294](https://github.com/ayushrameja/burnhop-rust/actions/runs/34699835294) passed on macOS ARM64 and Windows x64; both job logs confirm 102 workspace tests, strict Clippy and locked executable builds. No runtime/CI fix was required. This documentation update triggers its own CI; the final checkpoint response must verify that exact SHA separately. New human approval, Windows GPU/hardware, Mac-to-Windows, physical held-input and real internet checks remain pending.


## Human localhost playtest approval — 2026-09-12

The user reports completing the manager's two-client Mac checklist and says: “pass just did that and all looks and feels right”. This is user-reported approval of one local Rust server and two native clients connecting to 127.0.0.1:5000: identities/movement/jump/jet, combat/death/respawn/Tab scores, held-input focus recovery, client departure/replacement with fresh scores, and server-shutdown disconnection. No new automated or manager-observed physical playtest is claimed. This supersedes earlier statements that new local human approval was unreported.

The manager independently verified final saved SHA b64d213b1bdcfaa272848926ae9716fdff56c042 against live remote main and GitHub Actions run 34701237678: both macOS ARM64 and Windows x64 passed 102 workspace tests, formatting, Clippy and locked builds. This records the saved code checkpoint's result, not CI for subsequent documentation edits.

Still pending: eight-human feel/spawn fairness, delayed moving-target aiming, held-input cases beyond the supplied local checklist, a second physical machine/LAN test, Windows hardware/GPU, Mac-to-Windows and real internet hosting/reliability. Local server-plus-client operation does not establish internet reachability, a Host Game UI or official India deployment. Preserve the approved gameplay and visuals.


## Current milestone 10 — native menus and owned hosting

2026-09-12: built on `b64d213b1bdcfaa272848926ae9716fdff56c042`, preserving the user-reported two-client localhost approval and its existing local documentation edits. No commit, push, deployment or agents are authorized for this milestone. Second-computer/LAN testing is explicitly deferred.

No arguments now opens a styled native Bevy menu with Practice, Host Game, Join Game and Quit. Escape pauses offline practice, but online menus release gameplay intent and keep polling/sending neutral input. Guests leave independently; Stop Hosting requires an explicit confirmation. Failed/cancelled connections can be retried without restarting. Address entry supports the existing numeric IPv4/bracketed IPv6 format, with local-only defaults and explicit LAN-interface binding, no advertised wildcard/internet address. See [handoff 10](handoffs/10-menu-and-hosting.md).

Hosting links the existing workspace server library and owns one standard-library worker thread. The worker runs the existing authoritative server and bounded clock; the host client still connects over UDP. It requires no separate server executable/terminal. Drop requests stop, joins the worker and releases its socket; failed partial startup also cleans up. The standalone headless server remains independent of Bevy. The internal client-to-server path dependency is the only dependency/lockfile change; the core remains dependency-free and gameplay/protocol fixtures are unchanged.

Local formatting, strict Clippy, **111 tests**, locked host/explicit Mac ARM64 builds and dependency boundaries pass. Agent-operated native windows verify host/join, guest leave without host shutdown, fresh joins, confirmed stop/disconnect, same-port rehosting, normal window-close port release, failed-bind/retry, cancellation/timeout/retry, compact/desktop layouts, editable IPv4/IPv6, native IPv6 loopback hosting and menu/focus recovery. The preserved offline native scripted combat route passes all seven checkpoints. Fourteen inspected nonblank PNGs are in [the screenshot index](screenshots/10-menu-and-hosting/README.md).

These are agent-native and automated checks, not new human approval. Final uncommitted changes have not run Windows CI; Windows hardware/GPU, Mac-to-Windows, deferred second-machine LAN, real internet and prior eight-human/latency limitations remain. No long soak was repeated because gameplay, packet format/scheduling and transport reliability did not change. Next: human menu/hosting review, then separately authorized save and exact-SHA CI.


## Menu and hosting human approval — 2026-09-13

The user explicitly reports all six manager-requested checks passed on the local Mac setup: Practice pause/resume; Host Game and Join Game; input recovery across menus/focus; guest leave/rejoin; Stop Hosting and same-port rehosting; and usability. This is user-reported hands-on approval of milestone 10, superseding earlier statements that its human review was pending. Preserve this approved behavior and visual baseline.

The manager independently reran 111 tests, formatting, strict Clippy and the locked Mac build successfully during review. New-change Windows CI, Windows hardware/GPU, deferred second-machine LAN, Mac-to-Windows, real internet and eight-human testing remain separate outstanding validation. Next: a separately authorized save and exact-final-SHA macOS/Windows CI checkpoint, then the web-to-native feature inventory. This approval records playtest results; it does not commit or push the changes.


## Checkpoint 11 — approved menu and hosting save, CI pending

2026-09-13: the user explicitly authorizes committing and normally pushing milestone 10 to `origin/main`, superseding the earlier task-specific save restrictions. All six human approval notes above are preserved. Fresh checkpoint checks pass formatting, strict Clippy, all 111 tests, locked host and explicit Mac ARM64 workspace builds, independent core/server checks and all-target dependency boundaries. No implementation fix or dependency upgrade was needed; the intended internal client-to-server hosting dependency remains. Core, frozen fixtures, gameplay tuning, codec/prediction, pinned versions and CI workflow are unchanged from `b64d213b1bdcfaa272848926ae9716fdff56c042`.

[Checkpoint 11](handoffs/11-menu-hosting-checkpoint.md) records the reviewed source/tests, fourteen original screenshots, validation and manual CI handoff. Remote CI for this checkpoint is **pending**, including Windows compilation; historical green runs do not validate this commit. The save procedure will locate the exact final pushed SHA's Actions run and hand it to the user without waiting for completion. No watcher, automation or another task is requested. Windows hardware/GPU, second-machine LAN, Mac-to-Windows, real internet and eight-human/latency limitations remain untested. This checkpoint is not fully validated while CI is pending. Next: the user monitors both CI jobs; the web-to-native feature inventory is later work.
