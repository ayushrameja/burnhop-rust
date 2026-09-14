# 16 — Local character customization

2026-09-14. Implemented from clean checkpoint `8a13d129226dbaab80cb2b5775bf8caa78bc39f7`. The request reports both checkpoint CI jobs passing 135 tests; this task did not inspect or monitor CI. No commits, pushes, deployment, extra agents, dependency changes, browser edits, map changes or networking changes.

## Launch and use

```sh
cd /Users/ayushrameja/codebase/LHP/burnhop-native
cargo run -p burnhop-client --locked
```

Choose **Character** on the main menu (fifth action). Mouse-click a row to advance it. **Tab / Shift+Tab / Up / Down** selects a control; **Left / Right** changes the selected field, option, facing or pose in either direction; **Enter / Space** advances or activates it. First choose the category with **Edit**, then change its option on the next row. The preview labels saved versus unsaved appearance. Headgear covers scalp hair without changing the selected hairstyle/color. Preview facing and pose are temporary viewing controls.

**Apply & Save** writes the draft and returns to the main menu only on success. **Cancel / Escape** discards edits and restores the saved appearance. **Restore Defaults** previews the original approved pilot; it needs Apply to persist. Choosing the `approved` preset also restores the original recipe. Base/Field/Scout switches retain selected skin and hair, while replacing their supported clothing/headgear fields. On save failure, stay in Character with the draft and an actionable error; the saved look stays intact. At 480×320 an error replaces the keyboard hint so all seven controls remain visible.

Choose **Practice** or **Ember Relay Practice / Offline** to play with the saved look. Standard controls are unchanged: A/D, Space jump, Shift jets, mouse aim/fire, 1/2 weapon, R reload, Escape pause/menu, F5 active-map reset. Ember Relay also uses C/Down crouch. Online continues using the approved shared pilot, even when a local appearance is saved. The screen explicitly says **Offline practice only**.

## Supported catalog and source evidence

| Category | Implemented subset |
| --- | --- |
| Outfit/preset | `approved` (exact native original); `base` (native recipe from active base garment options); `field`, `scout` (adaptations of active web outfit recipes) |
| Skin | `porcelain`, `light`, `tan`, `warm`, `brown`, `deep` |
| Hair | `none`, `buzz`, `crop`, `swept` |
| Hair color | `black`, `brown`, `chestnut`, `blond`, `grey`, `white` |
| Headgear | `none`, `helmet`, `cap`, `beret` |
| Headgear / top / trouser colors, independently | `olive`, `sand`, `slate`, `rust`, `navy`, `forest`, `charcoal`, `cream` |
| Preview | Right/left; standing, crouching, jet, upward aim, animated reload, folded death |

Actual source inspection: web `renderer.ts` calls `drawDetailedCharacter` for gameplay; `CharacterCreator.tsx` uses `APPEARANCE_PARTS`, `COLOR_CHOICES`, `OUTFITS` and `applyOutfit` from `appearance.ts`. Active web outfits are Field/Scout/Heavy; Base is a review recipe plus independently active parts, **not a fourth active web outfit**. `applyOutfit` preserves personal face/hair/build. Native Field/Scout therefore preserve the editable skin/hair selections; the explicitly named approved reset is a native exception. The web `CHARACTER_LOOKS` recipes supply reference gear/color choices, not a claim of full outfit fidelity.

The selected art is project-owned procedural Canvas design/data. Native `artwork.rs` redraws hair/headgear, eyebrows/cheek accents, jacket closure/pockets/harness, pack seams/vents, sleeve/cuff and trouser details, and boot fastenings as polygons. Skin/material/hair palette roles are adapted from the active web catalog. No raster asset, font, downloaded sprite or third-party artwork was imported. Field adds a utility chest panel; Scout adds exposed lower sleeves. Fixed approved boots, pack and rig remain; none of the web build dimensions are ported.

**Remaining gaps:** Heavy, spiky/tied-back hair, face-shape/eye/mouth pickers, editable facial hair/sideburns, eyewear, individual top/trouser/glove/vest/belt/boot style selectors or their remaining color slots, body build, named saved looks, rename/delete/undo, web-save import, dance/reduced-motion/settings parity and online appearance synchronization. Facial features and beard treatment remain recipe artwork, not separately editable fields. This is a bounded native catalog, not web parity.

## Persistence and boundaries

- macOS: `$HOME/Library/Application Support/Burnhop/appearance.conf`.
- Windows: `%APPDATA%\Burnhop\appearance.conf`.
- Other host fallback: `$XDG_CONFIG_HOME/Burnhop/appearance.conf`, or `$HOME/.config/Burnhop/appearance.conf`.
- Version 1, UTF-8 named-ID key/value format. Required header `burnhop-appearance=1`, then exactly the eight supported fields. IDs and color names are independently whitelisted. Freeform hex, missing/duplicate/unknown fields, invalid UTF-8 and unsupported versions are rejected. Reads stop after 4097 bytes and reject files over 4096 bytes.
- Missing file uses the approved default. A failed/malformed/unsupported load leaves the file untouched, uses the approved default and reports the problem in Character. No implicit migration or overwrite. Tests use isolated temporary directories; the ordinary app loads the per-user path only at startup.
- Saving creates an exclusive same-directory temporary file (0600 on Unix), writes and synchronizes it, closes it, then replaces the destination with `std::fs::rename`. Up to 32 temporary-name attempts are bounded; failed saves remove only their own temporary file. Saved in-memory state changes only after replacement succeeds. Tests cover existing-file replacement, failure with a prior config, and exhausted temporary slots. No delete-before-rename, database, account or cloud service.
- Appearance is owned by client `Playground`, separate from world/combat state. Session resets replace simulation state without replacing appearance. The pilot renderer uses saved artwork only for offline actor zero; every bot and online slot uses baseline artwork. No fields were added to core, protocol, snapshots or server. Future multiplayer needs a separately versioned/validated identity metadata contract and bounded peer artwork strategy; it must not trust arbitrary assets/paths from clients.

## Bounded rendering

The existing baseline atlas plus two fixed replaceable atlas handles (saved/draft), each 1024×1024 RGBA at 4× authored resolution: **12 MiB of CPU pixel data** across the three images, plus normal engine/GPU copies. One shared atlas layout. Changed appearance values replace image data on the existing handle; unchanged frames do not regenerate textures. No per-look cache or allocation of new pilot entities on edits.

Eight existing 24-part pilot pools and labels remain. Preview allocates 24 Bevy UI image nodes once plus a container/caption, then updates transforms using the **same `pilot::pose_in`** used by gameplay. Arms use a separate atlas sleeve tile for independent top/trouser colors; approved sleeves reproduce the original limb art. Existing movement/crouch joints, grips, barrel clipping, firing/slide feedback, reload movement, hit tint and death fold/fade are preserved. Appearance never feeds body dimensions, movement, collision, weapon tuning, aim rays or damage.

## Verification and evidence limits

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --locked` | **142 passed**, zero failed/ignored: 50 client, 66 core, 18 protocol, 8 server |
| Frozen 12,000-command range regression | Pass; fixture files and test assertions unchanged |
| `cargo build --workspace --locked` | Pass on this Mac |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Pass |
| Core/server boundary | Core has zero dependencies; headless server/codec/manifests/lockfile unchanged |
| Native rendering | Actual Apple M1 Pro / Metal; 1280×720 and 480×320 captures inspected |

New tests cover catalog round trips; invalid colors/IDs/version, duplicate/incomplete/malformed/oversized files; missing-file fallback; replacement/save failure; Apply/Cancel/Defaults; identity preservation on outfits; native keyboard/focus/input isolation; online exclusion for all eight slots; constant atlas/entity counts; and actual client simulation equality with/without contrasting cosmetics through crouch, jets, fall recovery, death/respawn, pause, reset, menu return, range switching, firing and reload. Every compared world/combat field remains equal. Existing host cancellation/socket-ownership and frozen gameplay tests remain intact.

**Agent-operated ordinary native interaction:** opened Character by keyboard and mouse, edited Field/Scout and deep skin/hair, changed facing/poses, inspected compact controls, canceled drafts, previewed defaults, applied Scout, inspected both maps and F5, closed/restarted the app and verified the saved deep-skin Scout. Two native clients completed Host, Join, leave/rejoin, confirmed host shutdown and guest disconnection. Eight-player rendering used the existing companion with seven synthetic UDP peers; all eight visible pilots retained baseline appearance, with a further jet capture. This is not eight humans or an internet soak. Test-created saved choices were returned to the approved default after verification; no pre-existing appearance file was present.

**Scripted visual evidence:** a debug-only `BURNHOP_CHARACTER_REVIEW_DIR` fixture captures ten deliberately injected poses/UI states using the actual native renderer, then exits. It does not save those fixture looks. It covers contrasting gameplay artwork, both facings, crouch/jet/aim/reload/death, desktop/compact preview, beret and save-error UI. The permission-denied UI fixture injects an unavailable path result; file-system failures are separately exercised in tests. Frames establish presentation, not successful physical movement or gameplay routes. A compact error hint clipping finding was corrected and recaptured.

The existing native `--combat-playtest` attempt emitted initial pistol-fire frames then **canceled on OS focus loss**; no seven-checkpoint completion is claimed. Preserve that qualification. The frozen regression and new simulation tests passed independently. No long network soak rerun was warranted. Native logs retain the previously observed Winit Destroyed warning on closing windows; no renderer panic was found.

[Inspected screenshots and original capture/verification logs](../screenshots/16-character-customization/README.md). No new human approval, Windows hardware/GPU, current-change Windows build/CI, LAN, Mac-to-Windows, internet/NAT or performance guarantee is claimed. The request reports the prior saved checkpoint's CI success, not these changes' CI.

To reproduce injected visual evidence with a debug build (optional, not ordinary play):

```sh
BURNHOP_CHARACTER_REVIEW_DIR=/private/tmp/burnhop-character-review cargo run -p burnhop-client --locked -- --offline
```

The normal F9 framebuffer recorder still requires `BURNHOP_CAPTURE_DIR`. Debug fixtures are excluded from release builds. No release/package/distribution changes.

## Short human playtest checklist

1. Open Character. Try Field/Scout, a contrasting skin/hair combination, no headgear and a helmet. Check both facings and crouch/jet/reload preview; resize to 480×320 and back. Verify outfit changes retain skin/hair.
2. Apply; play the range and Ember Relay. Aim/fire/reload while moving and jetting; crouch in an Ember sleeve. Check joints, readability and unchanged feel against the approved pilot/bot.
3. Pause/resume, F5, die/respawn and fall through a recovery void. Return to the menu and restart the application; the saved appearance should remain.
4. Edit without applying, then Cancel. Restore Defaults, then Cancel. Neither should replace the saved choice. Restore Defaults + Apply should persist the approved original.
5. Host/Join with a friend or second local client. All online pilots should use the approved baseline; offline customization should return when leaving. Physically test held keys/mouse across focus/menu changes.

Ready for review. Human visual/feel approval and any future save remain separate steps.
