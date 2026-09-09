# Roadmap

## Completed

- [x] Create a separate project folder and local Git repository.
- [x] Record the agreed direction and agent continuity instructions.

## 1. Native foundation and movement

- [ ] Inspect the reference movement, camera, collision, and map data.
- [ ] Verify compatible Rust/Bevy versions and establish a minimal Cargo workspace.
- [ ] Separate gameplay simulation from client rendering.
- [ ] Open the game on Apple Silicon macOS and Windows.
- [ ] Add one arena, a controllable character, jump, jetpack, and camera.
- [ ] Compare movement feel to the browser reference; record hardware and checks.

## 2. Small combat loop

- [ ] Add two weapons, hit resolution, health, death, and respawn.
- [ ] Add a readable minimal HUD and practice flow.
- [ ] Verify important collision and gameplay behavior.

## 3. Authoritative multiplayer

- [ ] Select networking transport/library and document the decision.
- [ ] Run the shared gameplay core in a headless Rust server.
- [ ] Connect two clients with prediction, reconciliation, and remote presentation.
- [ ] Test eight players, jitter, packet loss, disconnects, and frame/server timing.
- [ ] Validate an actual Mac-to-Windows match.

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

2026-09-09: Repository documentation only. No runtime code or tests exist yet. Next action: inspect reference gameplay and select the minimal Rust/Bevy foundation. No performance claims have been validated for the native project.
