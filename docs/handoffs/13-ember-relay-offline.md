# 13 — Ember Relay offline

2026-09-13. Playable implementation for native review and human playtesting, based on the user-approved v2 layout. Existing uncommitted inventory/design documents are preserved. Browser repository read-only; no dependencies/upgrades, agents, commits, pushes, deployment, new weapons, audio, backend work or online Ember Relay.

## Launch and selection

From `/Users/ayushrameja/codebase/LHP/burnhop-native`:

```sh
cargo run -p burnhop-client --locked
```

Choose **Ember Relay Practice / Offline**, immediately after the unchanged first/default **Practice** action. Mouse or Tab/Up/Down then Enter/Space works. Host Game, Join Game, the standalone server, `--offline`, and the existing movement/combat/online scripted flags retain their range defaults. Leave Ember Relay to restore the range menu backdrop.

A/D moves, Space jumps, either Shift jets, **C or Down holds crouch**, mouse aims/fires, 1/2 selects weapons, R reloads. Escape pauses/resumes or returns to the menu. F5 fully resets the **active** offline map. F1 reveals compact help; Tab keeps its existing scores role. Release all gameplay controls after entry, reset, recovery, or focus/menu transitions before fresh input.

Optional development traversal review (not the ordinary player experience):

```sh
BURNHOP_CAPTURE_DIR=/private/tmp/ember-review cargo run -p burnhop-client --locked -- --ember-playtest
cargo test -p burnhop-gameplay-core --test ember --locked -- --nocapture
```

The injected review has 26 independent traversal fixtures at 60 starting fuel followed by two genuine falls. Initial positions/velocities are explicitly injected **once per stage**, with the attacking bot disabled during traversal fixtures; every intervening movement tick uses the ordinary core. Stages stop/refill for screenshots. It is not a teleport selector, online mode, continuous human circuit, or human approval. Focus pauses offline review; gameplay input cancels the injected route. The last recovery leaves ordinary playable Ember Relay. F9 framebuffer capture remains opt-in through `BURNHOP_CAPTURE_DIR`.

## Implemented behavior and design fidelity

- Exact **3200×1900**, **26 rectangles + nine convex quads** from the approved SVG; nine small floating platforms, two lower chambers, both 60-clearance sleeves and full-height V1/V2. Three specified exterior boundary rectangles; **no global floor**. Exact S0 `(200,1112)` and fixed B0 `(1170,1032)` are the only role spawns. S1–S7 remain validation/future data.
- Dependency-free core owns continuous SAT sweeps, strict overlap, support and rays. Whole convex shapes are swept; only rendering triangulates. Walkable normal ≤−0.55, upward tie preference, zero-duration graze rejection, commanded horizontal slope speed, grounded descent snap and maximum ten contacts with unresolved displacement discarded. Range uses its unchanged three-pass rectangular solver. Movement, spawn/stance clearance, bot visibility, hits and barrel clipping query the same active terrain.
- `OfflinePracticeState` clones world/combat plus map, stance and release state. Standing 36×68; crouch height 54.20060507330696; 11-tick transition with anchored feet and increment-by-increment clearance. Grounded speed scales to 160 at full crouch; airborne tuning stays native. Jump-held/jet intent suppress crouching. Partial stance can remain under the roof and expand after the whole body clears. The pilot bends its existing articulated limbs and lowers torso/head, without scaling/redesigning the character.
- Living top Y>1900 triggers recovery before combat, with checks before and after movement. Pre-crossing skips fuel/movement; post-crossing retains that tick’s fuel effects. Health, ammo/reserve, selection, fuel/delay, scores and timers are retained; an existing reload advances/completes once. New combat intent and bot firing are suppressed on the recovery tick. Fresh grounded S0 motion/stance, a distinct recovery event, neutral gate and **180 eligible bot-grace ticks** replace stale movement/effects. Pause/dead-target time does not consume grace. Ordinary neutral fuel regeneration continues after recovery. Dead actors remain on the native 180-tick death/respawn path. F5 restores fresh loadouts, fuel and scores on the active map.
- Standard 720-world-unit framing, horizontal extent 3200 and bottom camera extent 1735. Camera follows the interpolated standing-feet anchor, so in-place crouching does not bob the camera. Entry/reset/respawn/recovery snaps both poses/camera and invalidates old aim until the snapped frame. Native inverse last-displayed projection, current-body shot origin and fresh-click-after-invalid-cursor/resize rules remain. Compact thresholds and essential HUD are preserved.
- Original native evening gradient, subdued background blocks, cool slate terrain, pale walking rims and amber broken edges. No imported map/art/music. Actual gaps stay visually open. Terrain pools are created once and map-switched by visibility; range art returns on leaving. All five menu actions fit at 480×320 with 36-pixel buttons. Native review corrected the Ember pause heading and desktop help wrapping.

**Geometry/tuning deviations: none.** Route fixture corrections placed the test actor on the actual authored slope height and outside the west ramp; they did not change map coordinates. Crouch timings, fuel, jump, acceleration/braking, weapons and bot policy were not retuned. Decorative complexity remains deliberately modest.

## Automated verification

Final local checks pass on Apple Silicon macOS:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --locked` | **135 passed**, none failed/ignored: 43 client, 66 core, 18 protocol, 8 server |
| `cargo build --workspace --locked` | Pass |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Pass |
| All-target dependency trees | Core zero dependencies; server no Bevy/window/GPU |
| Frozen practice reference | All three SHA-256 hashes unchanged; independent **12,000-command** every-field comparison and Mac golden pass unchanged |
| Lockfile/manifests/protocol/server source | No changes |

New tests cover exact SVG/data identity, every authored spawn and 60-tick support, readable validation failures, permitted exterior-boundary overlaps versus forbidden authored intersections, every ramp in both directions, idle stability and connected seams; thin floors/ceilings/high-speed diagonals and over 1,000 valid randomized extreme sweeps without penetration/contact-cap hits; both sleeves, partial expansion and ceiling thrust; held Space and airborne stance; both real voids; strict threshold and dead precedence; reload/resource preservation, eligible grace and release; clone/replay/F5; cover/open rays; map switching; actual Bevy physical held Shift/Down/mouse and repeat gates; ordered crouch taps; camera clamps and actual inverse-projection round trips at different sizes, DPRs and projection scales. Original tests/assertions and frozen fixture files were not weakened or rewritten to match output. The existing menu keyboard test adds one Tab for the newly inserted second action.

Measured route fixtures all pass at **60 and 100 starting fuel**. Reported fuel cost is maximum depletion before regeneration, not the later settled reading:

| Stages | At 60 fuel: simulated time / cost |
| --- | --- |
| Cover hop / rising west shaft jump | 0.700 / 0.667 s; no jet |
| F01 / F02 / F03 | 1.400 / 1.533 / 1.817 s; 20 fuel each |
| Crown / crown→F06 / risky F03→F05 | 1.533 / 1.867 / 1.867 s; 22.857 / 20 / 20 fuel |
| Four lower hops, both directions, with braking/settling | 0.700–0.783 s; no jet |
| M2 U05→F04→F05→F06 | 1.617 / 1.517 / 1.533 s; 25.714 / 22.857 / 22.857 fuel |
| M1 lower→U12→surface | 0.917 / 1.433 s; 19.048 / 20 fuel |
| M3 lower→U11→surface | 1.533 / 1.517 s; 22.857 each |
| Optional M4 / F09 | 1.567 / 1.600 s; 25.714 each |
| T01/T02 both directions, including stance/approach/stop | 1.733–2.033 s; no jet |

An early slow lower launch deliberately fails its landing criterion; successful lip launches use the specified approach speed. A 20-fuel U05 fixture rests 80 normal ticks to 60 fuel then completes the central climb; a separately tested airborne release/refill/repress case arrests its particular fall. These are specific low-fuel outcomes, not a promise of recovery from every position. Narrow landing success uses active countersteering, not passive coasting. No route failure required geometry correction.

## Native evidence and limits

Actual built Bevy/Metal executable, agent-operated on this Mac; distinct from the headless tests and from human approval:

- Native injected run completes **26 traversals + both true void falls**, checking no interior overlap each tick. Includes ground, F01–F09, both lower chambers/sleeves, M1/M2/M3/M4 and braking on short landings. From neutral top Y 1100 each void recovers after 80 ticks. At recovery HP 57, pistol 7/48 and two kills remain; fuel 31 becomes 71 through ordinary airborne regeneration, then continues regenerating while neutral. This is not a fall refill.
- A second final-build injected run also completes all 26 stages and both recoveries, resized to 480×320 at the beginning.
- Desktop 1280×720, intermediate 800×524 and minimum **480×320** inspected. Final-build main menu entry, reset, pause/resume and focus, map-aware heading, compact help, mouse aiming/fire after resize (12→11 rounds) and active-map F5 (11→12) observed. Range pause snapshots retain the same tick across resizing; Ember pause captures also retain tick 1204 at 1280×720 and 800×524.
- Two actual native windows: Ember→range Host, guest Join as Player 2 at 2/8; guest leave with host at 1/8; fresh rejoin; compact Stop Hosting, default Keep Playing, confirmed stop and guest disconnection; same-port rehosting. Closing the host releases UDP 5000, independently checked by rebinding it. Host/Join always display the original range. Workspace tests separately cover occupied bind, retry/cancel and input isolation. No long network soak was warranted. The hosting log retains ten undersized-packet rejections during teardown/rehost; pinned transport source rejects packets shorter than 18 bytes before decoding. Sender was not captured, so no cause or transport fix is claimed. Observed lifecycle flows succeeded.
- A separate preserved native combat-script attempt canceled on OS focus loss before completion; its log is retained and no new seven-checkpoint completion is claimed. Ordinary native range/hosting interaction and the frozen 12,000-command test provide this task’s range regression evidence.
- [Screenshots and index](../screenshots/13-ember-relay/README.md) use nonblank native framebuffer PPM→PNG conversion, with no image generation/compositing. Labels identify injected fixtures versus ordinary UI interaction. Screenshots establish rendering/readability, **not movement or aiming feel**.

Tool key chords produced native movement and clicks, but do not substitute for physically held Shift/Space/C/mouse tests. Injected held-command routes and Bevy input-resource tests are separate evidence. No new human map approval, eight-human balance/fairness, Windows compilation/CI or Windows GPU/hardware, second-machine LAN, Mac-to-Windows, real internet/NAT/relay, or performance guarantee is claimed. The fixed bot covers the western brow rather than the entire map. No online posture/map ID is encoded; future online adoption requires a separate compatibility/prediction/packet-budget design.

## Human playtest checklist (about 10 minutes)

1. Launch without flags. Compare ordinary Practice, then select Ember Relay. Hop the cover, descend the yard, jump M1 and walk the rising brow; note any seam snag or surprising cover.
2. Ascend F01/F02/F03. Countersteer before short landings, try the crown and F03→F05 shortcut. Rest/refill, then repeat one attempt with low fuel. Judge target readability and waiting time.
3. Drop through M1, cross T01 crouched in both directions, release inside, jump/jet against its ceiling, and stand after clearing the lip. Cross the lower gaps with two separate ordinary jumps and stop on U05; return in reverse. Repeat T02 and exit M3 via U11. Try M2 and optional M4.
4. Fall through each real void with used ammo/fuel; hold controls through recovery. Check preserved health/resources/scores, then release everything and press afresh. Compare F5 full reset and combat death/respawn.
5. Hold a stationary cursor while running/jetting/crouching; resize to 480×320 and back. Check edge aiming, recovery snap and fresh click after re-entry. Exercise actual held controls through Escape and OS focus changes.
6. Return to the range; recheck familiar controls and local Host/Join. Record missed landings, underside bumps, congestion and overall feel before proposing geometry changes. Human approval and any future save remain separate steps.
