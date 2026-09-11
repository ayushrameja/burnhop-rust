# 01 — Native foundation handoff

Completed 2026-09-11. Bounded foundation implementation is ready for review. Windows build/window validation remains outstanding; this is not a complete native rewrite or playable movement milestone.

## What changed

- Inspected `AGENTS.md`, project context, roadmap, README, Git status and remote before editing. Native repository initially contained documentation only and had no uncommitted changes. Existing instructions and project direction were preserved.
- Pinned stable Rust 1.98.1 (edition 2024, rustfmt, Clippy) and exact Bevy 0.19.1; generated `Cargo.lock`. Official version sources and compatibility rationale are linked in [README](../../README.md).
- Created a two-member Cargo workspace: `burnhop-client` and `burnhop-gameplay-core`. Core has no dependencies and only declares the planned 60 Hz tick frequency. There is no simulation yet. Client reads that constant for its startup log; future rules can be reused by a Rust server without Bevy.
- Client opens a resizable native 1280 × 720 window with VSync, `Camera2d`, dark background, three gray platform shapes, and a stationary orange 36 × 68 placeholder. Scene is generated from sprites and requires no asset files. Geometry is decorative, not collidable.
- Disabled Bevy's broad default feature set; enabled native windowing, threading/logging, and the sprite renderer. No extra direct third-party dependencies, custom linker, dynamic linking, or runtime shell.
- Added development optimization levels (own code 1, dependencies 3), inherited unsafe-code prohibition and Clippy lints, and macOS ARM64 / Windows x64 build-check CI.
- Documented browser movement/collision/camera/map findings in [REFERENCE_GAMEPLAY.md](../REFERENCE_GAMEPLAY.md), exact setup/run/check commands in README, and updated context/roadmap.

## Verification performed

Local hardware: Apple M1 Pro MacBook Pro, 10 CPU cores, 16 GPU cores, 16 GB RAM. macOS 26.6.2 (25G83); host target `aarch64-apple-darwin`; Xcode Command Line Tools available. Hardware identifiers are intentionally omitted.

| Check | Actual result |
| --- | --- |
| `rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy` | Installed successfully; rustc 1.98.1, commit 48a229cea (2026-09-01) |
| `cargo build --workspace` | Full native compile and link passed; first build reported 4m 16s on this machine (not a performance benchmark) |
| `file target/debug/burnhop-client` | Mach-O 64-bit executable arm64 |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings |
| `cargo test --workspace --locked` | Passed: both unit-test harnesses and core doc-test harness contain **zero tests**; no gameplay coverage is claimed |
| `cargo check -p burnhop-gameplay-core --locked` | Passed independently |
| `cargo tree -p burnhop-gameplay-core --locked` | Only the core package; zero dependencies |
| `cargo build --workspace --locked` | Passed using the generated lockfile |
| `cargo run -p burnhop-client --locked` | Executed the client; process ran until test SIGINT and exited 0 |
| Actual native window/rendering | Visually verified the same built executable through the local app wrapper described below; expected title, dark background, all three platform shapes and orange placeholder visible |
| Native close button | Clicked the window close button; process absence confirmed afterward |
| CI on GitHub | **Not run**. Workflow authored locally; no commit/push or CI dispatch performed |
| Windows compilation / launch / hardware | **Not performed**. No Windows success claimed |
| Gameplay / frame-time / networking tests | Not applicable to this placeholder; none performed |

### Local window-inspection detail

The desktop inspection tool could not select an unbundled `cargo run` executable by name/path. A brief process stack sample confirmed the app was running the Winit/AppKit loop and Metal renderer rather than stalled at startup. This environment inherited `RUST_LOG=warn`, which hid the normal informational startup log.

To visually inspect the same binary, created a **local-only ignored** wrapper at `target/smoke/Burnhop Foundation.app`, containing a minimal `Contents/Info.plist` and a `Contents/MacOS/burnhop-client` symlink to `target/debug/burnhop-client`. The native UI tool opened that wrapper and captured the actual scene. It is an inspection convenience, not a signed application, installer, shipping format, or required runtime dependency. Ordinary development still uses the README's `cargo run` command.

The direct-run SIGINT shutdown emitted one `bevy_winit::state` warning: `Skipped event Destroyed for unknown winit Window Id`, and exited 0. No startup/render failure was observed. Normal window close also ended the wrapper process. The inspection tool automatically reopened the app when asked for post-close UI state, so closure was verified by checking process absence without another app inspection afterward. No test game process was deliberately left running.

## Boundaries and limitations

Browser reference `../burnhop` remained clean and untouched. No assets were copied, no existing code was removed, and nothing was committed, pushed, deployed, or purchased. There are no movement, collision, combat, networking, accounts, Go services, transport choices, or backend dependencies in this milestone.

CI uses `macos-14` / `aarch64-apple-darwin` and `windows-2022` / `x86_64-pc-windows-msvc`. Its formatting, strict Clippy, test-harness and executable-build jobs use the pinned toolchain and locked dependencies. A future passing CI run will prove those build checks, not a visible GPU window or human playtest. Windows ARM, distribution, signing, performance targets, and cross-platform deterministic simulation remain outside the validated scope.

## Next task: movement in one arena

1. Read context/roadmap and the reference notes. Choose and document the simulation coordinate convention; proposed: retain browser Y-down world units and convert to Bevy Y-up at the client boundary.
2. Add plain state, explicit input commands and a 60 Hz step function in the core, starting with the practice range's four rectangles and boundaries. Keep window input, camera and rendering out of the core.
3. Implement horizontal acceleration/braking, gravity, swept rectangle collision, jump/coyote/buffer behavior and jet fuel transitions. Add behavior tests for thin-platform tunnelling, walls/ceilings, ledges, input transitions and repeatable command replay before expanding map complexity.
4. Connect native input, fixed stepping and visual interpolation; add a following camera with bounded arena framing. Make one arena feel good before importing Outpost polygons or artwork. Do not add combat or services to this task.
5. Run build checks and an actual movement playtest on Apple Silicon and Windows. Record OS/GPU, results, and outstanding limitations separately from CI. Run the newly authored CI once changes are reviewed and a later user-authorized push occurs.
