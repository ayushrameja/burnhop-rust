# Burnhop — field range

Milestone 06, 2026-09-12. Primary reference: the sibling browser game's actual `34-character-field-detail.png`, `38-character-jet.png`, and `ui-refresh/practice.png`, plus `character.ts`, `weaponArtwork.ts`, `renderer.ts` and arena data. Also inspected the actual native placeholder window before replacing it.

## Identity and composition

Keep the expressive oversized head, field cap, sage armor, articulated trousers and substantial jet boots. Use one illustrated field pilot with a small cyan equipment stripe for the local player, ochre for opponents, plus explicit YOU / BOT labels in practice and YOU / P1–P8 identity labels online. Practice and online use exactly the same artwork and pose builder. No customization system.

The arena becomes a quiet desert foothill training range. Broad low-contrast mountain layers, a pale moon, sparse scrub and a modest range sign establish depth. Solid terrain has a dark cut face, warm aggregate facets, a clearly lit sage top surface, and small inset construction marks. Every solid is drawn from the existing collision rectangles. Scenery is behind solids and actors, lower in contrast, without bright platform-like edges. No new geometry or invisible colliders.

## Palette

| Role | Hex |
| --- | --- |
| Ink / outlines | `202b29` |
| Sky / charcoal | `182727` |
| Far / middle landscape | `293b39` / `344943` |
| Earth face / stone | `4b5043` / `686955` |
| Sage armor / terrain top | `8d9b70` / `b4bc8b` |
| Canvas / readable text | `ede4c9` |
| Skin / shadow | `ddb47d` / `a87751` |
| Local equipment / jet / fuel | `75d4d0` |
| Opponent / low fuel | `dbab6a` |
| Damage / combat warmth | `e89a70` |

No bloom, neon framing, full-screen flashes or screen shake. Health and state changes always include words or numbers, not color alone.

## Character and aiming contract

The body remains 36 × 68 world pixels, feet at body bottom. Head including cap occupies approximately the upper 23 pixels; torso/belt 22; articulated legs and soles the lower 25. The visible head and armor largely fill the hitbox without a large invisible margin. Outlines and internal highlights survive the normal camera scale and Retina rendering.

Use a single startup-generated RGBA atlas from small authored polygon drawings. Keep head, torso, pack, upper/lower limbs, hands, boots and weapons separate sprites. Reuse atlas handles and entities. This is maintainable code artwork, with no imported asset/license dependency. Legs use two fixed-length segments and a displacement-driven stance/recovery cycle. Arms solve from shoulder to the actual weapon grip. Limbs connect; the body never scales during animation.

The weapon pivot stays at the approved body-center aim origin. Pistol and M416 have separate silhouettes: compact slide/grip versus stock, sight, long handguard and magazine. Weapon artwork scales to the existing decorative barrel length. Terrain in front of the origin clips the visible weapon assembly, and muzzle flashes/tracers start at the confirmed ray origin and stop at its confirmed endpoint. No visual claim that a barrel sticking through cover can fire through it.

## Animation and effect limits

Idle breath is less than one pixel on head/shoulders; soles and weapon origin remain fixed. Walk phase follows actual rendered displacement, reversing naturally during backward motion. Stance feet counteract body travel; recovery lifts at most five pixels. Jump/fall tuck the legs; thrust separates the boot stance and shows two small cyan exhaust cones. Hit response is a brief warm tint; death folds the same rig locally and fades it, with the HUD explaining respawn. No flying ragdoll or confusing collision trail.

Reload moves the magazine/support hand cosmetically from authoritative remaining ticks; firing, reloading, equip, death and respawn never change gameplay timers. Confirmed shot recoil is limited to the slide/bolt detail, not the ray angle. Effect slots are allocated once: at most 24 shots, each with one tracer, one origin flash and three endpoint sparks, plus sixteen exhaust cones across eight actors. Tracers expire within 0.10 seconds, impacts within 0.18; effects clear on actor lifecycle/absence, reset and disconnect. Existing online shot deduplication is retained.

## Native HUD

Top left: compact Burnhop / practice-or-direct-connect identity. Top right: opponent or full connection/error status. Bottom left: health and fuel, numeric values, low/empty labels and bars. Bottom right: selected weapon, large magazine count, reserve and reload/equip/empty status. Compact control guidance along the bottom. Center message appears only for pause, death, or terminal connection states. Keep the aiming lane clear. Below 1050 logical pixels wide (or 500 high), use fixed, short readouts at the top corners: HP/fuel at the left and weapon + magazine/reserve + state at the right. Remove bars, the mode subtitle, duplicate weapon heading and routine score text before reducing essential type. Keep combat text at 12–16 logical pixels, with 4-pixel panel padding and 6-pixel side insets. Routine opponent/connection status is one short line at the bottom edge, without padding. Compact controls are collapsed by default behind “F1 Controls”; F1 reveals/hides the guide, and focus loss or returning from desktop collapses it. Long connection errors wrap in the central state overlay; never duplicate their full text in a compact corner. Death/respawn, low/empty fuel/health and reload/empty-ammo instructions remain explicit. Standard 1280 × 720 layout stays unchanged.

Use Bevy's existing bundled Fira Mono for clear numeric hierarchy and range-sign lettering. No extra font files. Avoid the old full-width multiline diagnostic slab; technical timing/route diagnostics stay in opt-in logs.

Review is the playable native result. Milestone 08 reuses this direction for eight players and adds a centered held-Tab scoreboard with identity/kills/deaths, readable at the existing 480 × 320 minimum. It closes on release, focus loss or disconnect; ordinary combat retains the short edge HUD. Native menus are added in milestone 10; audio and additional maps remain deferred.


## Native menus — milestone 10

Reuse the approved arena and pilots as a darkened backdrop. Desktop uses left-aligned Burnhop / Field Range branding and a single right-hand action card; compact windows use one inset card with the game identity in its heading. Canvas text, sage framing and dark field-green buttons retain the existing palette and Fira Mono. Hover brightens the fill, press changes it again, and a cyan outline identifies keyboard focus; disabled actions are muted and skipped during navigation. Stationary hover after reflow cannot steal keyboard focus. Use supported plain-text key names instead of missing arrow glyphs.

Address editing, cancellation, retry and host-stop confirmation belong in this card; technical route/frame diagnostics remain opt-in. Offline pause freezes the arena and animations. Online menus dim the ongoing arena and explicitly say the match continues with released input. Compact hosting menus include the actual connection address and player count. [Milestone 10 screenshots](screenshots/10-menu-and-hosting/README.md) record 480 × 320 and 1280 × 720 native checks. The user subsequently approved all six menu/hosting checks, including usability, on 2026-09-13; see the preserved approval addendum in [handoff 10](handoffs/10-menu-and-hosting.md).
