# 09 — Reviewed eight-player checkpoint

2026-09-12. The manager-reviewed eight-player milestone is saved in the existing [Burnhop Rust repository](https://github.com/ayushrameja/burnhop-rust). Its implementation commit passed macOS ARM64 and Windows x64 CI; the final documentation commit requires its own exact-SHA verification. The brief reports no blocking manager-review findings. **New human playtest approval remains pending.**

## Scope and preservation

Started on `main` at `0a7c7e3fe0692f05a5a9cbaef8da75bd91174957`, with existing uncommitted milestone work. A fresh fetch found `origin/main` at the same SHA, zero ahead/behind. Verified fetch/push remote: `git@github.com:ayushrameja/burnhop-rust.git`. Normal commits and pushes are explicitly authorized; never force-push or discard newer work.

The checkpoint includes eight stable player slots/generations, separated online spawns, simultaneous combat and single-credit scoring, bounded server ownership/lifecycle, protocol 2 lossless snapshots and measured-RTT scheduling, shared pilot presentation and held Tab scores, regressions and opt-in reliability/rendered-scene harness source. It includes milestone plan/handoff, context/roadmap/visual notes, eleven useful indexed native PNGs and bounded measurement logs/JSON, including the original 605-second soak.

No runtime fix was needed during saving. The only concrete correction was evidence provenance: `01-eight-pilots.png` was entirely black. Its original bytes were preserved at `/tmp/burnhop-checkpoint09-excluded/01-eight-pilots.png` and excluded from Git. Image 02 independently shows all eight pilots; desktop/compact scoreboard images were inspected again. The screenshot index and milestone 08 addendum state the correction. No evidence image was fabricated or modified.

Candidate artifact/credential-pattern review found no secrets, executables, symlinks or unexpected binaries. `target/` remains ignored; no build output, raw captures or temporary files are included. Manifests, lockfile, toolchain and frozen approved-practice fixtures are unchanged. Approved movement, combat tuning, visuals and protocol intent are preserved. No features, dependencies, infrastructure, hosting, deployment or agents were added. The sibling browser project was left untouched.

## Fresh local validation

Host: `aarch64-apple-darwin`, unchanged pinned Rust 1.98.1 / Bevy 0.19.1.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 102 passed, zero failed/ignored: 30 client, 47 core, 18 protocol, seven UDP/server tests; doc-test harnesses passed |
| `cargo build --workspace --locked` | Passed |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Passed |
| `cargo check -p burnhop-gameplay-core --locked` | Passed |
| `cargo check -p burnhop-server --locked` | Passed |
| `cargo tree -p burnhop-gameplay-core --locked --target all --edges all` | Core only, zero dependencies |
| `cargo tree -p burnhop-server --locked --target all --edges all` | No Bevy, window or GPU dependencies |
| Diff whitespace and candidate artifacts | Passed |

The original 17 movement tests and frozen 12,000-tick practice regression pass unchanged. No test was weakened, skipped or removed. The long soak and native routes were not repeated: this checkpoint made no runtime change or observed relevant failure. [Milestone 08](08-eight-player-reliability.md) retains their measured scope and limitations.

## Save and GitHub Actions

Implementation commit: [`437b6b67db0e07e9f237b54d90e90817d5c69748`](https://github.com/ayushrameja/burnhop-rust/commit/437b6b67db0e07e9f237b54d90e90817d5c69748), **Save reviewed eight-player multiplayer milestone**, normally pushed to `origin/main`. It contains 64 reviewed files. Immediately after push, the working tree was clean and `HEAD` matched `origin/main` with zero ahead/behind. No force push, merge, runtime fix or CI fix was needed.

[Native checks 34699835294](https://github.com/ayushrameja/burnhop-rust/actions/runs/34699835294), push event, exact `headSha` **`437b6b67db0e07e9f237b54d90e90817d5c69748`**, completed with conclusion **success**.

| Target | Runner | Result |
| --- | --- | --- |
| [macOS ARM64](https://github.com/ayushrameja/burnhop-rust/actions/runs/34699835294/job/103569513330) | `macos-14` / `aarch64-apple-darwin` | Passed, completed 2026-09-12 14:50:28 UTC |
| [Windows x64](https://github.com/ayushrameja/burnhop-rust/actions/runs/34699835294/job/103569513149) | `windows-2022` / `x86_64-pc-windows-msvc` | Passed, completed 2026-09-12 15:04:52 UTC |

Both completed job logs confirm **102 workspace tests passed, zero failed and zero ignored**, plus the separately run early frozen practice regression. Every job step passed: formatting, strict all-target Clippy, workspace tests, independent headless-server check and locked client/server executable builds. Windows spent most of its longer test stage compiling dependencies; the live log confirmed ongoing compilation before the suite passed. Long soak stays opt-in. No external blocker was observed.

The documentation commit recording these results also triggers the full workflow and requires independent exact-SHA verification in the final checkpoint response. These links validate the implementation commit only. Do not reuse a parent commit's green result or repeatedly commit merely to update the final CI link. Read the final checkpoint response or query GitHub Actions for the current `git rev-parse HEAD` to verify the final documentation SHA.

## Outstanding validation

- New human two-to-eight-player approval, spawn fairness and delayed moving-target aiming remain pending. Manager review and scripted checks do not establish human feel.
- Windows hardware/GPU launch, rendering, controls and performance; an actual Mac-to-Windows match.
- Real internet reachability, RTT/loss/jitter and reliability; controlled loopback impairment is separate evidence.
- Physical held mouse/Shift/Tab, cursor exit, focus transitions, dead/respawn holds and fresh-input recovery. Recheck focused movement/offline practice as part of the human checklist in milestone 08.
- No hit rewind, public authentication, automatic reconnect, NAT/relay or hosting was introduced; no fairness, frame-rate or general capacity guarantee is established.

Next: finish exact-final-commit CI and remote synchronization, then collect the remaining human/hardware/network evidence before changing approved tuning.
