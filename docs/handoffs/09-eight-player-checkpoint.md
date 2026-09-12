# 09 — Reviewed eight-player checkpoint

2026-09-12. Save the manager-reviewed eight-player milestone to the existing [Burnhop Rust repository](https://github.com/ayushrameja/burnhop-rust), then verify the exact final pushed commit on macOS ARM64 and Windows x64. The brief reports no blocking manager-review findings. **New human playtest approval remains pending.**

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

At preparation, the milestone is ready for the authorized normal commit/push. Exact-SHA CI is pending until pushed. Both existing jobs remain: `macos-14` / `aarch64-apple-darwin` and `windows-2022` / `x86_64-pc-windows-msvc`. They run formatting, the early frozen practice regression, strict all-target Clippy, workspace tests, an independent headless-server check and locked executable builds. Long soak remains opt-in.

Record implementation CI results here after completion. The documentation commit recording those results also requires its own exact-SHA verification in the final checkpoint response; do not reuse a parent commit's green result or repeatedly commit only to update the final CI link.

## Outstanding validation

- New human two-to-eight-player approval, spawn fairness and delayed moving-target aiming remain pending. Manager review and scripted checks do not establish human feel.
- Windows hardware/GPU launch, rendering, controls and performance; an actual Mac-to-Windows match.
- Real internet reachability, RTT/loss/jitter and reliability; controlled loopback impairment is separate evidence.
- Physical held mouse/Shift/Tab, cursor exit, focus transitions, dead/respawn holds and fresh-input recovery. Recheck focused movement/offline practice as part of the human checklist in milestone 08.
- No hit rewind, public authentication, automatic reconnect, NAT/relay or hosting was introduced; no fairness, frame-rate or general capacity guarantee is established.

Next: finish exact-final-commit CI and remote synchronization, then collect the remaining human/hardware/network evidence before changing approved tuning.
