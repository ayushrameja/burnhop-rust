# 12 — Web-to-native feature inventory

## Current continuation — Ember Relay design v2, 2026-09-13

Use [ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md](../ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md) and [editable side-view SVG](../design/ember-relay-layout.svg). **Design/documentation only; ready for user design review, not implemented.** Ember Relay remains a working title. V2 supersedes v1’s 17-solid layout and route estimates; earlier inventory/Outpost history below is preserved.

The user likes the original evening direction but found v1 too flat, orderly and safe. Requested: uneven ground/partial cover, smaller staggered floating landings with open air between them, a substantial lower alternate route and real fuel-related fall risk. Geometry must remain original, readable and compact; no MM copying/transformation, random collision roughness or movement/fuel retuning.

V2 retains 3200×1900 bounds and specifies **35 solids:26 rectangles/nine convex quads**. Broad surface slopes/depressions replace flat slabs. Nine 100–160-wide floats cover only 33.1% of the world in combined horizontal projection. Two lower chambers connect through 140/120-unit jump gaps and a 200-wide resting island, with two short 60-clearance crouch choices and four connections to upper combat. V1/V2 void corridors reach the recovery boundary without a safety floor. S0 is supported away from the pits; fixed botB 0 is on the western brow behind initial cover. Other seven spawn candidates are test/future data, not online fairness or random-spawn policy.

Approved behavior is unchanged: native standing movement/jump/jet/fuel (including airborne regeneration), two weapons/bot AI, resource/score-preserving falls, released-input gate and 180 eligible bot-grace ticks; range, visuals, menus and Host/Join remain regression boundaries. Core-owned offline stance/command state must not alter the fixed 36×68 online codec. Nine convex ramps use the same bounded mixed-query approach already proposed, with no concave importer or physics dependency. Camera Standard scale/follow stays native; deepest floor 1640 sets bottom 1735. No online map rollout, detailed assets, music, backend or extra map.

The brief contains exact coordinates, route/fuel arithmetic, full-body landing margins, safe-comparative versus risky routes, lifecycle/input/aim contracts and a native human route. Static geometry/reference checks and rendered-SVG inspection do not prove traversal feel, spawn fairness or combat quality. Playtest narrow landings, rising slope hops, shaft undersides and low-fuel recovery choices after implementation is requested. No material preference question remains unanswered.

This revision updates only the brief, SVG, [context](../PROJECT_CONTEXT.md), [roadmap](../ROADMAP.md) and this handoff. Inventory/Outpost analysis and all earlier work are preserved. No code/builds/dependencies/runtime assets/commits/pushes/deployment/additional agents; web read-only. Temporary rendering previews are QA files, not game assets.

## Preserved inventory and Outpost analysis history

2026-09-13. Completed source inspection and documentation from native checkpoint `7e9610a3d2398e058bfee74bf7fd684b8f0d13fe` and read-only web reference `7a398d4abefa8144fa8949998de4cd76e5dacf8a`. Both working trees were clean before this audit. No implementation, dependencies, generated assets, production access, deployment, commits, pushes or additional agents.

## Deliverables

- [WEB_TO_NATIVE_INVENTORY.md](../WEB_TO_NATIVE_INVENTORY.md): active entry points; map geometry/spawns/art; full appearance catalog; seven-weapon tuning/behavior; all settings/default bindings/storage; audio assets/triggers; backend/API boundaries; provenance; ordered playable milestones; concrete proposed next task.
- [PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md): current inventory and confirmed Outpost direction, preserving prior approvals and distinguishing reported CI from new verification.
- [ROADMAP.md](../ROADMAP.md): inventory completion, recommended migration sequence and next offline Outpost milestone; implementation items remain unchecked.

All documentation links use repository-relative paths for local sources, including the sibling web repository. Web source links require that checkout beside native; the inventory records exact reference SHAs for retrieval elsewhere.

## Confirmed user direction

During this task the user answered the map clarification: **“Outpost — restore the main web multiplayer map.”** The other actual candidate is **Practice range**, already represented in native. There is no active map named “output”; do not ask the map question again.

Keep approved native movement/combat/visuals and the six Mac menu/hosting checks intact: Practice pause/resume, Host/Join, input recovery, Leave/Rejoin, Stop/Rehost and usability. Rust stays authoritative; Go supporting services remain the agreed direction, with actual service reuse/adapter decisions later. Existing Germany hosting is user-reported; official India hosting remains future confirmed intent, not an action taken here.

The task brief reports the checkpoint agent observed macOS ARM64 and Windows x64 CI passing in [run 34740619832](https://github.com/ayushrameja/burnhop-rust/actions/runs/34740619832). The prior [checkpoint11](11-menu-hosting-checkpoint.md) correctly recorded CI as pending at its handoff time. This task records the later **reported** success, without claiming to have queried live Actions or repeated its 111 tests/builds. No new approval of Windows hardware or internet play follows.

## Findings that affect the migration

1. Web solo defaults to Practice range; multiplayer always uses Outpost. Outpost is 4659.2 × 2100 with 16 polygon contours, 8 named spawns and 7 pickup pads. The native range has only rectangles. Concave collision, polygon rays, crouch clearance and open-floor recovery are required before Outpost is playable; decorative bunker back walls must not become colliders.
2. All seven guns are active: Pistol, Revolver, AK-47, M416, UZI, UMP and Sniper. Native pistol/M416 share some catalog values but intentionally differ in reserves, hit regions, exact aim, origin, inventory and respawn. Preserve those approved choices instead of silently copying web balance.
3. Web character detail is procedural Canvas art with a complete local creator, outfits and saved looks. Native already has an approved articulated pilot; customization/state/UI/remaining art need adaptation. No character sprite sheet or cloud cosmetics inventory is hidden in the backend.
4. Web view tiers are weapon-limited and show more world at higher numbers. Native framing and Tab scoreboard differ. Camera options must preserve native defaults and control ownership deliberately.
5. Audio is substantial: 43 weapon WAVs, 12 other SFX WAVs and 3 MP3s, with 2 selected menu tracks and recorded/synthesized gameplay cues. Native audio is absent. Local SFX credits record CC0; music redistribution terms and map/art provenance remain unresolved, not assumed from bundled files.
6. Existing backend is a single-process Colyseus private guest-room/gameplay service. It has invitation lookup, lobby/ready/countdown/results/rematch, reconnection and guest appearance validation, but no account/profile DB, cloud saved looks, cosmetics ownership, public matchmaking queue or durable leaderboard found. It cannot serve native UDP gameplay by changing one URL.
7. Source beats stale prose: README punch is actually E, old music paragraph is obsolete, Outpost spawns are active online, and old camera screenshot names do not reflect current tiers. Practice chooser gives sniper 10 reserve; permanent racks override to unlimited.

## Implementation brief addendum — 2026-09-13

The user requested a documentation-only implementation brief, explicitly confirmed **offline Outpost** scope and prohibited implementation. Read [OUTPOST_IMPLEMENTATION_BRIEF.md](../OUTPOST_IMPLEMENTATION_BRIEF.md) before the original next-task proposal below. It resolves the detailed implementation contract and supersedes that proposal where more specific; context/roadmap/inventory recommendations remain preserved as history.

- Exact current 16-contour geometry (287 authored vertices), continuous polygon collision/rays, corners/slopes/bunker clearance and deterministic player/bot spawns are specified from source. Native's existing three-pass rectangular solver and frozen 12,000-command range regression stay unchanged.
- C/Down held crouch is recommended only for offline Outpost, with web height/0.18-second transition, partial blocked standing and Space/Shift precedence. An explicit core offline command/state wrapper carries crouch and jump-held state; fixed online codec/body dimensions and Host/Join range sessions remain unchanged.
- Keep native Standard framing and Tab scores. **Wide, view-tier controls and framing persistence are deferred** for this first milestone, narrowing the original tentative recommendation. The brief gives exact viewport/bounds/anchor defaults and camera/aim invalidation checks.
- Two material decisions remain: adopt nonpunitive fall recovery with preserved resources, neutral input and 180-tick bot grace (recommended); provide the authored layout/data's ownership or permission context (recommended: reuse confirmed own geometry with original simple native visuals and retained provenance notes). Detailed web art and music can wait; missing LICENSE metadata is not evidence that the user cannot reuse their own work.
- The brief includes native module boundaries, future regression tests and an 8–10-minute human route covering crouch/jet traversal, fall recovery, camera/aim/resize and return to the approved range and Host/Join. These checks have not been run in this documentation task.

This continuation adds the brief and updates only this handoff. Existing inventory/context/roadmap work is preserved; web remains read-only. Source references and local documentation links were checked, along with unchanged fixture hashes and allowed documentation scope. No builds, tests, code/dependency edits, generated assets, commits, pushes, deployment or additional agents. The original audit's four-file scope below describes that earlier task, not the combined current working tree. Implementation still requires a subsequent user request.

## Recommended next task (original inventory proposal)

**Milestone13: playable offline Outpost with polygon collision, tunnel crouch and map-aware camera.** See [inventory section11](../WEB_TO_NATIVE_INVENTORY.md#11-proposed-next-implementation-task--milestone13) for the full entry/exit contract, exact source links, affected systems, compatibility strategy and checks.

Keep range selectable and online Host/Join on its approved range initially. Import all authored Outpost outlines, safe player/target locations and simple readable native material art. Add grounded crouch using web-reference height/transition/clearance because the tunnels require it; preserve standing movement/jump/jet and existing weapons/bot policy. Handle map-specific fall recovery, restart, bot respawn and camera snapping. Add a minimal local framing preference alongside the camera, with approved Standard default and proposed offline Wide option; preserve Tab scores. Full textures, cosmetic catalog, new guns/racks, audio, rooms/backend and distribution stay in later playable milestones.

The map is confirmed. Offline-first scope, Standard/Wide policy, crouch reference values and native-combat-compatible fall recovery are **recommendations for the implementation brief**, not claims that the user already approved every behavior. Any shared snapshot/intent change needs complete codec/prediction state and explicit compatibility handling; an offline label does not excuse omitted stance data. Keep the frozen range fixture immutable and all six approved menu/hosting regressions.

Recommended order after offline map: Outpost in existing direct-connect sessions; character detail/customization; weapons in small batches; audio; complete settings/local persistence; Rust round flow then backend room/invite integration; distribution and broader hardware/internet testing. Basic settings/persistence accompany each feature. Crouch moves forward only because of map geometry; no backend rewrite is a prerequisite for traversing Outpost.

## Verification performed

- Read native instructions/context/roadmap/visual direction/latest handoffs; searched applicable web instructions before inspecting web sources.
- Recorded both full SHAs and clean initial status. Traced browser/native client/server entry points, map and asset manifests, simulation/presentation call sites, settings/appearance/weapon catalogs, backend guard/routes, deployment templates and CI definitions.
- Parsed both authored map JSON files, enumerated 16 contours / 8 spawns / 7 pads, and recorded hashes for map JSONs and Outpost preview. Counted55 WAVs and checked every WAV header: mono 44.1 kHz 16-bit PCM. MP3 decoding/listening not performed.
- Checked local documentation links and named source references; reviewed category coverage, defaults and deliberate native differences against implementations. Checked documentation whitespace and allowed-path diff scope. Final tree should contain exactly the four requested native Markdown changes; web stays clean, SHAs unchanged.
- No full builds, long tests, dependency changes or gameplay execution. Historical test assertions/reports are reference evidence only. Source inspection is not a new visual/audio/hardware playtest.

Left unverified: live Actions conclusions/remote branch state; current backend/provider deployment and performance; external source/license terms; subjective rendering/audio/aim feel; Windows hardware/GPU; second physical machine/LAN; Mac-to-Windows; real internet/NAT/relay; eight-human fairness. Original human Mac approvals remain recorded evidence, not repeated by this audit.

## Resume safely

Read the inventory and current context first. Recheck repository HEAD/dirty state before implementation; this inventory is intentionally uncommitted. Keep the web project read-only, use shared Rust rules, avoid asset reuse claims without provenance, and preserve existing fixtures/tuning. Do not reuse historical save authorization for a new commit/push. No automation, service migration, public deployment or additional agent is part of this handoff.
