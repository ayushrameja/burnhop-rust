# Eight-player measurements

2026-09-12, Apple M1 Pro / 16 GiB / macOS 26.6.2 (25G83), native aarch64, Rust 1.98.1. Optimized development profile (workspace opt-level 1, dependencies 3), unchanged pinned dependencies. Desktop applications remained open; this is a shared laptop observation, not an isolated benchmark. Some later soak time overlapped native review.

## Reproduce

From the native repository:

```sh
cargo run -p burnhop-server --example reliability --locked -- --seconds 30 --players 8 --profile baseline
cargo run -p burnhop-server --example reliability --locked -- --seconds 30 --players 8 --profile 50
cargo run -p burnhop-server --example reliability --locked -- --seconds 30 --players 8 --profile 100
cargo run -p burnhop-server --example reliability --locked -- --seconds 30 --players 8 --profile 150
cargo run -p burnhop-server --example reliability --locked -- --seconds 30 --players 8 --profile stall
cargo run -p burnhop-server --example reliability --locked -- --seconds 605 --players 8 --profile 100 --churn
```

The harness binds ephemeral loopback sockets and needs permission for local UDP. Long checks stay out of normal CI. It uses actual Renet/Netcode traffic, the server's 60 Hz clock, 30 Hz snapshots, and eight synthetic prediction clients generating ordinary movement/jump/jet/aim/fire/reload/switch input. No teleport, health, damage, ammunition or tuning manipulation.

## Injection and counter definitions

- Each client connects through a two-socket UDP proxy. Client-to-server and server-to-client datagrams independently receive uniform delay: baseline zero; profile 50 is 25 ±2.5 ms each way; 100 is 50 ±7.5 ms each way; 150 is 75 ±12.5 ms each way. Thus nominal RTT is 50/100/150 ms; total RTT jitter has triangular support ±5/15/25 ms.
- Per-direction independent random packet loss is 0%, 1%, 3% for 50/100/150. Handshakes, ACKs, control and encrypted state packets are all affected. Independent delivery times naturally reorder traffic. No retransmission is added by the proxy. Seeds are `0xB08 + client_index`; OS scheduling and packet arrival order prevent bit-identical reruns.
- `stall` uses profile 100 and stops **all client polling and input generation** for elapsed seconds 8–9. Server and proxies keep advancing, OS receive queues can retain incoming packets, and client clocks see the one-second gap on return. This is a client scheduling stall, not a forced one-second total packet-loss blackout. The check requires fresh applied acknowledgements for every client within two seconds after return.
- Earlier `local_transport` tests impair application messages. Their evidence is separate from this raw encrypted UDP proxy matrix.
- `applied` and `missing` are server command slots after the first four seconds; missing slots use neutral release commands. Spawn/death neutral gates and replacement handshake reservations are still included. `late_copies` counts rejected stale redundant **copies**, not unique lost commands; it is cumulative including warmup. `accepted` is available in opt-in server diagnostics; duplicate/rejected copies are separate from applied slots.
- `app_sent_bytes` is snapshot bytes **enqueued for all recipients**, excludes Welcome/Reject, and is not proof every byte arrived. `app_received_bytes` counts all received application payloads before decoding, including controls. `udp_*_bytes` counts proxy ingress bytes before injected loss, once per direction, including Renet/Netcode headers, encryption, ACK/control traffic; excludes UDP/IP/link headers and avoids counting the forwarded local proxy hop twice. Table rates use decimal kB/s, averaged over full run including handshake/dead actors/churn.
- `snapshot_max` is complete application payload size; `datagram_max` is observed UDP payload size. The exact codec maximum is 1,185 bytes, even when a particular run's maximum is lower.
- Server work timing covers command collection, shared simulation and associated tick bookkeeping; excludes transport polling, snapshot encoding/send and rendering. p95 uses the last 4,096 samples, maximum is lifetime. Client corrections exclude initial/lifecycle resets; p95 is the maximum of current clients' individual trailing 4,096-sample p95 values, not a pooled full-run percentile. Count/max retain departed clients.
- Queue/history/proxy peaks retain departures. Hard bounds are server input window 32 ticks, client history 128 entries (server-window admission stops earlier), snapshot history/one-update output 32, and proxy 512 packets per client. Final steady history peaks are 9–21; stall peak 29. Effects are fixed pools, checked separately.
- RSS is `ps` resident KiB for the **single headless harness process**, including server, eight clients and proxies. Sampled at 5 seconds, then every 30 seconds and at end; not total system memory and not renderer memory. Original short-run JSON's `rss_peak_kib` missed the final sample; the table takes max(reported peak, final). The harness now includes final RSS in that field; raw recorded JSON is retained unchanged.
- `stall_recovered=false` on non-stall profiles is not a failure; the recovery condition only runs for `stall`.

## Recorded matrix

Every 30-second profile kept eight clients connected, produced combat/respawns, stayed in bounds and dropped zero server ticks. The stall profile recovered within its two-second criterion. Missing counts and corrections remain visible:

| Profile | Applied / missing slots | Deaths / respawns | Tick work p95 / max ms | History / server queue / proxy peak | Nonzero corrections / max px |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 12472 / 0 | 48 / 43 | 0.049 / 0.952 | 9 / 7 / 4 | 0 / 0.000 |
| 50 | 12472 / 0 | 48 / 43 | 0.049 / 2.034 | 13 / 7 / 11 | 0 / 0.000 |
| 100 | 12471 / 1 | 44 / 40 | 0.048 / 1.257 | 18 / 8 / 18 | 0 / 0.000 |
| 150 | 12469 / 3 | 42 / 36 | 0.044 / 2.465 | 21 / 8 / 21 | 1 / 0.089 |
| stall | 11888 / 584 | 42 / 39 | 0.051 / 8.566 | 29 / 19 / 17 | 11 / 26.633 |

| Profile | Snapshot enqueue kB/s total | UDP down kB/s total | UDP up kB/s total | Max app snapshot / UDP bytes | RSS initial / final / observed peak KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 281.66 | 300.61 | 73.03 | 1184 / 1211 | 5104 / 5312 / 5312 |
| 50 | 280.45 | 299.34 | 75.40 | 1185 / 1212 | 4848 / 5040 / 5040 |
| 100 | 278.73 | 297.84 | 75.55 | 1183 / 1210 | 4912 / 5216 / 5216 |
| 150 | 276.95 | 296.39 | 75.97 | 1183 / 1210 | 5024 / 5312 / 5312 |
| stall | 278.86 | 297.96 | 73.98 | 1183 / 1210 | 4976 / 5424 / 5424 |
| 100 soak | 281.59 | 301.71 | 76.77 | 1185 / 1214 | 4896 / 5072 / 5936 |

The last row is the 605-second profile-100 soak. It sustained eight seats with brief intentional departure/handshake gaps, 21 total joins (13 replacements), 36,299 ticks, 13,670 shots, 916 deaths and 906 respawns. Applied 287,573 slots, missed 159, one late redundant copy; no dropped ticks. Server work p95 0.037375 ms, maximum 7.157750 ms. History peak 19, queue 8, proxy 18. Reconciliations 141,447, zero nonzero movement corrections by the defined metric. RSS initial 4,896, final 5,072, sampled peak 5,936 KiB; no runaway growth observed. This finite run and sampling do not prove absence of all leaks or establish public-host capacity.

## Failed development experiments and inspection limits

`eight-fixed50.json` preserves a 20-second fixed-six-tick experiment: it stayed connected at 50 ms but saw many stale redundant copies. `eight-fixed150.log` preserves all-eight timeout at about four seconds under 150 ms/3% loss. These used the earlier 1,385-byte schema and uncapped flush cadence. A subsequent adaptive but still sliced 30-second run had 1,979 missing of 12,472 steady slots and a 442.137 px maximum correction. These were sequential engineering experiments, not an isolated factorial benchmark of each change. The final matrix uses adaptive lead, lossless packing and 60 Hz transport flushing together.

Native final frame sample: one 1280 ×720 logical (2560 ×1440 Metal framebuffer) client with seven synthetic combat peers, local player mostly idle/neutral, no captures or forced scoreboard. 180 warmup +1,800 measured frames: mean 11.600 ms, median 8.484, p95 18.101, p99 20.154; 331 intervals over 16.7 ms. This includes VSync, focus transitions and OS scheduling, not GPU pass time or a stable 60/120 FPS claim. Reconciliation count 667, corrections zero, history peak 8, snapshot history 32, lead 6, five skipped initial slots, measured RTT 16.109 ms. See native frame logs. The machine was not isolated.

Both actual native clients completed the scripted two-player route. The actual offline combat route passed all seven checkpoints. The extra focused movement route was canceled twice by native focus loss (after one/two checkpoints); those logs are retained honestly. All 17 movement unit tests and the frozen 12,000-tick practice test passed. Earlier full native movement evidence remains in milestone 06; this run does not re-establish every physical-input subcase.

Initial app-control-attached capture attempts returned blank/stale framebuffers. They were discarded; direct native launches without that attachment produced the valid PNGs linked in the screenshot index. Screenshots are framebuffer evidence, not human playtest evidence. No Windows hardware, eight-human or internet run was performed.

Final native-checkout formatting, strict Clippy, 102 tests, locked host/explicit Mac builds and dependency trees are retained in [checks.txt](checks.txt).
