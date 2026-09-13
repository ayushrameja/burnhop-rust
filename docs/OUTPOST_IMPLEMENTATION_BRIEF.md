# Offline Outpost — implementation brief

> **SUPERSEDED FOR MAP GEOMETRY AND IMPLEMENTATION DIRECTION — 2026-09-13.** Use [ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md](ORIGINAL_MAP_IMPLEMENTATION_BRIEF.md) and the [original SVG layout](design/ember-relay-layout.svg). The user has not obtained MM layout permission and chose new original geometry rather than retaining/extending Outpost. Do not import, trace, transform or enlarge the coordinates below. D1 fall recovery is now approved (resources/scores preserved, released-input gate, 180-tick bot grace); D2 is resolved by choosing original geometry, not by granting permission. The text below is preserved historical analysis of source behavior, algorithms, camera and codec risks. Its map, spawn, art-import, menu-label and decision recommendations are no longer the current implementation task; the new map only needs rectangles and two convex ramps.

2026-09-13. **Reviewable proposal; no implementation authorized or performed by this document.** Outpost is the confirmed first map. This milestone makes it playable offline while retaining the approved Practice range and existing Host/Join sessions on that range.

Source baseline: native `7e9610a3d2398e058bfee74bf7fd684b8f0d13fe`; read-only sibling web `7a398d4abefa8144fa8949998de4cd76e5dacf8a`. The existing uncommitted inventory, context, roadmap and handoff are preserved. Local source links below require the sibling checkout. Function names identify the inspected implementation; historical test files describe evidence and candidate regressions, not tests run during this brief.

Read with [AGENTS.md](../AGENTS.md), [project context](PROJECT_CONTEXT.md), [roadmap](ROADMAP.md), [inventory](WEB_TO_NATIVE_INVENTORY.md), [gameplay reference](REFERENCE_GAMEPLAY.md), [visual direction](VISUAL_DIRECTION.md), and approvals in [movement](handoffs/02-movement.md), [combat](handoffs/03-combat.md), [menu/hosting](handoffs/10-menu-and-hosting.md) and [checkpoint 11](handoffs/11-menu-hosting-checkpoint.md). Earlier instructions to defer polygons/crouch were milestone-specific; the selected Outpost now supplies that concrete need. Earlier tentative Standard/Wide recommendations are narrowed here to the existing native framing, with Wide deferred.

## 1. Decision and approval ledger

**P — preserve:** retain an existing native contract; rerun its regressions during implementation. **R — resolved recommendation:** necessary new Outpost behavior or a routine technical choice resolved here; no additional preference question, but no claim of prior human approval. **D1/D2 — user decision:** the two material choices in section 8. New Outpost traversal and visuals still require human acceptance after implementation.

| Choice | Existing web evidence | Recommended native behavior | Status |
| --- | --- | --- | --- |
| Entry and map ownership | [arenas.ts](../../burnhop/src/game/arenas.ts) defaults solo to range; [App.tsx](../../burnhop/src/App.tsx) selects either arena; [multiplayer/map.ts](../../burnhop/src/multiplayer/map.ts) fixes web online to Outpost. | Keep current Practice action/default as range. Add explicit offline Outpost action; Host/Join always use approved range. | P existing actions; R new action, section 6. |
| Geometry and traversal | [outpost.json](../../burnhop/public/assets/outpost.json), [collision.ts](../../burnhop/src/game/collision.ts) use real concave outlines, slopes and open underside. | Exact authored coordinates, mixed polygon collision/rays in Rust core; existing native rectangle branch unchanged. | P range; R Outpost algorithm; D2 provenance. |
| Crouch | [stance.ts](../../burnhop/src/game/stance.ts), `updateStance` in [simulation.ts](../../burnhop/src/game/simulation.ts). | Outpost-only feet-anchored 36 × 68 → 36 × 54.20060507330696, 0.18 s transition, clearance checked each expansion tick, grounded half-speed at full crouch. | R faithful traversal prerequisite; range/online stay standing (P). |
| Crouch controls and combat | [controls.ts](../../burnhop/src/game/controls.ts): C/Down hold; Space jump; both Shift keys jet. Web crouch also reduces spread. | C/Down hold only during offline Outpost play; Space/Shift keep current roles. Exact native rays, whole current body hitbox, no new spread bonus. | P approved input/weapon rules; R new posture. |
| Spawn and ordinary death | [outpost.json](../../burnhop/public/assets/outpost.json), web solo `createWorld`/target respawn versus [multiplayer/match.ts](../../burnhop/src/multiplayer/match.ts). | Fixed player courtyard and target coordinates; retain native stationary attacking bot and native 180-tick combat death/respawn. | P combat rules; R map-specific positions. |
| Falling | Web solo `recoverFallenPlayer` preserves resources; online `die` counts a death. | Nonpunitive solo recovery preserving health/inventory/fuel, plus neutral input gate and 180-tick bot grace. | D1: new native recovery policy. |
| Camera | [camera.ts](../../burnhop/src/game/camera.ts), [renderer.ts](../../burnhop/src/game/renderer.ts): default scale 1.5, standing-feet anchor, map bounds. | Native Standard scale 1, existing aspect handling/rates/lag; Outpost bounds and standing-feet anchor; no zoom control/settings this milestone. | P native framing; R map bounds/stance anchor. Web zoom would change feel and is deferred. |
| Visuals | [map notes](../../burnhop/docs/maps/outpost.md), [outpostArtwork.ts](../../burnhop/src/game/outpostArtwork.ts) distinguish solid contours from decorative backgrounds. | Original simple native fills/rims using the authored contours and approved palette/pilot. Detailed web art, preview, fonts and all audio can wait. | P native identity; R simple Outpost treatment; D2 layout provenance only. |
| Network compatibility | Web [multiplayer/map.ts](../../burnhop/src/multiplayer/map.ts) has map-aware compatibility; native [codec.rs](../crates/protocol/src/codec.rs) assumes fixed body dimensions. | Separate explicit offline command/state wrapper; shared core algorithms; unchanged network schema/version/online rules. | P online contract; R offline API boundary, section 6. |

Scope excludes weapon parity, pickups/racks, customization, audio, backend/services, account work, new maps beyond Outpost, online Outpost, new camera presets/settings persistence, package/distribution work and dependencies chosen merely to load this static map.

## 2. Geometry, collision and traversal

### Authoritative map input

[outpost.json](../../burnhop/public/assets/outpost.json) is already in world units: width **4659.2**, height **2100**, `floorY: 1830`, `openFloor: true`, `platforms: []`. Positions are top-left, X right/Y down. [build-outpost.mjs](../../burnhop/scripts/build-outpost.mjs) authors a 3328 × 1152 design grid, scales by 1.4 and adds 180 sky pixels. **Do not apply that transform a second time.** Preserve JSON number values and point order when converting to static native f64 data; no runtime dependency on the sibling repo, browser or generator.

The inspected JSON is 21,990 bytes, SHA-256 `70c6f8a9c7f98cca85be9466b84df3fa29a7487ad466a2033ba796f692ce86d7`. It includes the repaired bunker lips: west right underside raised 18.2 world pixels, both east lips 26.6; about 63 pixels minimum clearance. An older outline would make three passages impassable even while crouched. Source: [map clearance notes](../../burnhop/docs/maps/outpost.md) and [outpost-bunkers.test.ts](../../burnhop/src/game/outpost-bunkers.test.ts).

| Polygon ID | Material | Vertices | Shape |
| --- | --- | ---: | --- |
| west-bunker-roof | bunker | 12 | Concave |
| west-island | rock | 24 | Concave |
| west-sky-stone | rock | 6 | Convex |
| west-sky-island | rock | 9 | Convex |
| west-middle-island | rock | 14 | Concave |
| central-rise | rock | 18 | Concave |
| east-sky-island | rock | 14 | Concave |
| east-middle-island | rock | 15 | Concave |
| east-sky-ramp | rock | 7 | Convex |
| east-sky-stone | rock | 6 | Convex |
| east-bunker-left | wood | 6 | Concave |
| east-bunker-right | wood | 6 | Concave |
| east-base | rock | 59 | Concave |
| central-lower-saddle | rock | 22 | Concave |
| west-lower-bridge | rock | 23 | Concave |
| west-base-and-tunnel | rock | 46 | Concave |

Counts/materials come from JSON; convexity was calculated from successive nonzero cross-product signs during this source audit. Rock contours carry grass decoration. All 16 polygons are solid interiors with colliding sides, tops and undersides; there are no one-way platforms or separately encoded polygon holes. Concave empty passages remain empty. Material/grass do not change friction, damage or collision. Bunker back walls and dark rock columns in the renderer are **decorative**, not additional solids.

`compileArena` in [simulation.ts](../../burnhop/src/game/simulation.ts) adds left/right/ceiling rectangles even to Outpost: `(-W,-H,W,3H)`, `(W,-H,W,3H)`, `(-W,-H,3W,H)`. Preserve these coordinates/order after the contours. They enforce body left ≥ 0, right ≤ W and top ≥ 0 while traversing the world. There is **no floor collider at 1830 or 2100**. `floorY` is camera/scenery metadata here; falling recovery uses top-left Y > 2100. Do not draw a lit horizontal floor where none exists.

### Minimum core changes (R; range remains P)

Use a static map definition containing dimensions, boundaries, original polygon IDs/points/materials, player/bot spawns and open-floor policy. Validate once before entry: finite bounded vertices, positive dimensions, unique IDs, at least three distinct vertices, nonzero area, no self intersections, and whole-body supported spawns. [arenaValidation.ts](../../burnhop/src/game/arenaValidation.ts) validates input structure; [polygon.test.ts](../../burnhop/src/game/polygon.test.ts) adds the full-body/support evidence. Invalid authored data should fail Outpost entry with a readable return-to-menu error; never silently substitute a convex hull or spawn into rock.

Translate `compileTerrain`, `rectOverlapsSolid`, `sweepConvex`, `firstCollision`, mixed `moveAndCollide`, and `raySolidDistance` from [collision.ts](../../burnhop/src/game/collision.ts). Deterministic ear clipping removes collinear vertices, normalizes winding on a copy and splits concave contours into convex pieces; convex inputs stay whole. Cache compiled pieces, axes and AABBs once per immutable map. Collision, stance clearance, spawn validation, bot line of sight and bullet cover must use the same pieces. Rendering fills those pieces but draws only original contour edges: internal triangle seams must not become visible ledges or walls.

Preserve these web mixed-terrain semantics explicitly:

- Epsilon **1e-8**; strict interior overlap is forbidden, touching is legal. Swept AABB versus each convex piece uses its edge normals plus X/Y axes, broad-phase swept bounds and continuous time of impact over the entire requested displacement. No discrete end-position overlap correction or velocity-dependent substep shortcut.
- At equal entry times prefer the smaller normal Y (upward slope face), both between axes and solids. Reject zero-duration corner grazes (`entry >= exit - epsilon`) so triangulation vertices do not snag. Keep deterministic contour/piece iteration order. Test actual opposing corner contacts as well as harmless grazes; do not replace this with the native rectangular tie rule globally.
- Up to **10 contacts** for mixed terrain. Walkable means normal Y ≤ **-0.55** (about 56.6° maximum slope from horizontal). Walkable contact preserves commanded horizontal displacement and sets remaining vertical displacement to the slope tangent; it does not cancel horizontal velocity or let gravity slide an idle pilot downhill. Steeper faces use wall-like tangential projection. Hit flags clear the corresponding velocity components as in web `updateMovement`.
- Ground support is rechecked at the final location. From established ground with downward intended motion, probe down `abs(dx) * 1.52 + 0.5`; otherwise the post-contact support probe is only `1e-5`. Disable established-ground snap during jump/jet takeoff; use the same polygon path for the nine-tick buffered-landing probe. Walking off an edge must lose support and consume ledge grace correctly.
- Ceiling contact blocks upward travel, never grants ground or forces standing. High-speed diagonal ascent must slide below the roof without entering it; descent must land on the first supported surface. The sweep accepts displacements beyond gameplay caps for regression testing. At the contact iteration cap, discard unresolved displacement, keep a valid position and record a diagnostic in test/debug contexts; never apply it unchecked.
- Polygon rays intersect the same solid interiors, including zero distance when starting inside cover. Keep native nearest-cover priority within 1e-8, native body-center origin and current two-gun damage/timers. Bot sight and cosmetic barrel clipping must also see polygon cover. This is collision completeness, not weapon parity.

The native [collision.rs](../crates/gameplay-core/src/collision.rs) currently uses **three** passes and combines equal-time rectangular contacts; web's rectangle-only branch uses four. Retain the **native** implementation, ordering and tolerances for the range. A generalized collision interface must dispatch rectangle-only range calls to that exact path. Do not port web's older range branch over it. Preserve standing movement constants, fuel, jump/jet transitions and combat event ordering in [lib.rs](../crates/gameplay-core/src/lib.rs) and [combat.rs](../crates/gameplay-core/src/combat.rs).

The three source fixtures and their [hash ledger](../crates/gameplay-core/tests/fixtures/approved_practice/README.md) are immutable. [practice_regression.rs](../crates/gameplay-core/tests/practice_regression.rs) must still compare all 12,000 commands against independent frozen code on each target. Do not update fixtures, golden output, tolerances or debug-field normalization to bless a new range result. Shared f64 code and same-platform replay do not establish cross-platform bit identity.

## 3. Crouch contract

Web evidence: [controls.ts](../../burnhop/src/game/controls.ts) `defaultControls`, [stance.ts](../../burnhop/src/game/stance.ts) constants/height, and [simulation.ts](../../burnhop/src/game/simulation.ts) `updateStance`/`updateMovement`/`fireShot`. Web also supports configurable toggle/combined-jet controls; those are outside this milestone.

**Faithful traversal recommendation (R):** width stays 36; standing height 68; crouched height is the exact expression `68.5 * (68 / 85.94)` = 54.20060507330696. Maintain a simulation crouch amount in [0,1]. Each 60 Hz tick approaches its target by `(1/60)/0.18`; full transition completes on tick 11, without pretending it is an 11-tick linear fraction. Height interpolates from 68 to the crouched height. Keep feet Y fixed when resizing; top Y moves. Grounded speed target is `320 * (1 - 0.5 * crouch_amount)` (160 at full crouch); airborne target stays 320. Existing acceleration/braking/gravity/fuel values stay unchanged.

Target crouch only when grounded, C or Down held, neither jump edge nor jump held, no held separate jet intent, and jump buffer empty. Apply stance at the web-equivalent point before this tick's movement/jump processing. Every expansion proposes the next taller whole AABB from the same feet and checks strict overlap against all solids. If blocked, reject **that whole increment**, retain the previous amount/height, and retry every future tick. Partial standing is valid: the web bunker tests explicitly expect height greater than full crouch but less than 68 under the three low lips. Do not require all 68 pixels before allowing any expansion, and do not force the actor downward through a floor.

Jump or Shift intent requests standing but does **not** wait for standing clearance to attempt movement. Under a roof the short collider remains short, jump/thrust follows normal rules and the sweep stops it at the ceiling. Thrust still spends fuel while pushing against a roof; no automatic cancel/refund. In the open, takeoff returns toward standing; airborne crouch hold cannot shrink the body. Holding Space suppresses recrouching after landing until release; this needs an offline jump-held signal, absent from the native wire command today. See web [polygon clearance test](../../burnhop/src/game/polygon.test.ts) for the blocked-crouch jump case.

**Native-specific boundaries (P/R):** enable stance only in offline Outpost. Range/online ignore C as gameplay and retain 36 × 68. C/Down use hold behavior; Down remains menu navigation when a menu owns input. Space remains jump and both Shift keys remain separate jets, never crouch. Preserve A/D, mouse fire, R reload, 1/2 weapons, F5 restart, Escape menus, F1 guide and held-Tab scoreboard. Duplicate events and rapid taps obey the existing ordered buffer; catch-up ticks must not replay press edges.

Pause/focus loss clear pending/held crouch and jump-held alongside existing inputs; frozen simulation retains its current body shape. Resume first sends release/neutral intent and rechecks expansion normally, so a pilot cannot stand through a roof on focus return. Menus must never leak a held Down into gameplay. Fresh Outpost entry, restart and combat respawn start standing at a validated spawn.

Keep the approved single articulated native pilot. Add a lowered hip/torso and bent-leg pose using actual collider height and planted feet; do not scale the whole body or import web character art. Interpolate feet and height, then derive top Y, so intermediate frames do not float or bob the camera. The current rig has a fixed 34-pixel feet-to-aim origin and range-only barrel clearance in [pilot.rs](../crates/client/src/pilot.rs); make these map/body aware for Outpost. Native shots still originate at **current collider center**, and hit the current whole body. Web's crouch spread ×0.75, recoil, weapon offsets, head/leg regions and cosmetic catalog are deliberately not ported. The shorter hitbox and lower aim origin are new Outpost posture behavior, not a claim that standing combat tuning changed.

## 4. Spawns, recovery and resource state

### Authored locations and deterministic selection

All coordinates below are standing body **top-left**, directly from [outpost.json](../../burnhop/public/assets/outpost.json). Its target X literal is `489.99999999999994`; preserve that f64 value even though the UI can display 490.

| Role / named point | X | Y |
| --- | ---: | ---: |
| Player / west-courtyard | 266 | 1232 |
| Solo target (native bot recommendation) | 489.99999999999994 | 1232 |
| west-bunker | 588 | 649.6 |
| west-roof | 551.6 | 336 |
| west-bridge | 1019.2 | 1184.4 |
| central-saddle | 1635.2 | 1232 |
| central-rise | 2357.6 | 781.2 |
| east-courtyard | 3262 | 1232 |
| east-bunker | 4149.6 | 1100.4 |

Recommendation R: always spawn/recover the local actor at west-courtyard; always initialize/respawn the stationary native bot at targetSpawn. Keep all eight named points as validated source data, not a random or nearest-safe respawn system, checkpoint feature or menu of teleports. Validate full 36 × 68 bodies within bounds, no interior solid overlap, downward support within 1 world pixel and no player/bot spawn overlap. Authored spawn bodies should be touching support and remain at their coordinates after 60 neutral ticks, as [outpost.test.ts](../../burnhop/src/game/outpost.test.ts) asserts. Refuse invalid map entry rather than repeatedly recovering into invalid geometry.

Native [combat.rs](../crates/gameplay-core/src/combat.rs) currently hardcodes `BOT_SPAWN`, `CombatState::default`, `spawn_arena` and range `SPAWN_CANDIDATES`. Fix **all** offline initialization/reset/respawn paths through explicit map spawn policy; changing the first player spawn alone is insufficient. Online candidates stay unchanged. The proposed bot is only about 224 pixels from the player rather than the range's 520: encounter timing/damage falloff and visibility can differ because of placement, while its stationary behavior, sight check, 60-tick attack interval and 180-tick initial/respawn grace stay native. No pursuing AI or passive-web-target conversion.

### Web recovery versus death

Web solo `recoverFallenPlayer` in [simulation.ts](../../burnhop/src/game/simulation.ts) triggers only for openFloor and **player top Y > arena height**, checked both before and after movement. It returns immediately to `playerSpawn`, zeros velocity/impulses and jump buffer, clears thrust flags, sets height 68/crouch 0, grounded true and coyote 8. Health, fuel/delay, inventory, reload/equip/cooldown state and accumulated results are not reset. Combat timers can advance normally on that tick (including reload completion), but active fire/reload/melee actions are suppressed. A pre-movement recovery skips that tick's fuel update; a crossing during movement has already spent/regenerated that tick's fuel. The function does not clear physical frontend held keys; a held command can act on the next tick. The test's “input cleared” wording refers to simulation latches, not a new device release requirement.

Web online is different: [match.ts](../../burnhop/src/multiplayer/match.ts) `stepMatch` calls `die` for the same fall threshold, drops weapons, sets health 0, increments deaths and starts 120 ticks; `spawn` restores a pistol with 60 protection ticks. Do not copy this into offline Outpost or alter native online.

### Recommended offline lifecycle (D1 for fall policy; ordinary reset/death P)

| Event | Position/body and timers | Health, ammo and fuel | Input, camera and presentation |
| --- | --- | --- | --- |
| Fresh Outpost / F5 restart | Player courtyard, bot target; standing, grounded, zero velocity, coyote 8, no jump/thrust latches. Clear encounter scores on restart as existing practice does. Keep F5 core tick progression consistent with existing reset; new sessions use their existing fresh epoch. | Native fresh state: health 100, pistol 12 + 48 reserve, M416 30 + 120, pistol selected, fuel 100/delay 0, weapon timers cleared; bot retains native loadout policy. | Clear buffered/held inputs, require neutral before fresh action; snap camera, previous/current poses equal, clear old effects. Start native bot grace 180. |
| Living player falls (D1) | At Y > 2100, pre/post movement check, return to courtyard standing immediately; zero velocity, buffer/latches, set grounded/coyote from validated support. No death, kill or respawn event; emit distinct recovery event. Advance world tick once. | Preserve health, both guns' ammo/reserves, selected gun, fuel/delay at recovery point, scores and life state. Normal combat timers advance exactly once; reload completion can transfer ammo, but recovery grants no refill. No new equip/reload/fire action on recovery tick. | Suppress player/bot shots for that tick; reset bot grace to 180 **after** ordinary tick processing, first countdown next tick. Clear device queues, crouch/jump/jet/fire/reload/selection intent and aim target; require all gameplay controls released before fresh actions. Snap camera/poses and clear transient shot/jet/hit feedback. |
| Combat death / respawn | Keep native 180-tick death timer and scoring; death takes precedence for an already dead actor, with no fall teleport/resurrection. Respawn at map's role spawn, standing. | Native death/respawn resets the respawned combatant to full health/fresh native loadout and fuel through existing lifecycle. Other actor state and encounter scores persist. | Existing neutral gate and bot grace after either actor respawns; clear stance and interpolation at the respawn discontinuity. |
| Pause, focus loss, leave | Offline pause freezes tick/weapon/life/fuel progression and physical stance. Leave destroys Outpost session state. | Pause preserves resources; a later fresh session uses fresh resources. | Clear input and catch-up backlog; no old inputs or Outpost camera/geometry leak into range/Host/Join. |

Implement the recovery check before combat resolution, so a crossing cannot shoot from the void or be immediately shot at the restored point. If already below the threshold, skip movement/fuel update; if it crosses during movement, retain that tick's ordinary fuel change, matching web. Preserved reload state may complete during recovery; resetting resources or every weapon timer would be a different policy. Normal fuel regeneration resumes on later neutral ticks. The neutral gate must include physical held crouch and Space, both Shifts, movement and fire, plus pending actions; held controls cannot silently rearm after a teleport. Preserve facing until a fresh valid aim target. Do not stall time or the entire encounter while waiting for release.

The neutral gate and fresh bot grace intentionally go beyond web recovery. They protect the existing native input-release convention and give the relocated player time to orient without awarding health/ammunition. They also delay bot attacks after a deliberate fall: this is the material gameplay choice D1, not existing approval. The stationary bot cannot normally fall; invalid bot geometry is a validation failure, not a new bot recovery/AI system.

## 5. Camera, viewports and mouse aim

### What web actually renders

[camera.ts](../../burnhop/src/game/camera.ts) defines a **1280 × 720 logical canvas**, not a constant 1280 × 720 world view. [renderer.ts](../../burnhop/src/game/renderer.ts) letterboxes with `viewportScale = min(windowWidth/1280, windowHeight/720)` and centered offsets; DPR affects backing pixels. Default tier 1 has scale **1.5**, so it shows **853⅓ × 480 world units**. Tiers/scales/world extents:

| Web tier | Scale | World width × height |
| --- | ---: | --- |
| 1 (default) | 1.5 | 853.333333 × 480 |
| 1.5 | 1.35 | 948.148148 × 533.333333 |
| 2 | 1.2 | 1066.666667 × 600 |
| 2.5 | 1.1 | 1163.636364 × 654.545455 |
| 4 | 0.75 | 1706.666667 × 960 |

Tab cycles only weapon-allowed tiers (`allowedViewLevels`); pistol allows tier 1, M416 through 2.5, and dual wield limits to 1. This is not an unrestricted optical zoom. Camera tracks interpolated `(left + width/2, feet - 34)`; it deliberately ignores crouch head movement. Web top-left clamps to X `[0, max(0,W-viewWidth)]`, Y `[0,max(0,floorY+95-viewHeight)]`. For Outpost/default that is X up to 3805.866667 and Y up to 1445. Follow is exponential, rate X 20/Y 24, presentation dt clamped [0,.06], lag ≤ 24/scale and 32/scale world units. No lookahead or shake; changing tier immediately recalculates target. It follows `floorY+95 = 1925`, not the full 2100 fall-recovery boundary.

### Exact initial native defaults (P/R)

Keep [adapter.rs](../crates/client/src/adapter.rs) `view_size` and `camera_axis` framing at **Standard scale 1**. For logical window aspect A, world H = `min(720, arenaHeight, arenaWidth/A)` and world W = H × A. Bevy uses `ScalingMode::Fixed { width: W, height: H }`, center coordinates and Y-up transform, filling the viewport without web letterbox bars. At normal desktop and minimum sizes:

| Logical client area | Native world view | Outpost center X bounds | Outpost center Y bounds |
| --- | --- | --- | --- |
| 1280 × 720 | 1280 × 720 | [640, 4019.2] | [360, 1565] |
| 1920 × 1080 | 1280 × 720 | [640, 4019.2] | [360, 1565] |
| 800 × 524 | ≈1099.236641 × 720 | [≈549.618321, ≈4109.581679] | [360, 1565] |
| 480 × 320 minimum | 1080 × 720 | [540, 4119.2] | [360, 1565] |

Use center clamp `[halfView, max(halfView, extent-halfView)]`, extent X = 4659.2 and extent Y = 1925. Follow rates remain 20/24, lag caps 24/32 world units, dt cap .06. Track interpolated feet minus **standing** half-height 34 for Outpost; for standing range this equals the current body-center target exactly. First courtyard frame at 1280 × 720 snaps to center **(640,1266)**, top-left **(0,906)**. Snap also on restart, combat respawn, recovery and session/map change; clamp/recompute on resize. Interpolation must never sweep a pilot or camera across a teleport.

Use the existing compact HUD below width 1050 or height 500 and minimum 480 × 320. Essential health/fuel/ammo stays readable, F1 guide collapsed; add concise C/Down help only in Outpost. At the minimum, the 68-world-pixel standing body is about 30.2 logical pixels high under current scale, consistent with existing native framing. Do not compensate by changing simulation/body size.

**Resolved scope:** no Wide toggle, weapon-dependent view tiers, zoom binding or saved framing preference now. Tab retains scores. Native default shows 1.5× web-default width/height at 16:9, a deliberate existing native difference. Copying web scale 1.5, adding automatic zoom near tunnels, expanding camera bounds to follow below 1925, or changing lag/anchor to current crouched center would alter approved framing/feel and requires a later explicit choice. The camera can lose sight of a sufficiently low falling pilot before Y > 2100 recovery, as the web bounds also do; retain the specified bound and verify the snap, rather than invent a floor or hidden zoom.

### Aim correctness through movement, scale and resize

Web `screenToWorld` subtracts canvas origin/letterbox offsets, divides by viewport scale and zoom, then adds camera top-left. Renderer aim uses the displayed player and current rendered camera even between ticks. Native already has a different approved contract: [combat_view.rs](../crates/client/src/combat_view.rs) `cursor_world`/`capture_aim` invert the **last displayed** Bevy camera `GlobalTransform` and cached projection using logical cursor coordinates, then negate world Y. Preserve it; do not map against an unsmoothed simulation camera or hardcoded 1280/720.

Each capture must use a coherent displayed transform, projection, viewport and active-map epoch. Recompute the world aim target from the latest cursor every frame even when the mouse is stationary: camera motion changes the world point under it. Simulation fires toward that world point from actual current body center; visual weapon direction and confirmed rays retain existing native handling. The screen reticle remains under the physical pointer. Do not claim zero presentation/simulation latency or web-style live-angle parity as part of this milestone.

Existing viewport-size mismatch > 0.5 logical pixel, invalid/outside cursor or lost focus yields no aim and cancels fire/pending fire edges. Preserve the fresh-click-after-resize/re-entry rule. A map switch/recovery invalidates captured aim until the new snapped frame/projection is available; never combine the old world target with the new spawn. If a projection scale changes (aspect/extreme-width resize now, any zoom only in a later milestone), the same inverse-projection path handles it; a same-size scale change must also invalidate stale capture via projection generation, since the current size-only guard cannot detect that case. Do not add a zoom UI just to test the math.

## 6. Selection, core boundaries and affected modules

Keep main menu's current first/default **Practice** action entering the range in one activation; keep Host Game/Join Game/Quit labels and behavior. Insert **Outpost Practice** after Practice, with `Offline / Outpost` identity in that session. This is a small explicit fifth action, not a persistent map setting or a new intermediate screen for the approved Practice action. At 480 × 320 keep all five buttons, focus outline and Quit reachable within the existing compact card; omit redundant subtitle text before shrinking the existing 36-pixel button minimum. Preserve keyboard order and prevent stationary hover from stealing focus. Source: [menu.rs](../crates/client/src/menu.rs) `controls`, `act`, `reset`, `release`, menu layout.

Pause/Resume and F5 operate on the active offline map. Leave returns to the normal range menu backdrop and clears Outpost state/render entities; Practice then starts the approved range. Host/Join and existing `--offline`, `--connect` and scripted range routes retain their current meaning; no CLI expansion needed. Host/Join construct fresh range sessions regardless of the last offline choice, with existing owned-server teardown, cancel/retry/address editing and confirmation rules. No online map selector, map ID handshake, Outpost server spawn, stance input transmission or settings persistence.

Use an explicit core-owned **offline state wrapper** (suggested names `OfflinePracticeState`, `OfflineCommand`): active map ID, world/combat plus crouch amount and neutral-gate state; command contains the existing `InputCommand` plus `crouch_held` and `jump_held`. Clone/restore/replay the complete wrapper, including partial stance and intent gate. The client must not own an authoritative visual-only crouch value. Current body height is derived/validated against stance on state creation/restore, not independently free to disagree with it.

Extract only necessary common movement/combat operations behind a collision/spawn policy or equivalent small internal interface. Both the existing `step_practice`/`step_match` wrappers and new offline Outpost wrapper call the same Rust mechanics for acceleration, jumps, jets, weapon timers, damage and lifecycle. Outpost supplies polygon queries, posture speed/height and its spawn/recovery policy; range supplies the original rectangle/no-stance policy with unchanged evaluation/event order. Do not duplicate the whole movement/combat loop in the client, fork TypeScript rules, fake polygon geometry with bounding boxes, or feed already-stepped movement into a second combat step that moves it again.

| Likely native module | Bounded work |
| --- | --- |
| [gameplay-core/lib.rs](../crates/gameplay-core/src/lib.rs), [collision.rs](../crates/gameplay-core/src/collision.rs); new core map/polygon/stance modules | Static Outpost definition, validation/compiled solids, shared query policy, polygon sweep/overlap/ray/support and buffered landing, offline posture wrapper. Preserve public range behavior/constants. |
| [gameplay-core/combat.rs](../crates/gameplay-core/src/combat.rs) | Polygon cover/sight, map player/bot spawn construction including F5/death paths, shared offline lifecycle and explicit recovery event/gate; unchanged two-weapon tuning. |
| [client/main.rs](../crates/client/src/main.rs), [adapter.rs](../crates/client/src/adapter.rs) | Active offline map/state, ordered extra holds, neutral release ownership, simulation dispatch, feet/height interpolation, map camera bounds/snap. |
| [client/menu.rs](../crates/client/src/menu.rs), [hud.rs](../crates/client/src/hud.rs) | Offline action, small map label/control help, same menu lifecycle/compact sizing. |
| [client/terrain.rs](../crates/client/src/terrain.rs), [pilot.rs](../crates/client/src/pilot.rs), [combat_view.rs](../crates/client/src/combat_view.rs) | Map mesh lifecycle/simple material visuals, stance pose, polygon barrel clipping, coherent camera/aim capture and recovery feedback clearing. Reuse [artwork.rs](../crates/client/src/artwork.rs) atlas. |
| Core tests; [client/tests.rs](../crates/client/src/tests.rs), [menu_tests.rs](../crates/client/src/menu_tests.rs), [playtest.rs](../crates/client/src/playtest.rs) | New Outpost outcome regressions and short opt-in native review route; retain existing range/menu routes. |
| [client/online.rs](../crates/client/src/online.rs), [protocol/lib.rs](../crates/protocol/src/lib.rs), [codec.rs](../crates/protocol/src/codec.rs), server | Compatibility inspection and regression coverage only; retain range rules, current wire layout/versions and packet envelope. No Outpost feature in these paths. |

### Fixed-codec guardrail

Native protocol version is **2**, gameplay version `0x4255_524e_0008_0001`, maximum message 1200 bytes and maximum snapshot 1185 bytes. `Wire for Player` writes X/Y but **omits width/height**; decode reconstructs 36 × 68. Only grounded/thrust-latched/thrusting occupy its flags; decoder rejects values > 7. Wire `InputCommand` has jump edge and jet edge/held, but **no jump-held or crouch-held**. `Hello` has protocol/gameplay fields, no selected map. Adding a Rust field alone cannot make this protocol crouch-safe.

For this milestone, leave the wire-visible types/schema and compatibility values unchanged; the offline wrapper is not serializable as an online snapshot or `NetInput`. Session dispatch must build a fresh standing range world before connecting, with no conversion that silently drops offline stance. Future online Outpost must explicitly version map/rules, serialize complete posture/intent state through prediction/acknowledgement/reconciliation and budget the packet: even one extra f64 per eight players is 64 bytes, exceeding the current 15-byte headroom. Spare flags are not permission to change their meaning. If implementation cannot preserve the old online semantics through the proposed boundary, revise this brief and compatibility plan before merging such a change; never silently keep the version constant after changing network gameplay.

## 7. Art provenance and playable fallback

This is a local evidence ledger, not a finding that the user lacks rights to their own work. Missing LICENSE metadata does **not** prevent an owner from using their own authored files. Conversely, bundling a file alone does not document who made it or any third-party grant. No external license terms were checked during this task.

| Specific files | Existing evidence / what is missing | Milestone disposition |
| --- | --- | --- |
| [outpost.json](../../burnhop/public/assets/outpost.json), [build-outpost.mjs](../../burnhop/scripts/build-outpost.mjs) | [map notes](../../burnhop/docs/maps/outpost.md) explicitly describe hand-authored simplified contours and a Mini Militia layout adaptation informed by videos/map-editor research; say no downloaded TMX, extracted textures or third-party editor code shipped. Generator and data support that local authoring account. No ownership/commission provenance statement or determination concerning the referenced layout's reuse is recorded. | Authored geometry/spawns are required for this exact Outpost milestone. D2 asks for the user's provenance context; do not demand a third-party license for code/data they own or falsely call the classic layout independently cleared. Retain notes and source hash with a later conversion. |
| [outpost-preview.svg](../../burnhop/public/assets/outpost-preview.svg), [outpostArtwork.ts](../../burnhop/src/game/outpostArtwork.ts), [outpostRenderer.ts](../../burnhop/src/game/outpostRenderer.ts), [outpost.worker.ts](../../burnhop/src/game/outpost.worker.ts), [canvasPicture.ts](../../burnhop/src/game/canvasPicture.ts) | Generator/locally drawn Canvas artwork and cache code are present; notes assert all shipped art is locally drawn. No separate ownership/third-party reuse terms found. These are named unresolved provenance records, not evidence of an imported texture pack. | Detailed artwork and selector preview can wait. No worker/cache/runtime port or copying of these assets is required: draw new native material fills on the shared contours. |
| [character.ts](../../burnhop/src/game/character.ts), [characterParts.ts](../../burnhop/src/game/characterParts.ts), [weaponArtwork.ts](../../burnhop/src/game/weaponArtwork.ts), [renderer.ts](../../burnhop/src/game/renderer.ts); [insignia.svg](../../burnhop/public/assets/insignia.svg), [range-banner.svg](../../burnhop/public/assets/range-banner.svg), [visor.svg](../../burnhop/public/assets/visor.svg), [rifle.svg](../../burnhop/public/assets/rifle.svg) | Procedural source/SVGs exist; no blanket ownership grant documented. Last two SVGs have no active source use found by inventory. No claim that missing metadata means infringement. | Not required. Retain existing approved native pilot/weapon atlas/range visuals. Defer character/environment art reuse questions to a milestone actually importing them. |
| [moonwalk-at-sundown.mp3](../../burnhop/public/assets/audio/moonwalk-at-sundown.mp3), [hangar-bhangra.mp3](../../burnhop/public/assets/audio/hangar-bhangra.mp3), [midnight-hangar.mp3](../../burnhop/public/assets/audio/midnight-hangar.mp3) | [audio notes](../../burnhop/public/assets/audio/README.md) call the selected tracks original SoundBreak songs and give track/generation IDs, creation date and cowriters; the third is legacy. No captured account-plan terms or redistribution/commercial-use grant. Missing evidence is the applicable creation-time terms/ownership record, not another MP3. | All music can wait; no music rights question blocks traversal. No audio implementation or asset copying now. |

**Playable visual fallback (R):** use the approved native atlas/font, a subdued original sky/backdrop, flat warm rock, gray concrete and brown timber fills, dark original contour outlines and restrained lit walkable rims. Build meshes from the same triangulation used for collision; leave passages clearly open. No web preview, screenshots, grain textures, branded insignia or music needed. Keep bunker back panels optional and dim enough to read as scenery. This yields the actual route geometry with original simple visuals; artistic detail is not a prerequisite for crouching under a roof.

If clarification concerns only detailed artwork, use this fallback and proceed with confirmed authored geometry when implementation is authorized. If the **layout/geometry itself** is not the user's work or its reuse is unresolved to the user's satisfaction, changing colors does not settle that question. Keep the approved range playable and use original synthetic slopes/tunnels only as nonshipping collision test fixtures; do not silently replace the selected map with a new “Outpost” or add another map. Completing the exact Outpost milestone then depends on D2. No asset generation or import occurs in this documentation task.

## 8. Validation and decisions

### Required implementation checks (future work, not run here)

Run the existing formatting/strict Clippy/locked workspace test and target-build checks appropriate to the eventual change, plus these meaningful outcome regressions. Preserve the current dependency boundary and frozen source hashes. Record Mac and Windows automated results separately from actual hardware/human review; the checkpoint's reported 111 tests/CI are historical evidence only.

| Case | Required observable result / source basis |
| --- | --- |
| Data conversion and compilation | All 16 IDs, 287 authored vertices, materials, 8 named spawns and exact map metadata match inspected JSON; no double scaling or old lip vertices. Reject malformed/self-intersecting/degenerate inputs. Deterministic compilation leaves source unchanged; reversed winding has equivalent solid occupancy/rays. [polygon.test.ts](../../burnhop/src/game/polygon.test.ts), [arenaValidation.ts](../../burnhop/src/game/arenaValidation.ts). |
| Range regression | Frozen 12,000-command state/event equivalence passes on each platform, unchanged fixture bytes; existing movement/weapon/menu/network checks pass. No new stance/debug field is normalized away to conceal a changed range result. |
| Slopes, seams and corners | Adapt the authored concave hill in [polygon.test.ts](../../burnhop/src/game/polygon.test.ts): 140 ticks each direction, uninterrupted support/no overlap, 320 target speed; idle 60 ticks no sideways drift; jump and separate jet leave support. Cross triangulation seams, walk off an edge, test zero-time grazes, equal-time entering corners, steep-face blocking and floor/ceiling mixed boundaries. Actual lower Outpost slopes must support body-width movement both directions where standing clearance exists. |
| Continuous movement and rays | Synthetic displacement (270,900) onto hill and (130,-500) into roof stops/slides without overlap; also test capped full-fuel jet travel, thin edges and actual bunker undersides. Roof test ends at Y=150, X=380 from (250,300), per web fixture. Open U-shaped concavity remains empty for bodies/rays; shots through sky pass, cover ties win, bot sight and barrel clip use the same surface. |
| Tunnel and bunkers | From (700,1460), 360 crouch-right ticks reach X > 1600, remain within 1300 < top Y < 1510, grounded at end, no interior overlap each tick. Adapt all four bunker mouths **both directions**, plus three low-lip release-to-stand cases and roof/floor rays from [outpost-bunkers.test.ts](../../burnhop/src/game/outpost-bunkers.test.ts). Full-width clearance, partial blocked standing and automatic stand after exit are mandatory. |
| Crouch/input replay | 11 ticks to full crouch; planted feet, exact height and speed interpolation; jump/Shift takes precedence without expanding through roof; airborne hold does not shrink. Both Shifts, C/Down, held Space, rapid tap ordering, multiple catch-up ticks, pause/resume/focus loss and Down-menu ownership. Replay clone/restore with partial stance and release gate produces equal full offline states/events on the same platform. |
| Central route/fuel | Adapt [outpost.test.ts](../../burnhop/src/game/outpost.test.ts): from central-saddle move to X ≥ 1920, jet to top Y ≤ 680, translate right above rise, land within 2189 < X < 2690 with positive fuel spent and no overlap. Preserve current fuel budget; failure must not be fixed by increasing fuel/jet tuning. |
| Spawn/death/recovery | Validate every standing spawn and 60-tick support stability; bot reset/death respawns at target rather than (910,1152). Y=2100 does not recover; Y>2100 does, including crossing this tick. Assert exact pre/post fuel policy, once-only timer advancement, reload-completion accounting, no free health/ammo, no fall death/score, neutral gate, camera/effect reset and bot grace. Already-dead player stays on 180-tick native death path. F5 resets resources/scores on active map. |
| Camera/aim | Check exact table extents/clamps and courtyard snap; feet anchor does not move when crouching in place. Projection round-trips at translated/clamped cameras, representative different scales without adding a zoom feature, 480×320/800×524/1280×720/1920×1080 and 1×/2× density. Stationary cursor plus camera motion updates world target; confirmed ray uses actual body center. Stale viewport/projection/map epoch cancels aim/fire until refreshed and fresh click; reticle remains under cursor. |
| Session/network isolation | Menu action/default/focus and compact card fit; Outpost pause/F5/leave → Practice returns exact range geometry, standing body, resources, camera and inputs. Host/Join after Outpost still connect range with unchanged wire messages and snapshot byte counts; no offline state enters prediction/codec. Recheck all six approved Mac menu/hosting behaviors after implementation. |

### Short native human route (about 8–10 minutes)

1. At 1280 × 720 choose **Outpost Practice**. Confirm player in west courtyard, bot at the nearby target, readable health/ammo/fuel and no unexpected shot until grace ends. Move/fire at the bot, then fire into a rock slope and bunker roof: cover must stop the confirmed ray. Keep native pistol/M416 behavior.
2. Descend into the western lower tunnel near X≈700, Y≈1460; hold C and walk right beneath both shelves toward X>1600. Release under the low ceiling, try Space and a brief Shift burst: no pop-through or forced standing; fuel spends normally. Walk into clear space and observe gradual standing without camera bob.
3. From the central lower saddle near (1635.2,1232), move toward X≈1920, jet up and right to land on the central rise. Idle on a slope, walk both ways, jump and test a wall/ceiling jet contact. The pilot must not snag a triangle seam, sink or receive free fuel.
4. Jet to the west bunker and east bunker as a brief entrance tour; crouch through their mouths and release beneath the three low lips. Automated bidirectional fixtures cover the full matrix; human check emphasizes clearance readability, partial standing and walking out. Use the existing opt-in review route to stage distant checkpoints if needed, not a shipping teleport menu.
5. Let go of thrust over a gap and fall out of the world while holding movement/fire. Confirm one courtyard recovery, unchanged health/ammo/fuel except ordinary tick progression, no death/kill added, clear old effects and a snapped camera. Held controls must wait for release; fresh controls work. Confirm bot grace, then use F5 to distinguish full restart from recovery.
6. At 480 × 320 and 800 × 524, aim at a recognizable contour while moving/jetting with the cursor stationary; reticle stays under the cursor and shots hit the terrain under the mapped aim. Resize during held fire, re-click after refresh; test focus loss/pause under a ceiling. Return to desktop size and check map-edge camera clamps, no zoom surprises and readable compact HUD.
7. Leave to menu, choose ordinary **Practice**, check approved range spawn/movement/aim/F5 and standing body. Then exercise existing Host/Join, input release, Leave/Rejoin, Stop/Rehost and minimum-size usability in the approved local setup. Online must show range and never accept Outpost crouch. Record actual hardware and any failed action, not merely a screenshot or build success.

### Only material user decisions

1. **D1 — Offline fall policy.** Recommended: adopt the nonpunitive recovery in section 4, preserving health/ammo/fuel and scores, requiring fresh input after release, and giving 180 ticks of bot grace. This extends web solo recovery to fit native attacking-bot/input behavior. The material alternative is counting falls as native combat deaths; that would need different resource/score/respawn acceptance expectations.
2. **D2 — Provenance of the authored Outpost layout/data.** Supply whether `outpost.json`/`build-outpost.mjs` are your own authored work or were supplied under permission, and any known restrictions concerning the referenced layout. Recommended disposition: reuse your confirmed authored geometry with its existing research/provenance notes and original simple native visuals; defer detailed web art and every music asset. Missing license metadata alone is not a reason to withhold your own work, and no music licensing decision is needed for this milestone.

No decision is requested about engine/dependencies, map selection, online rollout, camera presets, key remapping, weapon parity or backend scope. Those are either resolved above or deliberately deferred. This brief itself does not authorize implementation, asset imports, commits, pushes or deployment.

## Documentation audit

This task inspected the applicable instructions, existing inventory/handoff/context/roadmap/approved constraints and named web/native source implementations; parsed map coordinates/counts/convexity and checked local source/document links. It did not run builds, gameplay tests, native/browser sessions, CI queries, dependency changes, asset generation or external provenance research. Only this brief and a focused addition to [handoff 12](handoffs/12-web-to-native-inventory.md) are changed by this continuation. Prior inventory/context/roadmap work and frozen fixtures remain intact; web remains read-only.
