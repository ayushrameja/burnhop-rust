# Burnhop Native

A playable native Rust + Bevy combat practice loop for Burnhop: mouse aiming, pistol and M416, reloading, an attacking stationary bot, health, death and respawn, alongside the approved jumping/jet movement and following camera. Graphics are placeholders. A two-player direct-connect mode now runs the same rules in a headless Rust server, with local movement prediction and remote interpolation. Accounts, matchmaking, internet hosting setup and Go services remain deferred.

Read [project context](docs/PROJECT_CONTEXT.md), [roadmap](docs/ROADMAP.md), [browser behavior notes](docs/REFERENCE_GAMEPLAY.md), [multiplayer handoff](docs/handoffs/04-multiplayer.md), [combat handoff](docs/handoffs/03-combat.md), and [movement handoff](docs/handoffs/02-movement.md) (and the earlier [foundation handoff](docs/handoffs/01-foundation.md)). Agents must first read [AGENTS.md](AGENTS.md).

## Pinned versions and choices

- Rust **1.98.1**, edition 2024, pinned in `rust-toolchain.toml` with rustfmt and Clippy. The [official stable release announcement](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/) was verified on 2026-09-11.
- Bevy **0.19.1**, exact dependency constraint `=0.19.1`. Its [official release manifest](https://github.com/bevyengine/bevy/blob/v0.19.1/Cargo.toml) specifies Rust **1.95.0** as minimum; our toolchain exceeds it. `Cargo.lock` locks transitive packages and is included in version control for reproducible application builds.
- The client enables native windowing, sprite rendering, and Bevy UI with its bundled default font for the controls/fuel display. UI/font support adds 49 locked transitive packages; no existing pinned Rust/Bevy version was changed. No third-party physics engine is needed, and gameplay-core still has zero dependencies.
- Development code uses optimization level 1; dependencies use level 3, following [Bevy's setup guide](https://bevy.org/learn/quick-start/getting-started/setup/). The first build is substantial; incremental builds reuse it. Keep normal static linking and platform linkers for a straightforward launch.

## Setup on Apple Silicon macOS

Install Xcode Command Line Tools if missing:

```sh
xcode-select --install
```

Install Rust using [rustup's official installer](https://rustup.rs/), then open a new terminal. From this checkout:

```sh
cd /Users/ayushrameja/codebase/LHP/burnhop-native
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
rustup target add --toolchain 1.98.1 aarch64-apple-darwin
cargo run -p burnhop-client --locked
```

Run from an interactive desktop session with GPU access. `rustc -vV` should report host `aarch64-apple-darwin`. The toolchain file selects this version automatically inside the repo; it does not change your global Rust default.

## Setup on Windows

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++**, the MSVC compiler, and a Windows SDK, as described in [Bevy's platform prerequisites](https://bevy.org/learn/quick-start/getting-started/setup/). Install Rust from [rustup](https://rustup.rs/) and open a new PowerShell terminal. Use the MSVC host toolchain, not GNU. In your local checkout (replace the example path):

```powershell
cd C:\code\burnhop-native
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
rustup target add --toolchain 1.98.1 x86_64-pc-windows-msvc
cargo run -p burnhop-client --locked --target x86_64-pc-windows-msvc
```

Windows x64 is the initial Windows build target. Windows ARM and installers are outside this milestone. Installing a Windows Rust target on a Mac is not equivalent to linking or running a Windows application.

## Offline practice and direct-connect multiplayer

Build once, then start the server and each client in **three separate terminals** from this repository:

```sh
cargo build --workspace --locked
# Terminal 1: headless, no window or GPU
cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000
# Terminal 2: first native player
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
# Terminal 3: second native player
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
```

These commands also work in separate PowerShell terminals on Windows after the setup above; Windows execution is still unverified. On an already reachable LAN, bind an explicit interface address (for example `--bind 192.168.1.20:5000`) and pass that same server address to each client. Numeric IP addresses with ports are required; IPv6 uses `[address]:port`. The recorded agent transport checks exercised localhost; the user-reported multiplayer approval does not specify network conditions. No automatic firewall changes, NAT traversal, relay, room discovery or deployment is included. This uses Netcode's unsecure development authentication; it is a trusted direct-connect milestone, not authenticated public hosting.

Both clients need the same protocol and gameplay versions. The HUD shows connecting, assigned player, connected, disconnected and compatibility-error states. You are orange; the opponent is red. There is no online bot. Close a client to leave; a new process can join the freed slot. A disconnected client must be restarted to join again. F5 is ignored online, and losing focus clears input while the match and reload/respawn timers continue.

Offline practice remains the default, or can be selected explicitly:

```sh
cargo run -p burnhop-client --locked -- --offline
```

To reproduce the native multiplayer smoke route, add `--online-playtest` to **both** client commands. Once both join, each moves/jumps, fires, kills, dies and respawns using real UDP traffic and the shared authoritative rules. Watch for `ONLINE PLAYTEST COMPLETE` in both terminals. These are actual rendered clients with **injected commands**, including background input for this explicit test mode; this is not a human playtest. A key/click cancels the route. The ordinary mode clears input when unfocused. Add `--diagnostics` to the server command to log joins, leaves, confirmed combat and overload.

The first protocol uses six ticks of input lead and six ticks of remote display delay, with no hit rewind. High latency, loss or stalls can produce corrections and missed short actions. See the [multiplayer handoff](docs/handoffs/04-multiplayer.md) for exact scheduling, bounds, delivery rules, known limits and the short human checklist.

## Controls and offline practice

| Key | Action |
| --- | --- |
| A / D | Move left / right; holding both cancels directional intent |
| Space | Jump; a late ledge press still jumps, and a tap just before landing queues a hop |
| Left or right Shift | Hold for direct jet thrust, including takeoff from the floor |
| Mouse | Aim; hold left button to fire either weapon at its own cadence |
| R | Reload selected weapon; empty magazines do not reload automatically |
| 1 / 2 | Select pistol / M416; changing weapons cancels reload and takes 0.3 s |
| F5 | Offline only: reset both actors, health, ammunition, fuel, counters and input; release/repress controls afterward |
| Window close button | Quit |

Space never activates the jet in this milestone. Holding Space does not repeat jumps. Shift must be released and pressed again after landing or exhaustion; regenerated fuel does not restart a held jet. Both Shift keys share one hold. Losing focus pauses the encounter and clears held/queued input; click back into the window and press controls again. Leaving the window with the cursor stops firing; click again after returning. Invalid cursor positions cannot shoot. Death and respawn also clear controls. Resize freely; the arena/collision dimensions never change.

The teal floor mark is player spawn. The red bot stands at the browser target spawn and fires a pistol once per second while you are alive, visible and within range. It waits three seconds after reset or either respawn; it also reloads. Both actors have 100 HP and respawn after three seconds. Your pistol starts with 12 + 48 rounds; M416 starts with 30 + 120. F5 or player respawn refills both. Shots lose damage with distance, stop at terrain, and cannot hit the shooter.

Jump alone rises about 86 world pixels; the first platform is 260 pixels above the floor, so use the jet and steer around platform edges. Platforms are solid on every side.

Optional reproducible rendered combat check (about 22 focused seconds):

```sh
cargo run -p burnhop-client --locked -- --combat-playtest
```

It drives aiming, sustained fire, reloads, switching, moving fire, bot/player death and respawn, and F5 reset through the input buffer. Keep the window focused without pressing keys/clicking until `COMBAT PLAYTEST COMPLETE` (seven checkpoints). Native device input or focus loss cancels it. This is scripted evidence, not a human playtest. `--combat-diagnostics` instead leaves normal device controls active and logs tick commands and confirmed combat events. Default runs do not print those diagnostics.

The earlier movement route remains available (about 21 seconds), with bot combat suppressed during that route:

```sh
cargo run -p burnhop-client --locked -- --movement-playtest
```

Keep the window focused without pressing keys until it prints `PLAYTEST COMPLETE` (eight checkpoints). It exercises platform undersides, floor/platform landings, the arena ceiling/right wall, fuel exhaustion/regeneration/reactivation, and reset through the same input buffer. Keyboard input or focus loss cancels the route and returns control to you. This is a scripted smoke check, not a human movement-feel test. See the handoff's short manual checklist.

## Everyday checks (both platforms)

Run at the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
cargo run -p burnhop-client --locked
```

Use `cargo fmt --all` to apply formatting. `cargo test --workspace --locked` runs movement/combat, protocol validation, prediction/interpolation, input/focus and real localhost UDP tests without opening a window. The transport tests use ephemeral loopback ports; environments that prohibit local sockets need permission to run them. To check the core alone without compiling Bevy:

```sh
cargo check -p burnhop-gameplay-core --locked
cargo tree -p burnhop-gameplay-core --locked
```

The tree should contain only `burnhop-gameplay-core`. For an explicit platform build, append `--target aarch64-apple-darwin` on Apple Silicon or `--target x86_64-pc-windows-msvc` on Windows. Build artifacts are ignored under `target/`; a normal host build produces `target/debug/burnhop-client` (or `.exe` on Windows), and explicit-target builds use `target/<target>/debug/`.

## Layout and boundary

```text
crates/client/          Bevy device input, offline/online adapter, camera and HUD
crates/gameplay-core/   Dependency-free actors, movement/combat, 60 Hz ticks and collision
crates/protocol/        Wire codec, bounded input queues, prediction and Renet client adapter
crates/server/          Headless authoritative match and bounded server clock
```

The core has zero dependencies, no Bevy types, and no clocks or networking. `MatchState` contains two optional reusable actors; `step_match` advances movement and combat once per tick. `step_practice` adapts the existing player/bot practice state into those same rules. Both shot decisions are made before damage, so simultaneous kills are valid. The unchanged movement-only `step` also supports local prediction. Actor snapshots include full movement state, combat state, neutral-input gate, scores and join generation. Serialization and Renet 2.0.0 / renet_netcode 2.0.0 remain outside the core; the headless server has no Bevy dependency. Coordinates retain top-left, X-right/Y-down `f64` browser pixels; Bevy converts only for display.

## CI and validation limits

`.github/workflows/build.yml` installs the pinned toolchain and runs formatting, Clippy, tests, and executable builds on Apple Silicon `macos-14` and x64 `windows-2022` runners. These labels are listed in [GitHub's official runner images](https://github.com/actions/runner-images). CI does not open a window or establish game feel/GPU performance. See the handoff for actual local results; an authored workflow is not a passing CI run.

The browser project at `../burnhop` is a read-only reference and is not required to run this client. Remote: `git@github.com:ayushrameja/burnhop-rust.git`. The user reports that the prior Mac movement and controls feel correct with placeholders; this is user-reported approval. The user also reports approving Mac combat feel. The user also reports participating in a successful two-player multiplayer playtest and approving that milestone; no network conditions, platforms or individual physical-input checks are inferred. Milestone 4 separately records automated transport, actual two-native-client UDP and offline regression verification. Windows compilation/hardware and GitHub CI are pending checkpoint verification. Next: finish the repository checkpoint and CI, then document Windows hardware/GPU, Mac-to-Windows and real internet validation separately.
