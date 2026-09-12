# Eight-player plan and predeclared checks

2026-09-12, before refactoring. Clean baseline: `0a7c7e3fe0692f05a5a9cbaef8da75bd91174957`. Stage in a temporary source copy then conflict-check into native repository. Browser read-only. No agents/commit/push/hosting/tuning changes.

Baseline fmt/strict Clippy passed. First workspace run: 88/90 tests passed; compatibility handshake still Connecting, synthetic impairment queue assertion failed. Serial repeat: all six transport tests passed. Locked baseline build passed. Investigate accelerated UDP scheduling; retain behavior coverage.

## Plan

- Eight stable IDs with generations, one movement/combat implementation, unchanged two-slot offline adapter/frozen practice fixture. Resolve rays from all living bodies before damage; terrain then lowest body ID breaks ray ties; apply damage in shooter-ID order and credit only the first lethal hit per victim. Simultaneous return fire remains valid.
- Validate eight candidates against unchanged geometry/full-body clearance/support. Online joins/respawns maximize nearest living separation; prefer unoccupied bodies, lowest candidate index breaks ties. Contested fallback greatest separation without immunity. Offline exact old spawns.
- Eight gameplay seats plus two bounded handshake/rejection seats. Preserve ownership/rate/input limits/neutral missing commands/timeouts. Clear effects both from and targeting departing slots.
- Bump wire/gameplay compatibility, retain full f64 authoritative state. Measure fixed message size and Renet packetization; 30 Hz replaceable unreliable snapshots; library fragmentation if justified, no custom reliable transport.
- Test six-tick lead first. Only demonstrated failures justify bounded measured-RTT scheduling. Explicit missing slots, never extra simulation, bounded history/queues, stall/clock recovery diagnostics.
- Eight approved pilot rigs with actual slot identities, per-generation interpolation/effects; held Tab scoreboard and compact normal HUD.
- Bounded counters/timing samples, repeatable real-UDP harness. Separate message impairment from raw UDP proxy impairment. Long soak opt-in; CI short behavior tests.

## Intended pass criteria

- Preserve frozen 12,000-tick practice; eight unique ownerships; ninth Full; fresh generations reset state/scores; one death/kill per victim/tick; validated spawns/bounded decoding.
- Each profile: eight remain connected in steady portion, no unexpected protocol/rate rejection; finite in-bounds state; one simulation step per actor/server tick; bounded histories/queues; combat yields kills/deaths/respawns; applied input resumes within two seconds after a one-second stall. Report late/missing/corrections without claiming human feel.
- Baseline; 50 ms RTT/light jitter; 100 ms RTT ±15 ms RTT jitter/1% per-direction loss; 150 ms RTT ±25 ms RTT jitter/3% per-direction loss; one-second bidirectional stall. Record exact one-way distribution/seed/injection point. Measure original six-tick scheduling separately.
- At least 600 wall-clock seconds: eight real UDP clients, movement/fire/reload/respawn and periodic replacement joins. Hardware/build/network recorded; tick work/overload, application versus UDP bytes, buffer peaks and process RSS where measurable. No unbounded growth.
- Actual native Mac renderer plus seven synthetic clients: visible pilots/effects/Tab/compact HUD/departure/reuse/screenshots. Retest offline and two-player. Windows hardware/internet remain unverified.
- Final fmt/strict Clippy/all tests/locked builds; core dependency-free/server GPU-free; preserve platform CI, lengthy tests opt-in.
