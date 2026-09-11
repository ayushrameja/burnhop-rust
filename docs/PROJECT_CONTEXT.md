# Project context

Last updated: 2026-09-11

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
- The user reports personally playtesting and approving both Mac movement and combat feel with placeholders (recorded as user-reported evidence in `docs/handoffs/03-combat.md`). No weapon or movement tuning was changed for multiplayer. `MatchState` now contains reusable actors; `step_match` owns both players’ movement, weapons, rays, damage and lifecycle. `step_practice` is an adapter to the same rules with the existing stationary bot policy. The 12,000-tick pre-refactor practice command stream is preserved as an equivalence regression. After Windows CI exposed a platform-specific raw trace hash, the test now compares every tick against frozen approved source on each platform and retains the original Mac golden; runtime rules are unchanged. See `docs/handoffs/04-multiplayer.md`.
- Two-player direct connect now uses a headless Rust server at 60 Hz, explicit assigned actors and scheduled input sequences, bounded queues, 30 Hz authoritative snapshots, local movement prediction/reconciliation and remote interpolation. Core has zero dependencies; transport and the fixed bounded codec live in a separate protocol crate. Native focus loss sends neutral input without pausing the match. No hit rewind, automatic reconnect or world reset online. Asset representation, deployment provider and distribution/update mechanism remain to be evaluated.
- The checkpoint brief reports user participation in a successful two-player multiplayer playtest and approval of the reviewed milestone. This is user-reported evidence; network conditions, platforms and individual physical input/focus subcases were not supplied. See the dated addendum in `docs/handoffs/04-multiplayer.md`. Automated tests, CI compilation and hardware/internet validation remain separate evidence.
- The reviewed milestones and test-only portability fix are committed and pushed to `main`. GitHub CI passed formatting, strict Clippy, all 80 tests and locked executable builds on macOS ARM64 and Windows x64 for `de704556391dae3f58ffa83aba17160691535342`. See `docs/handoffs/05-checkpoint.md` for both CI run links and the original Windows regression failure. CI is not Windows hardware/GPU or live cross-platform/internet playtesting. Verify each later commit's own CI status.
- Hardware performance targets should be chosen and measured before making performance promises.
- No production deployment, server relocation, or paid service has been configured as part of this setup.
- The user created the remote repository `git@github.com:ayushrameja/burnhop-rust.git` and authorized syncing this local project to it.
- The working directory name is `burnhop-native`; the public game name need not change.

## How to resume

Read this file, the roadmap and `docs/handoffs/05-checkpoint.md` for the saved milestone/CI checkpoint. Inspect the existing game's movement and map boundaries as needed. The movement milestone implements one arena, a controllable character, jump/jet fuel, rectangular collision, camera and behavior tests. Read `docs/handoffs/04-multiplayer.md` for the direct-connect architecture, actual Mac checks and remaining human/platform checks; `03-combat.md` records the approved practice combat loop; `02-movement.md` records the user's movement approval and earlier evidence. Read `docs/REFERENCE_GAMEPLAY.md` before porting tuning or map data. Do not start with a complete account system or a full port of all cosmetics.
