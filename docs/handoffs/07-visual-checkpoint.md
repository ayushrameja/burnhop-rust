# 07 — Approved visual milestone checkpoint

2026-09-12. The user-approved gameplay visuals and reviewed compact-HUD correction are saved in the existing [Burnhop Rust repository](https://github.com/ayushrameja/burnhop-rust). Gameplay, protocol, tuning and pinned dependencies remain unchanged.

## Saved milestone

Implementation commit: [`bf34b95bf0f9953b85635b8ec48a645ccfba45a9`](https://github.com/ayushrameja/burnhop-rust/commit/bf34b95bf0f9953b85635b8ec48a645ccfba45a9), **Save approved gameplay visuals and compact HUD**, normally pushed to `main`.

The 37 committed files include the shared illustrated pilot/animation rig, weapon artwork, arena presentation, bounded confirmed effects, compact-HUD fix and its regressions, opt-in frame/capture review helpers, visual direction and continuity documentation, and 22 intentional indexed PNG review screenshots. The original obstructed minimum-window image remains as historical evidence. No raw captures, logs, executables, temporary app wrappers, secrets or build output were committed. The review helpers are source code; generated captures are excluded.

Verified fetch and push origin: `git@github.com:ayushrameja/burnhop-rust.git`. Fetched before saving: `main` and `origin/main` both pointed to `c00adcf48b529343031a201dafeca8c67cb52b09`, with zero divergence. GitHub reported default branch `main`, push permission, no active branch rules and an unprotected branch. Used the existing normal direct-push workflow; no history rewrite or force push.

All 26 protected core/protocol/server, manifest, lockfile and toolchain files remain byte-identical to that baseline. The unchanged CI workflow is a 27th verified protected file. Client input capture, practice simulation, aim projection and aim capture functions are also unchanged. The online client only exposes received remote presentation state and adds a short jet pulse to the existing opt-in smoke route; live gameplay/network rules are preserved.

No agents, new gameplay, tuning, maps, dependencies, eight-player work, Go services, hosting, purchases, release or deployment were used. The sibling browser project was left untouched. Existing local work was preserved.

## Approval and review evidence

- The user reports completing the gameplay visual playtest and approving the result. Platform, offline/online mode and individual checklist results were not specified; no additional outcomes are inferred.
- The current checkpoint brief records that the compact-HUD correction passed final review. [06-gameplay-visuals.md](06-gameplay-visuals.md) records that result in a dated addendum, preserving earlier review-pending statements as historical evidence.
- This checkpoint reviewed candidate source, documentation and artifact scope; a credential-pattern scan found no matches, and the candidates contained no executable files or symlinks. Selected current minimum-platform and connection-error screenshots were inspected again.
- Earlier actual Mac Metal renderer, device and scripted route evidence remains in milestone 06. Screenshot review here is not a new rendered session or human playtest. Automated tests and CI are separate from the user's approval.

## Fresh local checks

Host target: Apple Silicon `aarch64-apple-darwin`, using the unchanged pinned Rust 1.98.1 toolchain and Bevy 0.19.1 dependencies.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | **90 passed**, zero failed/ignored: 26 client, 44 core, 14 protocol, 6 server/transport; doc-test harnesses passed |
| `cargo build --workspace --locked` | Passed |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Passed |
| `cargo check -p burnhop-gameplay-core --locked` | Passed |
| `cargo check -p burnhop-server --locked` | Passed |
| `cargo tree -p burnhop-gameplay-core --locked --target all --edges all` | Core only: zero dependencies |
| `cargo tree -p burnhop-server --locked --target all --edges all` | No Bevy, windowing or GPU dependencies |
| Candidate/staged whitespace and artifact review | Passed; only reviewed milestone files staged, `target/` ignored |

The unchanged approved-practice regression compares each tick with frozen approved source for 12,000 ticks and retains the Mac golden. No checks were weakened, skipped or removed.

## GitHub Actions evidence

Implementation run: [Native checks 34681615290](https://github.com/ayushrameja/burnhop-rust/actions/runs/34681615290), push event, exact `headSha` **`bf34b95bf0f9953b85635b8ec48a645ccfba45a9`**.

| Target | Runner | Result |
| --- | --- | --- |
| [macOS ARM64](https://github.com/ayushrameja/burnhop-rust/actions/runs/34681615290/job/103521154084) | `macos-14` | **Passed**, completed 2026-09-12 08:03:57 UTC |
| [Windows x64 MSVC](https://github.com/ayushrameja/burnhop-rust/actions/runs/34681615290/job/103521154137) | `windows-2022` | **Passed**, completed 2026-09-12 08:13:00 UTC |

Both completed job logs confirm **90 workspace tests passed, zero failed and zero ignored**, plus the separately run early practice regression. Both jobs passed formatting, strict workspace Clippy, independent headless-server checking and locked native executable builds. Run status is `completed`, conclusion `success`; no implementation or CI fix was needed.

GitHub emitted the existing non-blocking annotation about `actions/checkout@v4` using Node 24 instead of deprecated Node 20. Checkout and all checks passed; the workflow was not changed. No external blocker remains.

This handoff references the validated implementation run. The documentation commit containing it triggers a separate full workflow; the final checkpoint response must independently verify that run's exact `headSha`, both target conclusions and run link. Do not treat the implementation run as proof for a later commit, or repeatedly commit merely to update a CI link.

## Remaining checks and limitations

- **Windows hardware/GPU:** launch and render the game on a real Windows machine, then assess movement, combat, visual readability, animation and performance. A Windows CI build is not Windows GPU or human playtest validation.
- **Cross-platform multiplayer:** document an actual Mac-to-Windows match. Passing both CI jobs does not establish a live cross-platform session or universal deterministic simulation.
- **Physical input:** document unreported sustained mouse/Shift holds, cursor exit, held-input focus transitions and fresh-input recovery. Scripted routes and input tests remain separate evidence.
- **Internet:** test actual reachability, latency, loss and jitter, plus frame/server timing under play. NAT/relay setup, regional player hosting and public services remain deferred.
- No new performance benchmark was run. Previous frame-interval observations and compact-camera limitations remain in milestone 06. Eight-player expansion, additional maps, menus, audio and hosting are outside this checkpoint.

Next: collect concrete Windows hardware, Mac-to-Windows and real internet evidence while preserving the approved visual/gameplay baseline.
