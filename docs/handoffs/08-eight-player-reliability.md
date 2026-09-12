# 08 — Eight-player multiplayer and reliability

2026-09-12. Expanded the approved native checkpoint `0a7c7e3fe0692f05a5a9cbaef8da75bd91174957` from two to eight direct-connect players. Native changes are ready for review, **not committed or pushed**. The browser project is untouched. No agent delegation, hosting, purchases, services, maps, weapons, gameplay tuning or dependency changes.

Eight synthetic clients sustained authoritative movement/combat, scoring, respawns and replacement joins through controlled packet loss and a 605-second soak. One real Mac native window rendered all eight actors; two actual native windows and offline practice also completed their scripted combat routes. This is controlled local evidence, not eight-human, real internet or Windows hardware validation.

## Baseline and plan

The native checkout was clean and matched the requested checkpoint. Read the repository instructions, current context/roadmap/visual direction and prior handoffs. The [plan and intended pass criteria](08-eight-player-plan.md) were recorded before major refactoring. Work was staged in a temporary source copy and conflict-checked against original file hashes before applying to the native checkout, preserving any newer work.

Baseline formatting/strict Clippy/build passed. The first baseline test invocation passed 88/90: accelerated loopback tests raced real UDP delivery in compatibility and impairment checks. All six transport tests passed on serial repeat. The updated test driver yields one real millisecond between accelerated iterations and caps staged snapshots at the same 32-entry client bound. A duel test now approaches distant respawn positions using ordinary movement; the new spawn policy deliberately no longer guarantees the old short duel distance.

Final test count is **102** (30 client, 47 gameplay-core, 18 protocol, seven actual UDP integration tests), preserving the original behavioral coverage including the frozen 12,000-tick approved-practice comparison and all 17 movement tests. Formatting, strict Clippy, locked host and explicit Mac ARM64 builds pass. Both existing macOS ARM64 and Windows x64 CI jobs remain; short eight-player tests are included through the workspace suite, and the long soak stays opt-in. Current-change GitHub Actions was not run because commit/push was prohibited. Prior checkpoint CI is historical evidence only.

## Shared simulation, spawns and scoring

`MAX_PLAYERS = 8`; checked stable IDs P1–P8 index fixed optional actor slots. Each actor owns full movement/combat state, neutral-input gate, scores and a fresh join generation. `step_match` accepts eight commands and steps each present actor once. Core remains dependency-free. `step_practice` maps the original two actors into the same implementation, then maps the result back; its stationary bot policy, exact starting/respawn positions and approved frozen output remain unchanged.

For each tick: process lifecycle, move all actors, collect one common living-body snapshot, decide every shot, then apply damage. The nearest ray intersection wins; terrain wins an equal-distance body tie, lower actor ID wins equal-distance body ties. Damage applies in shooter-ID order and clamps to remaining HP. The first lethal shot gets one kill, the victim gets one death, and later same-tick hits cannot score again. Shots already decided still return fire when their shooter dies that tick. Simultaneous respawns resolve in stable actor order and see earlier respawns when selecting separation.

Eight candidates use existing floor/platform tops. Coordinates are top-left of the unchanged 36 ×68 body:

| Candidate order | x | y |
| --- | ---: | ---: |
| Original player | 390 | 1152 |
| Original bot | 910 | 1152 |
| Left floor | 90 | 1152 |
| Right floor | 2240 | 1152 |
| Platform 1 | 210 | 892 |
| Platform 2 | 700 | 687 |
| Platform 3 | 1300 | 887 |
| Platform 4 | 1900 | 602 |

Each candidate must be finite, wholly inside the arena, supported and free of solid overlap. Online joins/respawns prefer no overlap with living actors, then maximize distance to the nearest living actor; candidate order breaks ties. If all valid candidates are contested, choose the best distance anyway: no immunity, random retries or geometry change. A generic no-valid-candidate fallback uses the arena spawn; the shipped arena has eight validated candidates. This is separation, not proven spawn fairness or line-of-sight safety. The second player can now start at the far side rather than the old bot spawn, and respawns can use platforms. Offline spawns remain exact.

## Ownership, lifecycle and overload

The server has eight gameplay slots and ten transport seats: two extra bounded seats allow pending handshakes and a clear `Full` rejection for the ninth gameplay client. Under a simultaneous connection flood the transport's own full result may precede an application rejection. Connection-to-slot ownership, 32-tick input window, token/message limits, three-second handshake/input-silence timeout and bounded half-second rejection delivery grace remain. Compatibility is checked before a slot is assigned. A spoofed ID cannot control another actor.

Departure removes the actor and queue, clears retained shots both **from and targeting** that slot, and accumulates bounded diagnostic counters. Reuse creates a fresh generation, full health/ammunition/fuel, zero scores and neutral gate. The old process cannot pass input to the new connection. Client interpolation and shot deduplication track actual actor ID plus generation; life/death changes clear effects and pose/history. Death counters also detect a death-and-respawn cycle skipped between snapshots, even if the next visible state is alive at the same position.

The authoritative loop remains 60 Hz. Each clock update admits at most 250 ms and executes at most five fixed ticks, dropping excess whole wall-clock ticks. It slows simulation under overload; it never enlarges `DT` or runs unbounded catchup. Missing command slots immediately use neutral/release input, so loss cannot indefinitely repeat held movement/jet/fire. Rejoining requires a new client process; no automatic reconnect or online F5 reset was added.

## Protocol and packetization

| Boundary | Version or limit |
| --- | --- |
| Application protocol | `2` |
| Gameplay compatibility | `0x4255_524e_0008_0001` |
| Stable Netcode envelope ID | `0x4255_524e_484f_5001` |
| Pinned transport | Renet 2.0.0 / renet_netcode 2.0.0 |
| Complete three-command input bundle | **136 bytes maximum** |
| Complete eight-actor/eight-shot snapshot | **1,185 bytes maximum** |
| Decode envelope | 1,200 bytes |
| Snapshot rate | 30 Hz, replaceable unreliable full state |
| Transport flush cap | 60 Hz on client and server |
| Channel allocation | 16 KiB per configured direction/channel, 4,096-byte update send budget |

The stable transport ID lets older clients reach a clear application compatibility rejection. All participants need the new build. Snapshots retain every authoritative f64 value, weapon/life timer, grace/buffer/fuel state, neutral gate, score and generation needed for reconciliation. Fixed body dimensions are implied by gameplay compatibility; three booleans share a byte, and bounded ammunition/equip/weapon timers use bytes. There is no position quantization, delta baseline, missing-state extrapolation or invented reliable protocol.

Review of the pinned Renet source found a 1,200-byte slice threshold, three-second retention for incomplete unreliable slices and the configured 16 KiB channel cap. The first straightforward eight-player schema was 1,385 bytes and required two slices; incomplete snapshots under loss put pressure on reassembly. Rather than increase channel buffers, lossless packing reduced the exact worst case to 1,185 bytes, keeping each snapshot unsliced. The final soak observed UDP payloads up to 1,214 bytes including transport overhead; application size is not UDP wire size. This fits ordinary Ethernet-sized packets, but path MTU/internet behavior was not tested.

Handshake/control/release stay reliable ordered; input/state stay unreliable with up to two prior input commands repeated. Full snapshots replace prior ones, so a lost packet does not poison a delta chain. A fast polling loop previously flushed redundant ACK traffic repeatedly; flushing now caps at 60 Hz while receive polling remains responsive. Maximum theoretical snapshot enqueue rate is 284,400 bytes/s across eight seats; measured soak average was 281.59 kB/s because some states/shots are shorter. Observed UDP down averaged 301.71 kB/s total (37.71 per-seat equivalent), up 76.77 kB/s total (9.60 per-seat equivalent), excluding UDP/IP/link headers.

Decoding uses fixed arrays without peer-provided allocation sizes. It rejects oversized/truncated/trailing data, invalid tags/IDs/flags, nonfinite or excessive floats, actor-slot mismatches, invalid life/health/ammo combinations and inconsistent confirmed-shot shooter/tick/target data. Worst-case round-trip size and malformed last-slot cases are tested. New tuning that exceeds a byte must update this schema and compatibility; byte writes fail explicitly rather than silently truncate.

## Why input scheduling changed

A fixed-six-tick experiment stayed connected at 50 ms RTT, but all eight clients timed out at about four seconds under 150 ms RTT/3% packet loss. It was no longer keeping accepted future commands ahead of the server. The preserved development logs are in [measurements](../measurements/08-eight-player/README.md). Adaptive lead alone with the still-fragmented schema then showed 1,979 missing of 12,472 steady slots and a maximum 442.137 px correction. Final packing, bounded RTT scheduling and flush cadence were tested together; these sequential experiments do not isolate each change's independent performance effect.

The client estimates lead from Renet's measured RTT, initially falling back to Hello→Welcome elapsed RTT. The newest snapshot acknowledgement anchors its command clock; snapshot age advances from local monotonic elapsed time, capped at one second. Target lead is `clamp(ceil(RTT_seconds ×60) +4, 6, 24)` ticks. The received snapshot is already one direction old, so reserving a full RTT plus four ticks allows the command to travel back and tolerates send cadence/jitter. Lead grows promptly and shrinks at most one tick after ten stable lower-RTT seconds.

If command sequence falls more than two ticks behind the target, it skips ahead; skipped slots are neutral and do not perform additional simulation. A snapshot-age ceiling and server-window check prevent bursts from getting arbitrarily ahead. Acknowledgements explicitly retire applied **and missing** slots; `last_applied` distinguishes actual arrival. History is at most 128 commands, with the server-window guard stopping prediction much earlier in these runs; snapshots are capped at 32. The server always consumes one command per actor/tick regardless of input burst or resynchronization.

Only movement is predicted. Combat, health and scores remain authoritative. Remote interpolation still displays six ticks behind the received snapshot, without remote extrapolation or hit rewind. Higher RTT therefore increases delayed-target aiming error, and a dropped one-shot command can still be lost if all redundant copies miss. Synthetic correctness/recovery does not establish human feel.

## Network matrix and soak

Pass criteria were declared in the plan: eight connected steady clients, no unexpected rejection, finite in-bounds state, shared fixed-step simulation, bounded resources, combat/respawns, and fresh applied acknowledgement within two seconds after a one-second client stall.

The harness uses real encrypted UDP via eight local two-socket proxies. Profile 50 delays each direction by 25 ±2.5 ms with no loss; 100 uses 50 ±7.5 ms with independent 1% datagram loss; 150 uses 75 ±12.5 ms with 3% loss. Independent uniform delay naturally reorders packets, including ACK/control/handshake traffic. Seeds are `0xB08 + client_index`; scheduling still changes exact outcomes. Earlier application-message impairment tests are separate. The stall profile uses 100 and suspends all client polling/input for seconds 8–9 while server/proxies continue; it is not total network-blackout emulation.

Each final matrix run lasted 30 seconds. All five passed, with zero dropped server ticks:

| Profile | Applied / missing slots after 4 s warmup | Deaths / respawns | Correction count / max pixels |
| --- | ---: | ---: | ---: |
| Baseline | 12,472 / 0 | 48 / 43 | 0 / 0 |
| 50 ms | 12,472 / 0 | 48 / 43 | 0 / 0 |
| 100 ms /1% | 12,471 / 1 | 44 / 40 | 0 / 0 |
| 150 ms /3% | 12,469 / 3 | 42 / 36 | 1 / 0.089 |
| One-second client stall | 11,888 / 584 | 42 / 39 | 11 / 26.633 |

The stall recovered within its two-second criterion. Missing slots during the pause are expected neutral time, not simulated repeated input. Full values, bandwidth, counter definitions and raw JSON are in the [measurement index](../measurements/08-eight-player/README.md).

**605.002-second soak**, Apple M1 Pro /16 GiB /macOS 26.6.2 (25G83), aarch64, Rust 1.98.1, optimized development build (own code opt-level 1, dependencies 3), profile 100 with 1% per-direction packet loss. Desktop apps remained open; some native review overlapped later soak time. Eight synthetic clients used ordinary movement, jump/jet, aiming, sustained fire, reload/switch and respawn. One client left every 45 seconds, with 0.6-second departure gap then a fresh handshake.

- 21 total joins = initial eight plus **13 replacements**; each reused the departed slot with a new generation and zero scores.
- 36,299 simulation ticks; **13,670 shots, 916 deaths, 906 respawns**. Intentional short departure/handshake gaps aside, all eight remained active.
- 287,573 applied /159 missing slots after warmup; one stale redundant copy. Missing includes join reservations/handshakes and neutral gates.
- Zero dropped server ticks. Tick work p95 **0.037375 ms** over the last 4,096 samples; lifetime maximum **7.157750 ms**. This excludes receive/send/snapshot encoding and GPU work.
- History peak **19**, server queue peak **8**, proxy peak **18** packets. Snapshot bound 32, history hard bound 128, input window 32 and proxy hard bound 512 remained enforced.
- 141,447 reconciliation measurements, zero nonzero movement corrections under the defined metric (lifecycle/initial resets excluded).
- Headless harness RSS: **4,896→5,072 KiB**, sampled peak **5,936 KiB**. This includes the server/eight clients/proxies in one process, sampled every 30 seconds after the first five, not renderer or total system memory. No runaway growth observed; finite sampling is not proof against all leaks.

## Native rendering and regressions

[Representative native screenshots and provenance](../screenshots/08-eight-player-reliability/README.md) show eight approved pilots with cyan YOU/P1 and ochre P2–P8; shared health/weapon poses; multiple remote jet pairs; authoritative death/impact effects and respawns; desktop and **480 ×320 minimum logical** scoreboard/HUD; departure and fresh visible slot reuse. Desktop framebuffer was 2560 ×1440 at 1280 ×720 logical. The companion runs seven synthetic peers after the actual native client joins. Their process order is not guaranteed actor assignment: the first departure shown is P2; a separate visible-reuse repeat is P3.

The held scoreboard uses actual stable identities and authoritative kills/deaths. It releases on Tab release/focus loss/disconnect, retaining normal gameplay input. Persistent capture used the explicit `BURNHOP_REVIEW_TAB` inspection override; a Bevy system test separately exercises real pressed/released/focus/disconnect state. No human physical-Tab claim is inferred. Ordinary HUD edges/artwork/camera remain approved. Effects use the existing maximum **24 traces/pending shots** with five sprites each, sixteen exhaust cones and per-actor hit/animation state; no unbounded entity spawn per shot. Under saturation the oldest short-lived effect can be replaced; damage is unaffected.

Two **actual native Metal client processes** on localhost each printed `ONLINE PLAYTEST COMPLETE` after movement, jump, jet, kill, death and respawn through injected inputs. The actual offline native combat route passed all seven checkpoints, including rifle/pistol reloads, both deaths/respawns and F5 reset. Additional focused movement attempts canceled on native focus loss after one/two checkpoints; this run does not claim a new full native movement-route pass. All 17 movement tests and frozen practice pass; prior approved full native movement evidence remains in milestone 06.

A separate native frame sample used one 1280 ×720 window and seven synthetic combat peers, local input mainly idle/neutral, with no screenshots/forced scoreboard: after 180 warmup frames, 1,800 measured intervals had mean **11.600 ms**, median **8.484**, p95 **18.101**, p99 **20.154**, and 331 intervals above 16.7 ms. Includes VSync, OS scheduling and focus transitions; this is neither GPU timing nor a stable frame-rate target. Reconciliation/history metrics are in the retained log.

Early app-control-attached capture attempts produced blank/stale output. Those were discarded; clean direct native launches produced the reviewed images. Automated routes, framebuffer inspection, previous user approval and new human playtesting are separate evidence.

## Run and review

From `/Users/ayushrameja/codebase/LHP/burnhop-native`, use separate terminals:

```sh
cargo build --workspace --locked
cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000 --diagnostics
# Repeat client command for players 1 through 8:
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
# Offline practice:
cargo run -p burnhop-client --locked -- --offline
# Optional measured soak:
cargo run -p burnhop-server --example reliability --locked -- --seconds 605 --players 8 --profile 100 --churn
```

The [README](../../README.md) has platform setup, all profiles, the one-native/seven-synthetic review companion, frame/capture flags and everyday checks. Direct connect uses numeric IP:port on an already reachable local interface. No NAT traversal, relay, matchmaking, room codes, token service or hosting deployment is included; Netcode unsecure development authentication remains appropriate only for trusted direct-connect testing.

Short human checklist:

1. Start two, then up to eight matching clients. Confirm YOU and each P-number, move/jump/jet, switch/reload/fire and inspect Tab scores at desktop and minimum size.
2. Shoot one target from multiple sides; confirm one kill/death credit, three-second respawn, fresh health/ammo and no old jet/recoil/trace after death.
3. Close an active shooter, confirm its pilot/scores disappear, then join a replacement with fresh scores; try a ninth client and inspect `Full`.
4. Release Tab and switch focus while holding mouse/Shift; return and press fresh controls. Check cursor exit, dead/respawn holds and online F5 exclusion.
5. Judge distant/platform spawns and aiming at moving targets under actual latency. Record hardware, RTT/loss and human observations before tuning anything.
6. Recheck offline practice/F5 and the movement route in a focused window. Later repeat on Windows hardware, Mac-to-Windows and a real internet path.

Remaining limits: no hit rewind/lag compensation, no public authentication/hosting, no automatic reconnect, no eight-human/internet/Windows hardware validation, no fairness or frame-rate guarantee, and no CI result for this uncommitted change. Preserve these distinctions when saving the next checkpoint.

## Checkpoint 09 addendum — 2026-09-12

The subsequent checkpoint brief reports manager review with no blocking issues and authorizes saving to GitHub with exact-SHA CI verification. Earlier uncommitted/no-push statements above describe the implementation handoff, not the current authorization. New human playtest approval has not been reported. See [09-eight-player-checkpoint.md](09-eight-player-checkpoint.md).

Checkpoint artifact review found that the supplied `01-eight-pilots.png` is entirely black (all decoded scanline bytes are zero), contrary to the original screenshot-index claim. It is excluded from Git and preserved outside the checkout. The corrected index points to image 02, which independently shows all eight pilots; images 02 and 03 were visually inspected again. Other retained PNGs have nonblank image data. This corrects evidence provenance without changing runtime code or repeating the soak.
