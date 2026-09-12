# 06 — First gameplay visual scene

2026-09-12. The existing native arena now has an illustrated field pilot, articulated movement/aiming, distinct pistol/M416 artwork, consistent terrain and landscape depth, bounded confirmed effects, and a readable native HUD. The user subsequently approved the visual result after playtesting; see the dated addendum below. The compact-HUD review finding is addressed in the follow-up below; final review subsequently passed as recorded in the checkpoint addendum below. Windows hardware validation remains open.

## Direction and scope

Read AGENTS, project context, roadmap, README and all five prior handoffs. Both repositories were clean at inspection. Native baseline: `c00adcf48b529343031a201dafeca8c67cb52b09`; browser reference: `7a398d4abefa8144fa8949998de4cd76e5dacf8a`. Edits were prepared in a temporary isolated copy, with a SHA-256 ledger checked against the native checkout before applying changed/new files. Existing files were not reset or overwritten blindly.

The user's revised order is recorded in the roadmap and context: gameplay visuals precede eight-player networking. No new maps, collision shapes, customization system, menus, audio, accounts, Go services or hosting were added. No agents, purchases, deployment, commit or push.

Primary visual evidence was the browser's actual `34-character-field-detail.png`, `38-character-jet.png`, and `ui-refresh/practice.png`, together with its character, weapon artwork and renderer source. The actual native placeholder window was also inspected. The browser's expressive large head, field cap, sage gear, substantial boots and simple outlined weapons informed the new art. Its crowded UI was not carried over. See [VISUAL_DIRECTION.md](../VISUAL_DIRECTION.md) for palette, proportions, composition and limits.

## Implementation and files

| File | Purpose |
| --- | --- |
| `crates/client/src/artwork.rs` | Original polygon drawings rasterized once at startup into one 1024 × 1024 RGBA atlas, with 4× source pixels per world unit. Head, torso, pack, limbs, boots, hands, both guns, magazine, badge and exhaust share the texture/layout. No frame-time decoding. |
| `crates/client/src/pilot.rs` | Shared practice/online pilot renderer: 24 persistent part entities plus one label per actor. Fixed-length leg/arm joints, displacement-driven planted/recovery feet, idle breathing, airborne tuck, jet pose, aiming, magazine motion, confirmed bolt motion, local hit tint and a short fading death fold. No body scaling or weapon-pivot bob. |
| `crates/client/src/terrain.rs` | Draws the existing arena solids, lit collidable top edges, inset stone facets, range sign, sparse distant vegetation, moon and three parallax mountain layers. Scenery stays behind gameplay with lower contrast. |
| `crates/client/src/combat_view.rs` | Preserved pointer projection/input mapping; capped pending/active confirmed shots, pooled effect entities, reticle and lifecycle cleanup. Replaces per-frame effect entity creation/destruction. |
| `crates/client/src/hud.rs` | Health/fuel bars and numeric states, magazine/reserve, reload/equip/empty state, opponent/connection/errors, death/respawn/pause and compact guidance. Compact layout now uses short corner readouts and collapsed F1 guidance; see the review follow-up below. |
| `crates/client/src/online.rs` | Exposes already received/interpolated remote movement and generation to the renderer. Adds a short jet pulse only to the opt-in native smoke route. Transport, prediction, deduplication and live input rules are retained. |
| `crates/client/src/main.rs` | Registers presentation systems and removes old rectangle/debug-HUD rendering. Existing device capture, simulation stepping and camera math are preserved. |
| `crates/client/src/frame_profile.rs` | Optional 1800-frame wall-interval probe after 180 warmup frames. Console output only. |
| `crates/client/src/review.rs` | Optional bounded native framebuffer captures by state and up to 20 F9 captures. Writes PPM without adding image encoder dependencies. Disabled unless a local capture directory is supplied. |
| README, roadmap, context, visual direction, this handoff, screenshot index | Run/review instructions, revised ordering and evidence. |

### Artwork and asset provenance

All added art is authored as code for this milestone. The user's browser character and weapon drawings are the visual/proportional reference; no browser file was modified or copied into the runtime. There are no downloaded or generated third-party assets, models, textures, fonts or sounds. Bevy's already bundled default Fira Mono font is reused under its existing distribution/license; no font asset or dependency was added. The six PNGs are captures of the actual new native Metal renderer, losslessly encoded from framebuffer PPM data with Python's standard-library zlib. They are evidence, not runtime assets or concept images. Total screenshot size is about 545 KiB.

### Gameplay preservation

A hash comparison verifies all **26 protected core/protocol/server, manifest, lockfile and toolchain files** are unchanged from the clean baseline, including the frozen approved-practice fixtures. No movement, combat, map, spawn, timing, protocol or dependency tuning changed. The full 12,000-tick approved-practice equivalence test still passes. Client pointer inversion, input capture, fixed-step simulation and camera functions retain their original logic.

The visible pilot fits the existing 36 × 68 body with soles on its bottom edge and the cap at its top. The weapon pivot remains the approved body center. Artwork uses the existing barrel proportions; a read-only core ray/terrain query clips a barrel that would protrude through nearby cover. Confirmed tracers start at `Shot.origin` and end at `Shot.end`; the small origin flash is confined to that confirmed segment. Impacts use its confirmed outcome. No cosmetic barrel origin can bypass a wall or apply damage.

Reload feedback reads authoritative remaining ticks. Health, weapon state and online hits remain server-owned. Remote posing reads the existing interpolated actor's full movement state, including thrust. Existing monotonic shot deduplication remains in the unchanged protocol client; reconciliation does not replay cosmetic shots.

### Bounds and cleanup

At most **24 pending** and **24 active** confirmed shots are retained. The render pool has **120 persistent sprites**: tracer, origin flash and three impact sparks per active slot. Tracers last at most 85 ms, origin flashes 45 ms and impacts 180 ms. Each pilot has two state-driven exhaust cones; no particle emission queue. No bloom, full-screen flash or screen shake.

Reset advances a presentation epoch; death/life, actor presence, join generation and large presentation discontinuities clear old pose/effect history. Respawn and reused slots reset the rig; disconnect hides actor parts, labels, bars, weapon and exhaust. Confirmed hit tint is local to the pilot and lasts at most 120 ms. Atlas, meshes, materials and entities are created at setup and reused. The atlas is 4 MiB of RGBA pixels before engine CPU/GPU copies; no VRAM or total memory benchmark was performed.

## Verification

Host: Apple M1 Pro, macOS 26.6.2 (25G83), `aarch64-apple-darwin`. Both baseline and final renderer logs identify the **Metal** adapter. Windows was not compiled or run for these uncommitted changes; previous checkpoint CI is historical evidence only.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | **88 passed**, zero failed/ignored: 24 client, 44 core, 14 protocol, 6 server/transport; doc-test harnesses passed. |
| `cargo build --workspace --locked` | Passed |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Passed |
| Independent core/server checks and dependency boundary | Passed; core has zero dependencies; server has no Bevy/window/GPU dependencies. |
| Added meaningful tests | State priority; planted-foot displacement and sole bounds; constant leg lengths throughout a stride; all-angle arm lengths with both weapons/reload phases; nearby-wall artwork clearance; pose reset across absence/generation/reset; pending/active effect bounds/expiry and lifecycle cleanup. |
| Original native combat route | All **7** checkpoints passed in the actual renderer: pistol held-fire kill/reload, bot respawn, moving rifle kill/reload, player death/respawn and F5 reset. |
| Original native movement route | All **8** checkpoints passed: underside/floor contact, ceiling and empty fuel, held-jet regeneration gate, reactivation, right wall, platform landing and reset. |
| Actual online roles | Two separate native clients against localhost UDP both logged completion for movement, jump, jet, fire, kill, death and respawn. Foreground inspection confirmed both assigned roles and local/opponent label/color mapping. Framebuffer evidence shows both local and delayed remote jet exhaust. |
| Native device checks | Mouse shots with pistol/M416, terrain impacts, R reload, 1/2 selection, A/D and Space taps, F5 reset, and fresh input after focus recovery. Sustained holds were supplied by scripted routes, not claimed as physical input. |
| Resize/Retina | Actual 1280 × 720, 800 × 524 and **480 × 320** logical viewports at **2×** scale. Initial medium-size review caught HUD overlap; the later compact-HUD review found the minimum viewport still obscured upper gameplay. The follow-up below supersedes that initial layout assessment. Existing automated projection tests retain 1×/2× coverage. |
| Focus recovery | Finder took focus; native UI showed PRACTICE PAUSED. Movement-route log retained tick **4428** across loss/return, then fresh D/Space and firing worked. Online foreground changes showed input-released status while the server continued. |
| Departure/fresh join | Closing player Two removed its parts/bar/label from One. A fresh actual client acquired slot Two with **generation 3**, 100 health/fuel and 12+48 pistol ammunition; One showed a fresh upright opponent. |
| Abrupt server loss | Stopping the test server produced the existing five-second no-snapshot disconnected reason plus restart guidance, with actor visuals hidden. |
| Shutdown | Test clients/server closed. The existing Bevy/Winit `Skipped event Destroyed for unknown winit Window Id` warning appeared on close, as in earlier milestones; no renderer panic observed. |

Native routes inject ordinary input commands into the existing client/server path. They do not teleport actors, inject fixture damage or replace rendering. Tests and snapshots establish these concrete outcomes; they are not human approval of animation feel. The desktop capture tool occasionally returned stale occluded-window frames or stream/startup errors. Foreground refresh and native framebuffer captures were used to verify the current state; one reopen took several minutes in the inspection tool while the game continued running. Temporary wrappers, raw captures and logs remain outside the deliverable.

### Frame timing observation

Same Mac/Metal adapter, development optimization profile, 1280 × 720 logical / 2560 × 1440 physical window, AutoVsync, focused offline spawn with no input and the ordinary bot active. The unchanged baseline was built with the same optional probe. Each sample discards 180 startup frames and records the next 1800 `Time<Real>` deltas, sorted for quantiles. Screenshot capture was disabled. Other test clients were closed; the final test server was stopped near the beginning of the final sample.

| Frame interval | Placeholder baseline | Final scene |
| --- | ---: | ---: |
| Mean | 8.371 ms | 8.353 ms |
| Median | 8.301 ms | 8.298 ms |
| p95 | 14.956 ms | 11.833 ms |
| p99 | 18.984 ms | 19.779 ms |
| Frames over 16.7 ms | 56 / 1800 | 45 / 1800 |

The medians are effectively alike; tail variation goes in both directions. These are one short before/after sample on a normal desktop, including VSync, OS scheduling, rendering and presentation. They are not isolated GPU/CPU pass timings, a statistically controlled benchmark, a worst-case moving-combat test, an FPS guarantee or evidence for Windows performance. Repeat under controlled sustained combat before setting targets.

## Screenshots and remaining limits

[Six representative native screenshots](../screenshots/06-gameplay-visuals/README.md): initial range, both online jets, rifle reload, confirmed online rifle hit, death/respawn message, and the minimum viewport. Full-resolution captures are retained at about 545 KiB total.

- General user visual playtest approval is recorded below. Individual checklist results were not enumerated. Tiny pilots at the minimum window retain the approved camera/world scale; changing that scale is outside this visual milestone.
- Windows compilation/CI for these exact changes, Windows GPU/hardware, a Mac-to-Windows match and internet conditions were **not tested**. No build success is inferred from the prior 80-test checkpoint.
- Physical sustained Shift/mouse, held-input focus transitions and cursor exit still need the human checklist; automation lacks controlled standalone modifier/mouse holds. Existing input tests and rendered scripted holds remain separate evidence.
- Death is a small fading pose, not a ragdoll. Reload is magazine/hand and HUD feedback, not a detailed weapon mechanism. No full customization or audio pipeline.
- Eight-player work, new maps, full menus/settings/Host/Join, accounts and hosting remain deferred.

## Run and short user playtest

```sh
cd /Users/ayushrameja/codebase/LHP/burnhop-native
cargo run -p burnhop-client --locked -- --offline
```

For two players, run `cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000`, then two separate `cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000` processes. The server stays running. Full commands and optional rendered routes are in README.

1. Walk forward and backward while aiming up/down; watch the soles, knees and hands. Jump and hold/release Shift; compare boot exhaust with fuel state.
2. Fire both weapons, reload and switch mid-reload. Shoot a nearby platform/wall and check the visible impact agrees with cover. Keep the approved aiming feel as the reference.
3. Take damage, die, respawn and press F5 in practice; look for leftover parts, flashes or thrust. Resize down to 480 × 320 and back.
4. In both online roles, watch the other pilot move/jet/reload. Close one client and join again. Switch focus while holding controls, release outside, then return and press fresh controls.
5. Report one action and one observation, ideally with a screenshot. The most useful next improvement is the one you notice repeatedly while playing.

## Subsequent user-reported approval — 2026-09-12

The user reports completing the playtest and says the result looks good. This records **user-reported visual/animation approval** of the gameplay visual milestone. Preserve this visual direction and the previously approved movement/combat feel as the baseline for later work.

This is user-reported evidence, separate from the automated tests and native checks above. The report does not specify platform, offline/online mode or individual checklist outcomes; Windows validation and unreported physical-input subcases remain separate checks. This update changes documentation only and does not authorize a commit, push, deployment or eight-player implementation.

## Compact-HUD review follow-up — 2026-09-12

**Status: review finding addressed; ready for final review before a separately authorized save and CI run.** The user's completed playtest and “looks good” feedback remain **user-reported visual/animation approval**. They do not establish Windows execution, online/offline mode, or any individual physical-input checklist result.

### Correction and preservation

The original [minimum-window capture](../screenshots/06-gameplay-visuals/06-minimum-window.png) exposed two 222-pixel panels at `top=82`, taking almost the full width and hiding an upper platform. That image is retained as historical review evidence, not the final compact layout.

Only `crates/client/src/hud.rs` changes runtime behavior in this follow-up. At compact sizes, the vitals panel is 132 logical pixels wide at left=6/top=26; weapon/ammo is 164 wide at right=6/top=6. Their middle gap is 172 pixels at the minimum width, and both end above the former 82-pixel obstruction. Padding drops to four pixels, redundant headings/mode/score text and bars are omitted, and health/fuel/ammo keep 13/12/16-pixel text. Low health, low/empty fuel, magazine/reserve, reload countdown, equipping, inactive, empty/reload and empty/switch states still use explicit words/numbers.

The compact bottom edge contains a short opponent/connection status and **F1 Controls**. Native platform captures caught a bottom-status overlap during iteration; removing its padding and putting it two pixels from the bottom keeps the landing pilots' feet clear. F1 toggles a readable three-line guide; focus loss or a desktop/compact round trip collapses it. It does not consume or clear movement commands. Full terminal connection reasons and restart guidance wrap once in the central overlay; death/respawn and pause still take priority over the guide. The guide intentionally covers some lower gameplay while explicitly open.

The existing 1280 × 720 layout, bars, desktop copy, typography and placement are retained. Compact mode also applies below 500 logical pixels high so a wide-but-short window does not inherit large desktop panels. No artwork, animation, terrain, camera scale/projection, collision, weapon tuning, simulation, networking, dependencies or browser files changed. All pre-existing uncommitted visual work and the original six screenshots are preserved.

### Follow-up verification

Actual Apple M1 Pro / macOS native **Metal**, 2× Retina: **480 × 320**, **800 × 524**, and **1280 × 720** logical viewports (960 × 640, 1600 × 1048 and 2560 × 1440 framebuffers). Reviewed native windows and original framebuffer pixels, not HTML or layout mockups.

| Check | Result |
| --- | --- |
| Formatting | `cargo fmt --all -- --check` passed. |
| Strict lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` passed. |
| Tests | `cargo test --workspace --locked`: **90 passed**, zero failed/ignored (26 client, 44 core, 14 protocol, 6 server/transport); doc-test harnesses passed. |
| Locked build | `cargo build --workspace --locked` passed on the Mac host. |
| Focused regressions | Disclosure starts closed, toggles once per press, resets on focus loss/desktop, and restores desktop nodes on resize. The real Bevy input/HUD systems retain held movement through F1. Exhausted resources still expose LOW/EMPTY and weapon-switch guidance with help closed. |
| Native movement at all three sizes | Original **8 checkpoints passed at each size**, including platform underside/landing, ceiling, empty fuel, hold/repress behavior, right wall and reset. Additional ordinary input commands flew along both left and right edges; captured x=0 and x=2364 while thrusting near y=699. Upper-platform and ceiling views retain visible actors and open gameplay between the corner readouts. |
| Native combat at all three sizes | Original **7 checkpoints passed at each size**: pistol/rifle held fire and reload, moving fire, kills, death/respawn and reset. Reviewed normal, reload and death readouts. |
| Native connection failures at all three sizes | Actual localhost connection attempts without a server produced the full five-second no-authoritative-snapshot error plus restart guidance. All text remained within the viewport; compact status did not duplicate the full error. |
| Native F1 | Opened, captured and closed the guide at 480 × 320; the collapsed hint returned. Sustained movement preservation is separately covered by the Bevy regression, not claimed as a physical held-key test. |

Inspection used a temporary source copy and local `.app` launchers for exact initial window sizes. The copy used the unchanged simulation/input path, with additional movement commands and bounded framebuffer capture triggers for ceiling, empty fuel, platform landing and both side-edge jets. These inspection-only changes were removed before the final lint/test/build; no fixture damage, teleports, alternate physics or image edits were used. Captures were losslessly encoded from PPM to PNG. The delivered runtime correction is confined to `hud.rs`; README, direction/context/roadmap, this handoff and the screenshot index record the result.

### Evidence and remaining limitations

Updated screenshots **07–22** are linked in the [screenshot index](../screenshots/06-gameplay-visuals/README.md). Start with **07 minimum platform**, **08/09 edge jets**, **10 empty fuel at ceiling**, **12 reload**, and **14 connection error**. Medium and desktop evidence follows in the same index. The older 06 image remains available for direct before/after review.

Windows compilation/CI for this exact working tree, Windows hardware/GPU, a Mac-to-Windows match, real internet conditions and the unreported physical held-input/focus/cursor-exit cases remain unverified. The temporary scripts establish rendered outcomes, not human animation-feel approval. The approved camera scale still makes pilots small at 480 × 320; the short corner HUD and opt-in help still occupy some pixels, so this is not a promise of zero occlusion at every possible actor/camera position. No new performance benchmark was run. Final review of this correction remains before save/CI. No agents, commit, push, deployment or purchase were used.


## Reviewed save checkpoint addendum — 2026-09-12

The current checkpoint brief records that the compact-HUD correction **passed review**, following the user's earlier gameplay visual playtest approval. This supersedes the historical final-review-pending statements above. The user separately authorizes committing and normally pushing the approved milestone to `git@github.com:ayushrameja/burnhop-rust.git` and running GitHub Actions. Earlier no-commit/no-push statements describe their original task scope.

Fresh checkpoint checks passed: formatting, strict workspace Clippy with all targets, all **90 tests** (26 client, 44 core, 14 protocol, 6 server/transport; zero failed/ignored), locked host and explicit `aarch64-apple-darwin` workspace builds, and independent core/server checks. All 26 protected core/protocol/server, manifest, lockfile and toolchain files match baseline `c00adcf48b529343031a201dafeca8c67cb52b09` byte for byte; the unchanged CI workflow is a 27th verified file. Input capture, aim projection/capture and practice simulation functions are unchanged. The core has zero dependencies; the server dependency tree has no Bevy/window/GPU packages.

Reviewed client visual source, compact-HUD regressions, documentation and all 22 indexed PNG evidence files for commit scope and artifacts. No credential-pattern matches, executables or symlinks were found among the 37 candidate files. Selected current minimum-platform and connection-error screenshots were inspected again; this is screenshot review, not a new native or human hardware playtest. Original captures, including the obstructed HUD image, remain historical evidence. Raw captures, logs, app wrappers and build output are excluded.

Exact pushed-commit macOS ARM64 / Windows x64 CI will be recorded in the visual checkpoint after the implementation run completes. This checkpoint makes no new hardware, human input, cross-platform multiplayer or internet claim. Windows CI compilation and headless tests cannot validate Windows GPU rendering or human playtest feel.
