# 15 — Ember Relay save checkpoint

2026-09-13. This checkpoint saves the approved offline Ember Relay milestone to `main` in `git@github.com:ayushrameja/burnhop-rust.git`. The user explicitly authorized the commit and normal push, superseding the earlier task-specific save restrictions. **New remote CI is pending; the user owns monitoring.** This document does not claim either target has passed for this checkpoint.

## Saved scope and approval

- Exact approved v2 original map: 3200×1900, 35 solids, both lower sleeves/voids, S0/B0, shared polygon collision/rays, offline stance/blocked standing, resource-preserving recovery, released-input gate and 180 eligible bot-grace ticks.
- Native map selection, camera/aim/lifecycle cleanup, crouching pilot and simple original terrain presentation. Approved geometry/tuning, range behavior, fixed online body/protocol and Host/Join remain preserved.
- Meaningful collision, recovery, stance, camera/input and map-switching tests; opt-in injected native traversal fixtures.
- Original-map brief/SVG, web-to-native inventory and superseded Outpost analysis, handoffs 12–14, approval notes and context/roadmap history.
- All 20 milestone 13 screenshots and two P3 desktop/compact captures, their indexes and bounded verification/native logs. Five specifically reviewed milestone 13 `.log` files are explicitly included despite the general ignore rule, preserving referenced diagnostics without changing that rule. Temporary wrappers/captures, build output and local configuration are excluded.

The user completed the requested offline playtest and reports **“it is working great”**. [Independent review 14](14-ember-relay-review.md) found no blocking defects or collision-geometry/tuning deviations. Both P3 findings were fixed before saving: the cancellation test owns an ephemeral endpoint and verifies ten cancellation/retry cycles and socket cleanup; the west lower decorative accent follows the authored slope. Neither correction requires another human playtest. See [implementation 13](13-ember-relay-offline.md) and the [P3 evidence index](../screenshots/14-ember-relay-review/README.md).

The historical ten rejected packets remain unexplained at sender level: a pre-handshake disconnect mechanism was independently reproduced, and the former default-port test is a plausible source, but actual historical attribution was not established. The separate range-script attempt remains canceled on focus loss, not a completed native route or demonstrated gameplay defect. Original evidence bytes and qualifications are preserved.

## Validation and preservation

Before saving, local `main` and freshly fetched `origin/main` both pointed to `7e9610a3d2398e058bfee74bf7fd684b8f0d13fe`, with no ahead/behind commits. Reviewed tracked/untracked files belong to this milestone and its preceding inventory/design work; no unrelated or newer work was found to discard or reconcile.

Source and test content was checked against the prior review ledger and the exact P3 fixes. The already completed validation remains applicable; saving changes only this handoff and focused context/roadmap status. No runtime/test edit, dependency upgrade, geometry/tuning adjustment, fixture change, or workflow change was made while saving.

| Check | Evidence |
| --- | --- |
| Formatting | Fresh `cargo fmt --all -- --check` passes |
| Strict Clippy | Reused unchanged-code pass: `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Workspace tests | Reused unchanged-code pass: `cargo test --workspace --locked`; **135 passed**, zero failed/ignored (43 client, 66 core, 18 protocol, 8 server) |
| Frozen practice | Unchanged 12,000-command every-field comparison and Mac golden passed; frozen source/assertions preserved |
| Focused P3 checks | Owned-endpoint cancellation and all-35-solids SVG equality passed |
| Locked Mac builds | Both `cargo build --workspace --locked` and explicit `--target aarch64-apple-darwin` passed on unchanged code |
| Dependency boundaries | Prior verified zero-dependency core and headless server retained; manifests, lockfile, protocol/server and workflow unchanged |
| Native evidence | Prior actual desktop/compact renderer inspections retained; fresh checks validate all 22 PNGs' chunk checksums and nonblank decoded data |
| Save hygiene | Source/documentation whitespace checks pass. Full staged check reports only final blank lines in the original `routes.txt` and `workspace-tests.txt` console captures; those bytes are deliberately preserved. Full check with only `blank-at-eof` disabled passes. Candidate-path/content review excludes secrets/configuration/build/temporary output. |

No long traversal or network soak was repeated. The browser repository was not operated on. No game window was disturbed during saving. No agents, deployment, background watchers, automations or additional tasks.

## Exact-SHA CI and manual handoff

The normal push triggers the unchanged **Native checks** workflow in [build.yml](../../.github/workflows/build.yml). **CI is pending in this committed handoff.** The final task response supplies the final pushed SHA and exact run URL after a brief lookup verifying `headSha`, along with the observed queued/running/completed status and confirmation against live remote main. The checkpoint is identified by the commit containing this document; it does not embed its own hash or claim a future CI result.

Monitor both jobs for that exact final SHA:

- `aarch64-apple-darwin` — macOS ARM64.
- `x86_64-pc-windows-msvc` — Windows x64.

Both must pass before CI is cleared. They run formatting, the approved practice regression, strict Clippy, workspace tests, headless-server checking and locked executable builds. If a job fails, send the **run URL, head SHA, failing job and step, and the first relevant compiler/test error with surrounding log lines** (including assertion expected/actual values or test name where present). No repeated polling or completion wait is part of this task. If the exact run is not promptly available, use the final SHA on the [Actions page](https://github.com/ayushrameja/burnhop-rust/actions).

Windows hardware/GPU/input/resize review, second-machine LAN, Mac-to-Windows, internet/NAT/relay and eight-human feel remain separate. Neither Mac checks nor successful CI establishes those results. The user's prior offline approval remains valid; no additional repeat Mac human playtest is needed for this save-only step.
