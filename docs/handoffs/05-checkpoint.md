# 05 — Reviewed milestone checkpoint

Checkpoint date: 2026-09-11. Foundation, movement, combat and two-player direct-connect multiplayer are saved in the existing [Burnhop Rust repository](https://github.com/ayushrameja/burnhop-rust). No gameplay or tuning changes were made during this checkpoint.

## Repository and commit review

- Read AGENTS.md, project context, roadmap, README, all four prior handoffs and browser behavior notes before recording the checkpoint.
- Verified both origin URLs are `git@github.com:ayushrameja/burnhop-rust.git`. Fetched origin: local `main` and `origin/main` were both `b032fe8`, with zero commits of divergence. GitHub reported default branch `main`, administrator access, no active rulesets for this branch and no branch protection. Actions was enabled; normal direct push followed the existing repository rules.
- Committed the reviewed implementation as [`25826d07f3b55342a95e1464e4f0c1ac4c22ef8d`](https://github.com/ayushrameja/burnhop-rust/commit/25826d07f3b55342a95e1464e4f0c1ac4c22ef8d), **Add native movement, combat and two-player multiplayer**. It includes all four Rust crates and their tests, Cargo.lock, pinned toolchain, both CI platform jobs and project documentation. Earlier milestones had been left uncommitted, so this is one coherent implementation commit after the original planning commit.
- Reviewed all 38 candidate repository files by inventory, content/type and credential-pattern scan, plus source, tests, manifests, lockfile provenance, workflow and documentation review. No credential-pattern matches were found. All locked registry sources are crates.io. The 36 staged changes contain source/configuration/documentation only; the existing AGENTS.md and .gitignore are retained.
- Source/configuration hashes confirmed all 28 implementation, test, manifest, lockfile, toolchain and workflow files were unchanged from the reviewed checkpoint baseline. No dependencies were added or changed. No tests, platform jobs or checks were removed or weakened.
- `target/` is ignored, including its app wrappers, logs and local test evidence; none was staged. No generated executables, secrets, temporary packaging, deployment or services were added. The browser project was not modified, no agents were spawned, and no force push or history rewrite occurred.

## Approval and evidence boundaries

The user reports personally approving Mac movement and combat through playtesting and participating in a successful two-player multiplayer playtest, approving the reviewed multiplayer milestone. Multiplayer approval is **user-reported evidence**. Network conditions, platforms, machines, latency/loss and individual physical-input/focus/reconnect checks were not supplied and are not inferred.

The dated addendum in [04-multiplayer.md](04-multiplayer.md) records that approval. Earlier handoffs retain their historical results and pending statuses; their original statements about uncommitted work or unrun CI describe their own milestone dates. This checkpoint supersedes the general review-pending status without retroactively claiming new checks in those earlier milestones.

No new rendered or human hardware playtest was performed in this checkpoint. Earlier Mac native Metal/rendered/device evidence remains in handoffs 01–04. Automated localhost UDP tests and injected application-message impairment are separate from user playtesting and actual internet measurements. CI compiles and runs headless tests; it does not open the game or validate Windows hardware/GPU behavior.

## Local verification

Native host: Apple Silicon `aarch64-apple-darwin`; `rustc 1.98.1 (48a229cea 2026-09-01)`, LLVM 22.1.8. All checks below passed before the implementation commit. After the test-only portability fix in `de704556391dae3f58ffa83aba17160691535342`, formatting, strict workspace Clippy, all 80 tests, the host build and the explicit Mac target build passed again; runtime code and dependencies did not change.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed, no warnings |
| `cargo test --workspace --locked` | **80 passed**, zero failures or ignored tests: 44 core (17 movement, 22 combat, 4 multiplayer, 1 practice-equivalence regression), 16 native adapter/route, 14 protocol, 6 server/transport; doc-test harnesses also passed |
| `cargo build --workspace --locked` | Passed; native client and server are Mach-O arm64 executables |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Passed; explicit target build and link |
| `cargo check -p burnhop-gameplay-core --locked` | Passed independently |
| `cargo check -p burnhop-server --locked` | Passed independently |
| `cargo tree -p burnhop-gameplay-core --locked --target all --edges all` | Only gameplay-core itself: zero dependencies |
| `cargo tree -p burnhop-server --locked --target all` | No Bevy, windowing or GPU packages; only shared core/protocol and transport/crypto dependencies |
| `git diff --cached --check` and staged artifact/hash review | Passed before implementation commit |

The practice trace retains its original command stream and Mac golden. The final test also compares every state/event field at every tick against frozen approved source on each platform. An isolated negative control injected a 0.25-fuel difference at tick 100; the new comparison rejected it at that exact tick. No runtime file was modified for this control. This is regression evidence, not a general cross-platform determinism guarantee.

## GitHub Actions

Implementation run: [Native checks run 34615366243](https://github.com/ayushrameja/burnhop-rust/actions/runs/34615366243), triggered by the normal push of the exact implementation SHA above.

| Target | Runner | Result |
| --- | --- | --- |
| [macOS ARM64](https://github.com/ayushrameja/burnhop-rust/actions/runs/34615366243/job/103315826793) | `macos-14` | **Passed** all checks |
| [Windows x64 MSVC](https://github.com/ayushrameja/burnhop-rust/actions/runs/34615366243/job/103315826429) | `windows-2022` | **Failed** the Mac-only practice golden assumption; earlier client/movement/combat tests passed, later protocol/transport and executable steps did not run |

### CI portability fix

The first Windows job compiled its test executables successfully, then failed `approved_practice_matches_pre_refactor_12000_tick_trace`: Windows hash `16790751179273612442`, recorded Mac hash `11325689209779929004`. The original test hashed full-precision Debug output from a Mac trace even though the handoff explicitly did not assert cross-platform determinism. Both the original and current code use [`f64::hypot`](https://doc.rust-lang.org/std/primitive.f64.html#method.hypot), whose precision can vary across platforms; this is a reason not to impose one target's raw trace on another, not proof that this one function caused every differing bit.

Fix commit: [`de704556391dae3f58ffa83aba17160691535342`](https://github.com/ayushrameja/burnhop-rust/commit/de704556391dae3f58ffa83aba17160691535342), **Compare approved practice on each native platform**.

Recovered the preserved pre-multiplayer snapshot used during milestone 4. All three source hashes match the earlier baseline ledger, and the local run reproduces the original Mac golden. Added those exact files as frozen, test-only fixtures with provenance hashes. The moved regression now advances the independent original and current implementations with identical commands and checks all Debug state/event fields at each of 12,000 ticks, without rounding, tolerances or a Windows expected hash copied from failed output. The original Mac golden is retained on Apple Silicon macOS as an additional check. The unconditional per-tick comparison runs on every platform. CI also runs this lightweight check before renderer compilation so a mismatch fails early; the complete workspace suite still runs afterward.

Runtime source, movement/weapon tuning, Cargo manifests, Cargo.lock and the toolchain remain unchanged. Only the practice test, frozen test fixtures and the added early CI check changed. The test count remains 80; no test, platform job or existing check was removed, skipped or weakened.

Fix run: [Native checks run 34618663757](https://github.com/ayushrameja/burnhop-rust/actions/runs/34618663757), exact SHA `de704556391dae3f58ffa83aba17160691535342`.

| Target | Result |
| --- | --- |
| [macOS ARM64](https://github.com/ayushrameja/burnhop-rust/actions/runs/34618663757/job/103326824772) | **Passed**: early regression, formatting, strict Clippy, all 80 workspace tests, independent server check and locked executable builds |
| [Windows x64 MSVC](https://github.com/ayushrameja/burnhop-rust/actions/runs/34618663757/job/103326824506) | **Passed**: early regression, formatting, strict Clippy, all 80 workspace tests, independent server check and locked executable builds |

Both completed logs confirm **80 passed, zero failed and zero ignored** in the full workspace suite on each platform, plus the separate early regression pass. The frozen source independently reproduced the Windows hash `16790751179273612442` and the Mac hash `11325689209779929004`; the current implementation matched its reference at every tick on both systems. macOS finished at 16:07:38 UTC and Windows at 16:21:05 UTC on 2026-09-11. No window or GPU playtest is configured.

GitHub emitted a non-blocking annotation that the existing `actions/checkout@v4` is being run under Node 24 because Node 20 is deprecated. Checkout and all checks passed; the action was not changed to address an unrelated maintenance warning. No account, runner or permission blocker remains.

These recorded results apply to the exact linked implementation/fix commits. The documentation commit containing this handoff triggers another full CI run. Its final sign-off must independently verify that run's `headSha` and both job conclusions; the final checkpoint response reports that commit and run link. Earlier green runs are not evidence for a later commit.

## Remaining validation

- Windows hardware: launch/rendering, GPU behavior, movement/combat/multiplayer feel and performance on a real Windows machine.
- An actual documented Mac-to-Windows match. Passing the same CI suite on both targets does not establish a live cross-platform session or general deterministic simulation.
- Physical sustained mouse/Shift holds, cursor exit and focus transitions where individual results have not been reported.
- Real internet reachability and latency/loss/jitter, plus measured frame/server timing and player hosting from actual regional connections. No NAT/relay, public hosting, Go service or eight-player expansion was added.

Next: verify the current checkout's exact-SHA CI status before resuming, then collect concrete Windows hardware and cross-platform/internet playtest evidence before expanding scope. All commit/push/CI work for this checkpoint is authorized; no user action is required to resolve an external blocker.
