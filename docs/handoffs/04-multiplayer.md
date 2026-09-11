# 04 — Two-player authoritative multiplayer

Implemented 2026-09-11 in `burnhop-native`. One headless server and two actual native Mac clients completed movement, shooting, damage, death and respawn in the existing arena. Offline practice still works. Changes are left uncommitted for review; the browser repository is untouched.

## Baseline and approval

Read AGENTS, context, roadmap, reference notes, all three prior handoffs and README. The native repository already contained modified documentation and untracked foundation/movement/combat code, lockfile and CI. Preserved all of it. A temporary baseline copy enabled conflict-checked application of changes; no Git reset, commit, push, deployment, purchase or agent delegation occurred.

Before refactoring: formatting, strict workspace Clippy, all **54 tests**, and locked Mac build passed. The Agent 4 brief explicitly says the user personally playtested and approved both movement and combat feel on Mac. This is **user-reported evidence**, now recorded in the combat handoff; no extra physical mouse/Shift/focus subcase results are inferred.

## Architecture and transport choice

| Component | Responsibility |
| --- | --- |
| `gameplay-core` | Dependency-free actor state, unchanged movement and weapon tuning, ray hits, damage and respawns |
| `protocol` | Bounded fixed binary codec, compatibility, command scheduling, prediction history, interpolation, Renet client adapter |
| `server` | UDP connections and ownership, one shared match, 60 Hz headless loop and overload policy |
| `client` | Bevy device input, offline/online selection, predicted movement, authoritative combat HUD, camera and effects |

Reviewed two maintained options using official documentation and package source:

- **Renet 2.0.0 + renet_netcode 2.0.0, selected.** Its synchronous message channels and separate transport fit the existing fixed-step adapter. It does not require `bevy_renet` or a Bevy version change. [Official repository](https://github.com/lucaspoffo/renet), [Renet API](https://docs.rs/renet/2.0.0/renet/), [Netcode adapter API](https://docs.rs/renet_netcode/2.0.0/renet_netcode/).
- **Quinn**, evaluated as the alternative. QUIC offers streams and datagrams and supports Mac/Windows, but its async runtime and certificate trust setup add work for this small direct-connect milestone. It is a viable later alternative, not an implemented dependency. [Official Quinn documentation](https://quinn-rs.github.io/quinn/quinn.html).

Downloaded and inspected the exact Renet/Netcode 2.0.0 manifests and `client.rs`, `server.rs`, packet/channel and token source. The selected crates do not declare an MSRV; local Rust **1.98.1** compilation, tests and linking establish compatibility on this Mac. Bevy remains **0.19.1**. Windows has not been compiled or run locally. Cargo.lock adds the transport/crypto dependencies; the core still has **zero dependencies**, and `cargo tree -p burnhop-server` has no Bevy/window/GPU packages.

Channel 0 is Renet reliable ordered delivery with a 100 ms resend interval, used for hello, welcome, rejection and focus release. Channel 1 is unreliable/unordered, used for redundant input bundles and replaceable snapshots. A bundle contains the latest command and up to two previous commands; there is no custom reliable transport. Each channel has a 16 KiB send/receive message memory limit and a 4096-byte outgoing budget per library update. Reliable channel overflow disconnects; unreliable excess may be discarded.

The application codec has a **1024-byte maximum message**, strict tags/bools, finite bounded floats, no peer-supplied vector lengths, no heap allocation while decoding, and rejects trailing/truncated data. Source inspection confirms Renet's 1200-byte message slices and Netcode's 1400-byte maximum packet buffer. These small application messages do not require message fragmentation. Transport encryption/connection management comes from Netcode, but **Unsecure development authentication is deliberately used**: anyone with access to the endpoint can join, and no trusted identity or server authentication is provided. Do not mistake this for secure public hosting.

A future player host can run this same server executable and join it locally; official servers can run it without a renderer. Public reachability, secure token issuance, NAT traversal, relay, host migration, room codes, accounts and Go services are deferred. Only localhost connectivity was verified.

## Shared simulation and practice equivalence

`ActorId::One/Two` are neutral two-slot identities. `Actor` contains complete movement state, combat state, a neutral-input gate, kills/deaths and a join generation. `MatchState` has a monotonic tick and two optional actors. A new connection gets a fresh generation even when reusing a slot. Spawns stay at (390,1152) and (910,1152), with the existing 36 × 68 bodies and arena geometry. Both humans receive the same finite loadout; online has no bot.

`step_match` validates all active actors' ticks before mutation. It advances life timers, moves both living humans, decides both shots against the same post-movement living bodies, then applies damage. Both may kill each other in one tick. Dead/absent actors do not absorb or fire shots. Respawn restores full health, movement/fuel and weapons after 180 ticks, ignores commands on that tick and requires neutral input before fresh controls. There is no new collision between actors, spawn immunity, weapon, map or tuning change.

`step_practice` translates the old `World`/`CombatState` shape into these same actor rules and back. Only the existing stationary bot's decision cadence and unlimited pistol reserve remain practice-specific. Movement, weapon preparation, ray resolution and damage have one implementation. Existing movement/combat tests remain, with actor label renames only. An additional regression hashes **12,000 ticks of complete practice state and events against output captured from the untouched Agent 3 source**, covering movement, fire, reload/switch, reset, focus release, damage and lifecycle. This passes; it is same-machine regression evidence, not cross-platform determinism.

## Protocol and exact scheduling

Hello sends protocol version 1 plus gameplay compatibility stamp `0x4255524e00040001`. Changes to arena, tuning, state requirements or simulation ordering must bump gameplay compatibility; wire layout changes must bump protocol version. A stable transport ID permits a clear application compatibility rejection. No actor is assigned before a compatible hello. Welcome supplies assigned actor, join generation and server `start_tick`.

Commands contain an actor, sequence and input controls only. They cannot carry a position, velocity, health, ammunition, damage or a reset request accepted by the server. The included command tick is the client's local sequence and must match it. Server routing verifies the actor against the connection's assignment and overwrites the simulation tick itself.

- Local sequence starts at **1**. Its server tick is exactly **`start_tick + sequence - 1`**; welcome sets start_tick to current server tick + **6**.
- At each server tick the server takes at most one scheduled command per actor. Queue contents or message bursts never grant additional simulation steps.
- A missing command becomes an immediate **neutral release** for that tick: no held direction, jet or fire, and buffered intent is cleared. Physical gravity, braking and weapon/life timers continue. The server never holds the last command indefinitely.
- The slot is retired whether present or missing. A snapshot's `ack` covers **all retired sequences**, not a claim that every command arrived. `last_applied` identifies the latest command actually consumed. State is the world immediately before `state.tick`, after all previous ticks completed; `ack = state.tick.saturating_sub(start_tick)`.
- Duplicate/conflicting copies keep the first accepted command. Retired/stale arrivals are discarded. Out-of-order future commands are stored by sequence. Commands at or beyond current server tick + **32**, invalid local ticks, reset flags, invalid jet edges, or non-finite/over-limit aim are rejected. Aim is limited to ±100,000 world pixels per coordinate.
- Each connection has at most **32 pending commands**, 90 new commands/second with burst 12, and 120 messages/second with burst 24. Wall time refills these budgets, not catch-up simulation ticks. Each poll processes at most 16 messages per channel before rejecting excess. Normal input is one three-command bundle per local 60 Hz tick.
- Focus/death release on channel 0 retires queued commands through the supplied sequence, rejects late copies and sets the neutral gate. It is a bounded monotonic barrier, not a world reset. Unfocused clients keep sending neutral commands. A client sending no accepted input for **3 seconds** loses its actor, even if transport keepalives continue. A client with no snapshots for **5 seconds** displays disconnected. Netcode additionally has its own 15-second unsecure-token inactivity timeout.
- Two gameplay slots are allowed. The transport accepts at most four connections so two short pending/rejection exchanges can receive useful errors. Unhandshaken peers expire after 3 seconds; rejected peers retain only a 0.5-second reply grace before disconnect. Departure removes the actor and its pending queue; a new join uses a fresh generation. Seamless reconnect and migration are absent.

The executable polls I/O at roughly 1 ms intervals and advances simulation at **60 Hz**. It admits up to 250 ms into its accumulator, executes at most **five** catch-up ticks per iteration, drops excess whole ticks and preserves only the fractional remainder. It logs dropped ticks. DT never increases, and the game deliberately slows under overload. Transport timeouts/rate budgets use the actual elapsed wall time even when simulation backlog is dropped. Snapshots are sent at **30 Hz**. Renet's socket receive loop drains available datagrams; application bounds are not a guarantee against hostile raw UDP flooding. Flood hardening and real capacity/performance measurements remain future work.

## Prediction, reconciliation and presentation

The client immediately predicts only the local actor's **movement**, using the shared movement step, full fuel/velocity/support/grace/buffer/latch state and the authoritative life/neutral gate. Health, weapon selection/ammunition/timers, shot decisions, damage, kills, deaths and respawns are copied from snapshots. The pointer reticle is immediate; weapon/tracer/impact feedback waits for authoritative confirmation. There is no speculative damage or ammo deduction to roll back.

New snapshots must increase server tick and have valid ack mapping and join generation. Reconciliation restores the complete local actor, drops history through `ack` and replays remaining movement commands; omitted slots use neutral release. After a client stall it can skip ahead to reserve six future slots again instead of simulating extra time. Input history has an explicit **128-entry cap**, with the 32-tick horizon imposing a tighter practical limit. Prediction stops when it reaches that horizon without new snapshots. Death/respawn clears history and device input, sends a release barrier, and snaps local presentation/camera to the authoritative spawn. Corrections are immediate; smoothing/adaptive latency estimation are not added.

Remote snapshots retain at most **32 samples**. Display targets approximately six ticks (100 ms) behind estimated server time and linearly interpolates positions. It freezes at the last sample during a gap; it never extrapolates a remote actor indefinitely. Join generation and death/life transitions prevent interpolation across respawn teleports or reused slots. Departures hide the remote body, barrel and health bar immediately when observed.

Snapshots repeat the latest confirmed shot per actor with its server tick. Each client renders a shot only once using a monotonic tick watermark; reconciliation does not replay cosmetic events. Old effects are suppressed at initial join and after long gaps. Loss may omit a cosmetic tracer if a newer shot replaces it, but damage is already present in authoritative state. Actor IDs are mapped to local-orange/remote-red only at the view boundary.

**No server-side hit rewind or lag compensation.** Shots use the authoritative bodies on their scheduled server tick, while the opponent is displayed in the past. Six ticks of fixed lead is a simple low-latency direct-connect choice, not an internet latency solution. Larger delays, jitter, lost taps, stalls and corrections need human evaluation and later scheduling work. Cross-platform floating-point determinism is not asserted.

## Verification

Host: Apple M1 Pro, Apple Silicon macOS 26.6.2, native `aarch64-apple-darwin`. Actual clients reported **Metal**. No frame-time or latency benchmark is claimed.

| Check | Evidence |
| --- | --- |
| Baseline | Formatting, strict Clippy, 54 tests and locked Mac build passed before changes |
| Final formatting / strict Clippy / tests / locked build | Passed: `cargo fmt --all -- --check`, strict workspace Clippy, **80 tests** (44 core, 16 native adapter, 14 protocol, 6 server/transport), `cargo build --workspace --locked`, and independent `cargo check -p burnhop-server --locked` |
| Dependency boundary | Core dependency tree contains only itself; server tree has no Bevy or GPU packages; both binaries are Mach-O arm64 |
| Practice equivalence | Existing tests plus the pre-refactor 12,000-tick state/event trace |
| Shared multiplayer core | Both actors' movement equivalence, online reset exclusion, simultaneous lethal shots, exact respawn/full state, missing input and absent actors |
| Protocol/client logic | Size/tag/float rejection, malformed-byte corpus, ownership, compatibility, sequence/rate/buffer bounds, missing input/ack, full-state reconciliation, stale snapshots, life history clearing, interpolation and effect deduplication |
| Actual local UDP, synthetic clients | Two-way kill/death/respawn, explicit disconnect, fresh generation join, silence cleanup, incompatible hello, spoofed actor, oversized message, full match and message flood |
| Injected conditions | Actual localhost transport with application-message scheduling: 0–3 ticks of delay/jitter, every 13th outgoing input bundle and every 11th eligible snapshot delivery dropped, resulting reordering, queues bounded to 8 in each test direction. These are injected conditions, not measured packet loss or raw IP impairment |
| Two actual native clients | `Burnhop Online One` and `Two` ran the ordinary built client against the headless server. Both logged `ONLINE PLAYTEST COMPLETE`: movement/jump, firing, one kill, one death, full respawn. Inputs were injected by the explicit route, not physical holds |
| Native device checks | D/Space/F5 were exercised; online F5 left health/ammo/counters intact. Mouse clicks consumed rifle ammunition and reduced opponent HP to 8 then 0; the native HUD showed death, followed by respawn. R reloaded, including completion while Finder held focus |
| Focus and lifecycle | Unfocused HUD stayed connected and reported cleared input; the server continued. Closing client Two hid its actor in client One. A fresh actual client joined slot Two with generation 3, full health/ammo and zero personal scores |
| Lost server | Stopped only the test server; native client showed the five-second no-snapshot disconnected state and hid its opponent. This exercised abrupt server loss, not host migration |
| Offline native regression | Actual Metal client `--offline --combat-playtest` completed all seven original combat checkpoints, including both reloads, kills, player death/respawn and F5 |
| Windows / CI / internet | **Not run**. CI is extended to the whole workspace plus a separate headless server check. No Windows hardware, public reachability, cross-platform match or determinism claim |

Ignored wrappers and logs are under `target/smoke/`: `multiplayer-server.log`, `burnhop-online-one.log`, `burnhop-online-two.log`, `burnhop-online-fresh.log`, and `burnhop-offline-review.log`. They are local inspection aids, not application packaging. The usual existing Bevy/Winit “Skipped event Destroyed” warning appeared on close, without an observed gameplay/render failure. Native UI inspection caught unsupported punctuation glyphs and debug-style remote respawn text; those were replaced with plain HUD text. A final rebuilt native client was inspected in its connecting and no-server disconnected states (`burnhop-connection-review.log`).

The UI tool has no controlled-duration mouse-down/up or standalone Shift-hold API. Sustained fire and movement were injected in the actual native route; physical long holds, cursor exit while firing, focus changes with held Shift/mouse, and human multiplayer feel remain for the checklist. Compatibility rejection is proven through the real transport test; a separately built incompatible native executable was not launched.

## Run and playtest

From `/Users/ayushrameja/codebase/LHP/burnhop-native`, use separate terminals:

```sh
cargo build --workspace --locked
cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
# Offline, independently:
cargo run -p burnhop-client --locked -- --offline
```

The server command must remain running. Each client command opens its own native window. For the automated native route, append `--online-playtest` to each online client command; the explicit test route sends injected commands even while unfocused. A native key/click cancels it. A normal online window always clears device input on focus loss and sends neutral commands. F5 is offline-only. The full commands and platform setup are also in README.

Five-minute human checklist:

1. Move/jump/jet in each client. Confirm the other window follows and check local corrections, remote motion and aim feel.
2. Hold pistol fire, select M416 with 2, fire while moving, then reload with R. Use platforms as cover. Kill the other player and confirm a three-second respawn with full health/fuel/ammo; repeat in the other direction.
3. Hold fire/Shift, switch focus, release outside and return. The server must continue; fresh controls should be required. Try cursor exit while firing. F5 must leave the online match unchanged.
4. Close one client and join a fresh process. The departed actor should vanish, and the fresh player should have a clean loadout. Stop the server and verify the disconnected message.
5. Run offline practice and compare the already-approved controls/weapons. Report one concrete action and observation. Human multiplayer feel, Windows/CI and a later real cross-platform/internet test are the next evidence; extra hosting infrastructure is not needed to review this milestone.

## Subsequent user-reported approval — 2026-09-11

The checkpoint brief reports that the user participated in a successful two-player multiplayer playtest and approved the reviewed multiplayer milestone. This is **user-reported evidence**, separate from the automated transport tests and agent-operated native checks recorded above. The brief does not specify the network topology, latency/loss, machines or platforms used, or individual held-mouse/Shift, cursor-exit, focus, reconnect or disconnect checks. None of those conditions or subcase results is inferred from this approval.

This approval supersedes the original general human multiplayer review pending status. The original handoff and its verification table remain historical. Windows hardware/GPU playtesting, a documented Mac-to-Windows match, real internet conditions and the unreported physical input subcases still need separate evidence. The checkpoint task is authorized to commit and push the reviewed foundation-through-multiplayer work and run GitHub CI; its final evidence will be recorded in [05-checkpoint.md](05-checkpoint.md).
