# Browser gameplay reference

Reviewed 2026-09-11 at browser commit `7a398d4abefa8144fa8949998de4cd76e5dacf8a`. The browser working tree was clean. Paths below are relative to the read-only sibling `../burnhop`. This is source inspection, not a new browser playtest or a promise to reproduce every behavior.

## Movement and time

`src/game/simulation.ts` exposes a headless `stepSimulation(state, command, arena)` with explicit ticks, actor identity, mutable state, and returned events. It rejects mismatched ticks/actors. `src/game/timing.ts` accumulates elapsed time at 60 Hz, clamps frame input to 0.1 seconds, allows five catch-up ticks, then drops excess backlog. Rendering interpolates previous/current state. Retain explicit fixed-step rules; do not copy the browser catch-up policy into a future authoritative server without considering server overload separately.

Quantities are world pixels and seconds, with named integer tick timers:

| Setting | Reference value |
| --- | --- |
| Tick | 1/60 second |
| Standing collider | 36 × 68 |
| Horizontal speed | 320 pixels/s |
| Ground / air acceleration | 3800 / 2300 pixels/s² |
| Ground / air braking | 4200 / 320 pixels/s² |
| Gravity / jump impulse magnitude | 1500 pixels/s² / 520 pixels/s |
| Maximum jet rise / fall speed | 480 / 740 pixels/s |
| Coyote / buffered jump window | 8 / 9 ticks |
| Jet acceleration | 3600 pixels/s², tapering to half force |
| Fuel capacity / drain / regeneration | 100 / (40 ÷ 1.4)/s / 30/s |
| Regeneration delay | 24 ticks (0.4 s) |

Horizontal velocity approaches a target with separate grounded/airborne acceleration and braking. A grounded or coyote press jumps. Holding that first combined jump press does not automatically jet; a fresh airborne press latches thrust until release, fuel exhaustion, or landing. A separate jet binding supports direct takeoff. Buffered landings probe the actual horizontal and vertical path over the next nine ticks; a nearby platform alone is insufficient. Fuel force is full for the first tenth of the tank, then tapers; the final partial tick spends only remaining fuel. Fuel exhaustion requires a fresh press before reactivation.

`src/game/stance.ts` resizes from planted feet over 0.18 seconds; standing expansion checks clearance. Fully crouched grounded speed is half normal. Preserve these as later behavior references, not extra scope for foundation or mandatory crouch work in the next milestone.

## Collision

`src/game/collision.ts` uses swept axis-aligned body rectangles, preventing thin-platform tunnelling over an entire tick. Rectangles use a four-iteration contact/slide path; mixed polygon terrain uses continuous separating-axis sweeps with up to ten contacts. Concave contours are split with deterministic ear clipping and cached without mutating map input. Touching a surface is legal; strict interior overlap is blocked.

Walkable polygon normals have Y ≤ -0.55 in browser coordinates. Grounded descending motion snaps to support; jumping/jet takeoff avoids that snap. Slope walking preserves horizontal speed and idle actors do not drift sideways. Equal-time slope/corner contacts receive special handling to avoid invisible walls at seams. Ground support is checked after moving off an edge.

`compileArena` adds side walls, a ceiling, and a floor unless `openFloor` is true. Practice open-floor actors recover at spawn after falling below arena height. Retain collision in the shared core and presentation in the client. Start the next arena with rectangles; defer polygon parity until there is a concrete need.

## Camera and coordinates

`src/game/camera.ts` and `src/game/renderer.ts` use a 1280 × 720 logical viewport. The camera follows the interpolated standing-body anchor (`x + width/2`, `feetY - standingHeight/2`) so crouching does not bob the view. Camera position clamps to map width and `floorY + 95`; it uses exponential following at rates 20 horizontally / 24 vertically, with lag capped to 24 / 32 logical pixels and presentation dt capped at 0.06 seconds.

View tiers 1, 1.5, 2, 2.5, 4 map to rendering scales 1.5, 1.35, 1.2, 1.1, 0.75. Higher tier means more visible arena. Weapon-dependent tier restrictions belong to later combat work. Keep camera smoothing client-only; start with one readable framing for movement.

Browser rectangles use top-left X-right/Y-down coordinates. Bevy sprites use center X-right/Y-up positions. Proposed import boundary: keep reference world units and Y-down simulation data, then render a rectangle at `(x + width/2, -(y + height/2))` with size `(width, height)`. Convert camera bounds and pointer coordinates consistently. Decide and test this before implementing movement; the current placeholder uses Bevy coordinates directly and imports no map.

## Map data worth carrying forward

`src/game/types.ts` defines finite dimensions, `floorY`, rectangular platforms, player/target spawn top-left positions, optional terrain polygons, named multiplayer spawns, pickup pads, theme, and `openFloor`. Pickup pads use horizontal center and surface Y, unlike actor spawns. `src/game/arenaValidation.ts` validates bounds, positive dimensions, IDs, materials, and non-degenerate/non-self-intersecting polygons. Future import validation should also verify whole-body spawn clearance and support.

| Map | Size / floorY | Geometry and spawns |
| --- | --- | --- |
| `public/assets/arena.json` | 2400 × 1350 / 1220 | Four rectangles, solid floor, player (390,1152), target (910,1152) |
| `public/assets/outpost.json` | 4659.2 × 2100 / 1830 | 16 terrain contours, zero rectangles, eight named spawns, seven pickup pads, open floor; player (266,1232) |

Practice platforms `(x, y, width, height)`: `(110,960,300,42)`, `(570,755,350,44)`, `(1170,955,330,44)`, `(1730,670,370,46)`. These are useful data for the next simple arena. Outpost's rock/bunker/wood materials and grass flags mix collision geometry with presentation hints; preserve the distinction. `src/multiplayer/map.ts` validates/freezes the same authored JSON for client and server and hashes tuning/map data for compatibility. Defer compatibility protocols and serialization dependencies.

No artwork, fonts, audio, or JSON assets were copied. Check provenance/licenses and formats before later reuse. The user's reported map and latency issues remain undiagnosed; a language change does not establish a fix.

## Behavioral checks to adapt next

Read `src/game/simulation.test.ts`, `src/game/polygon.test.ts`, and `src/game/camera.test.ts`. Prioritize speed/braking, floor/wall/ceiling contact, high-speed thin-platform sweeps, ledge grace consumption, buffered landings, jet press/release/exhaustion, and identical command outcomes across 30/60/144 Hz presentation. Later polygon checks cover seams, slopes, concavities, standing clearance, and supported spawns. Shared Rust code alone does not prove cross-platform floating-point determinism.

## Combat reference for milestone 3

Inspected at the same browser commit above; the browser working tree remains untouched. Sources: `src/game/weapons.ts` (`WEAPONS`, `WEAPON_HANDLING`, `createWeapon`, `equipWeapon`, `advanceWeaponTimers`), `src/game/simulation.ts` (`getWeaponOrigin`, `fireShot`, `advanceCombat`, `applyPracticeCombat`), `src/game/combat.ts`, `src/game/stance.ts`, and `src/game/collision.ts` (ray intersections).

| Browser tuning | Pistol | M416 |
| --- | --- | --- |
| Trigger | Repeats while held | Repeats while held |
| Magazine / starting reserve | 12 / unlimited (`-1`) | 30 / unlimited (`-1`) |
| Cadence | 12 ticks / 0.20 s / 5 shots/s | 7 ticks / 0.1167 s / ~8.57 shots/s |
| Reload | 72 ticks / 1.20 s | 114 ticks / 1.90 s |
| Body damage | 18 | 23 |
| Falloff start / end | 300 / 800 pixels | 800 / 1600 pixels |
| Minimum damage factor | 0.40 | 0.80 |
| Maximum ray range | 1000 pixels | 1900 pixels |
| Base / maximum spread | 0.45° / 1.8° | 0.35° / 1.6° |
| Recoil per shot | 0.45° | 0.7° |
| Head / leg multiplier | 1.75 / 0.75 | 1.75 / 0.75 |
| Scaled artwork muzzle length | 17 × 68/85.94 ≈ 13.45 pixels | 34 × 68/85.94 ≈ 26.90 pixels |
| Maximum view tier / dual wield | 1 / supported | 2.5 / unsupported |

Neither selected weapon has a moving gameplay projectile or bullet speed. The browser resolves an instant ray from the weapon origin against the closest terrain, then accepts a target body-region contact only if it is strictly closer by more than 1e-8. Cover wins exact/near ties. The tracer begins at the cosmetic muzzle unless the hit is closer than the barrel, in which case it starts at the origin. This prevents barrels protruding through walls from bypassing cover. The browser handles rectangles and terrain polygons; the native practice arena has rectangles only.

Standing single-weapon origin is collider center X and planted feet Y minus `46.94 × 68/85.94` (about 37.14 pixels). Direction includes deterministic hashed spread, stance-dependent bloom, and accumulated recoil. Bloom reaches its cap over six shots, recovers over 36 released ticks; recoil caps at 8° and recovers by 0.2° per released tick. Airborne spread is ×1.5 and full crouch spread ×0.75. Body damage uses linear distance falloff and rounds to the nearest integer. Head/leg regions have their own multipliers.

Timers advance before firing; a reload completion can fire on that completion tick when the trigger is held. Empty magazines require manual reload. Reloading cannot fire. Equip cancels reloads, imposes at least 18 ticks of equip delay, and transfers the weapon instance's ammunition and longer cooldown without resetting it. The browser equipment and dual-wield system is intentionally not ported.

The native milestone keeps held triggers, cadence, reload time, magazine sizes, body damage, falloff, range, and cover priority. Deliberate changes: finite player reserves (48 pistol / 120 M416), exact aim without spread/recoil, one whole-body hitbox, ray origin at body center, one fixed view, two carried weapons selected by 1/2, and a stationary attacking bot. Both actors respawn after 180 ticks; browser practice target respawn is 120 ticks and its target does not attack. See [03-combat.md](handoffs/03-combat.md) for the authoritative state contract and validation.
