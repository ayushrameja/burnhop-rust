# 11 — Approved menu and hosting checkpoint

2026-09-13. Save milestone 10 from previous checkpoint `b64d213b1bdcfaa272848926ae9716fdff56c042` in `/Users/ayushrameja/codebase/LHP/burnhop-native`. The user explicitly authorizes commits and normal pushes to `git@github.com:ayushrameja/burnhop-rust.git`, branch `main`; no force push. Earlier milestone-specific save restrictions are superseded. Remote CI is **pending**; this is not a fully validated checkpoint.

## Approval and preserved work

The user explicitly approved all six checks on the local Mac setup on 2026-09-13: Practice pause/resume, Host/Join, input recovery across menus/focus, Leave/Rejoin, Stop/Rehost on the same port, and usability. These are user-reported hands-on results. The manager independently passed 111 tests, formatting, strict Clippy and a locked Mac build. Original approval notes in context, roadmap and [handoff 10](10-menu-and-hosting.md), plus the previous localhost approval in checkpoint 09, are preserved.

Initial local HEAD and freshly fetched/live remote main both matched the previous checkpoint, with zero incoming or outgoing commits. Reviewed changes contain native menu/session source, owned-server lifecycle, nine additional regression tests, related HUD/input/presentation integration, the internal client-to-server path dependency, documentation and fourteen original PNGs plus their index. All fourteen PNGs passed file/CRC/data validation; representative desktop, compact host menu, stop confirmation and validation screens were visually rechecked. Earlier handoff 10 records inspection of all fourteen. Screenshot bytes are preserved.

No concrete implementation failure was found; no code fix, dependency upgrade, gameplay tuning or visual redesign was needed. Checkpoint documentation updates remove stale current review/save status and explicitly hand CI monitoring to the user. No unrelated/newer work was found. Temporary captures/wrappers/logs remain outside Git; ignored `target/` build output is excluded. Candidate text was checked for common secret patterns; none was found. The sibling `burnhop` project was left untouched.

## Fresh local validation

All checks passed on the local Apple Silicon Mac with the pinned toolchain:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --locked` | 111 passed; zero failed/ignored: 38 client, 47 core, 18 protocol, 8 server/UDP |
| `cargo build --workspace --locked` | Pass |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Pass |
| `cargo check -p burnhop-gameplay-core --locked` | Pass |
| `cargo check -p burnhop-server --locked` | Pass |
| Locked all-target core/server dependency trees and workspace metadata | Core has zero dependencies; server has no Bevy/window/GPU dependency; intended internal client-to-server link preserved |
| Diff and candidate review | Frozen core/fixtures, codec/prediction, pinned versions and workflow unchanged; whitespace checked before commit |

The 17 movement tests and frozen 12,000-tick approved practice regression pass unchanged. Native/human review evidence is retained from milestone 10; no new hardware playtest is claimed. No long-soak rerun was warranted because there was no new simulation, wire/scheduling or transport reliability change or relevant failure during this save.

## Exact-commit CI handoff

The commit containing this document is the checkpoint candidate. Its final SHA and exact Actions run URL are reported in the task handoff after the normal push; this document deliberately records CI as pending rather than adding a follow-up documentation commit requiring another run. The save procedure verifies the located run's `headSha` equals final pushed HEAD and checks local HEAD against live remote main. Run creation/status is not a passing result.

Workflow: **Native checks**. Monitor both jobs:

- `aarch64-apple-darwin` — macOS ARM64 (`macos-14`).
- `x86_64-pc-windows-msvc` — Windows x64 (`windows-2022`).

Open the exact run, select a job, then expand any red failed step to read its logs. Send back the run URL, full SHA, both job conclusions, failed job/step name, failing command/test and the first relevant error or assertion with surrounding lines (or the downloaded job log). Historical Windows success does not establish that these new changes pass. If the run cannot be located, use the final SHA on the [Actions page](https://github.com/ayushrameja/burnhop-rust/actions).

The user owns monitoring. Stop after opening/linking the run even if queued/running; no completion polling, background watcher, automation or another task.

## Remaining limitations

Windows compilation for this checkpoint remains unverified while CI is pending. Windows hardware/GPU, a second physical machine on LAN (explicitly deferred), Mac-to-Windows, real internet reachability/NAT/firewall behavior and eight-human feel/fairness remain untested. Prior latency/aiming limits and unsecure development authentication remain. No deployment, internet hosting setup or performance guarantee is part of this save. Preserve approved gameplay and visuals; a later web-to-native feature inventory is separate work.
