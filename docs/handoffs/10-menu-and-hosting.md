# 10 — Native menus and owned hosting

2026-09-12. Ready for local review in `burnhop-native`, based on saved checkpoint `b64d213b1bdcfaa272848926ae9716fdff56c042`. No commit, push, deployment, agents, gameplay tuning, maps, weapons, external dependencies or dependency upgrades. Existing uncommitted human-approval additions in context, roadmap and checkpoint 09 are preserved. The sibling browser project was not modified.

## Player flows

- No arguments opens a native Bevy main menu: Practice, Host Game, Join Game, Quit. It uses the existing arena/pilots as its backdrop, the canvas/sage/charcoal/cyan palette and bundled Fira Mono. Desktop branding and a single compact card keep the 480 × 320 minimum readable. Hover changes fill, keyboard focus uses a cyan border, press changes fill again, and invalid/connecting actions are visibly disabled.
- Practice starts the approved offline arena. Escape opens Resume / Return to Main Menu. Simulation, reload/respawn clocks and presentation animations pause offline. Focus loss retains the existing offline pause. F5 still resets practice.
- Host Game accepts a numeric **bind IP:port**, default `127.0.0.1:5000`. Loopback is labelled LOCAL ONLY. An explicit interface IP is labelled LAN INTERFACE; bind failure explains choosing a local interface or another port. IPv4 and bracketed IPv6, including numeric scope syntax, use the existing `SocketAddr` transport format. Port zero, unspecified, broadcast and multicast destinations are rejected by the player menu. In particular, `0.0.0.0` and `[::]` are never advertised as join addresses.
- A successful host enters the normal online arena through the ordinary UDP connection and authoritative snapshot handshake. The HUD shows the actual configured join address and snapshot-derived player count. Escape also shows both. For LAN play, enter the host computer's actual interface IP and share that exact address with friends on a reachable network. The UI does not discover interfaces or change firewalls, and makes no internet-reachability promise.
- Join Game has an editable address, validation, disabled Connect for invalid input, a connecting screen with Cancel focused, and retryable errors. Full-server, incompatible-build, timeout and disconnect reasons provide a next action. Edit & Retry preserves the address and returns to the relevant form. Cancellation, failed binding and client failure dispose of any partial session.
- Escape online opens Resume / Leave Match. The server, snapshots, reloads and respawns continue. Local intent is released and neutral commands continue. Guests leave without affecting the server. Hosts see **Stop Hosting...**, followed by a confirmation explaining that everyone disconnects; **Keep Playing** is the default. Confirmed stop, Quit, and normal window close release the owned host.

Menu controls: Tab / Shift+Tab or Up / Down select enabled controls; Enter or Space activates. Escape resumes a paused match, cancels connecting, or returns from a form/error. Click a field to select its contents; use Left/Right, Home/End, Backspace/Delete and Ctrl+A / Cmd+A to edit. Address input is bounded to 64 ASCII address characters, with a caret and a scrolling text slice. It supports numeric addresses, not DNS names or clipboard paste. Hover after screen reflow does not steal keyboard focus. Menu key/mouse events and focus transitions clear gameplay input; a fresh press is required after resuming.

## Ownership and lifecycle decision

`burnhop-server::owned::OwnedServer` binds one existing `Server`, then starts **one owned standard-library worker thread**. The client links the existing workspace server library; it never searches for or launches a server executable. Therefore normal `cargo run -p burnhop-client --locked` can host even when the standalone server binary has not been built. The only manifest/lockfile dependency change is that internal path dependency; no external package is added.

The worker uses the same `Server::poll`, `ServerClock`, `tick`, `snapshot` and `flush` methods as the standalone CLI. The fixed clock admits bounded catch-up (at most five ticks); it never duplicates gameplay rules in the renderer. Polling uses the existing nonblocking transport, followed by a one-millisecond park. Ownership is per session: a stop flag plus unpark requests exit, and Drop joins the worker before a retry can bind. `Server::drop` sends transport disconnections and drops its socket. Binding happens before success; spawn failure drops the moved server; client startup/handshake failure and cancellation drop the owned worker. Worker errors/panics become retryable host errors. No PID lookup or process termination is used by the runtime, so a guest cannot stop a separately launched server.

This is a portable cooperative worker lifecycle, not a child-process supervisor or a hard real-time termination guarantee. The worker shares the client's process failure boundary. Normal application teardown is verified; forced OS process termination relies on OS socket cleanup. No host migration is provided.

Per-session world, combat, input, clock, connection/prediction/history, address, scores and feedback are replaced on exit/start. The feedback epoch resets pooled pilot pose history. Arena, pilot, effect and menu entities are created once and reused, so session transitions do not accumulate UI or gameplay entities.

## Launch commands

From `/Users/ayushrameja/codebase/LHP/burnhop-native`:

```sh
# Ordinary player launch: main menu, including self-contained Host Game
cargo run -p burnhop-client --locked

# Preserved development launches
cargo run -p burnhop-client --locked -- --offline
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000
cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000

# Preserved opt-in routes / diagnostics
cargo run -p burnhop-client --locked -- --offline --combat-playtest
cargo run -p burnhop-client --locked -- --movement-playtest
cargo run -p burnhop-client --locked -- --connect 127.0.0.1:5000 --online-playtest
cargo run -p burnhop-server --locked -- --bind 127.0.0.1:5000 --diagnostics
```

The Cargo commands also apply in a configured Windows checkout. Explicit `--target aarch64-apple-darwin` or `--target x86_64-pc-windows-msvc` remains available. The standalone server still accepts its original bind options. Menu destination validation does not alter its CLI or the explicit client transport parser. All participants still need protocol 2 / gameplay `0x4255_524e_0008_0001`.

Existing `BURNHOP_CAPTURE_DIR`, F9, review Tab and frame-profile options remain opt-in. The capture system now also records menu screens without requiring simulation ticks; it is not part of ordinary player flow. Review captures are asynchronous: allow the screen to settle and inspect the resulting image before changing screens.

## Automated checks

All final local checks passed on macOS 26.6.2 (25G83), Apple Silicon, pinned Rust 1.98.1 / Bevy 0.19.1:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass |
| `cargo test --workspace --locked` | **111 passed**, zero failed/ignored: 38 client, 47 core, 18 protocol, 8 server/UDP |
| `cargo build --workspace --locked` | Pass |
| `cargo build --workspace --locked --target aarch64-apple-darwin` | Pass |
| Independent core and server checks | Pass |
| All-target dependency trees | Core has zero dependencies; server has no Bevy/window/GPU dependencies |
| `git diff --check` | Pass |

Nine new tests cover numeric address/caret/selection bounds; repeated practice reset and input/feedback cleanup; offline pause through the real simulation system without catch-up; occupied bind, retry, cancellation and partial-host client failure cleanup; actual UDP host/guest join, neutral online pause, guest leave, replacement, confirmation, host stop, guest disconnect and same-port rehosting; actionable rejection copy; Bevy keyboard navigation, disabled skipping, stationary-hover focus, Space and Resume-click isolation; constant UI entity counts; and repeated owned-worker drop/rebind without touching an occupied unrelated socket.

The original 17 movement tests and frozen 12,000-tick practice comparison pass **unchanged**. Gameplay-core, codec/prediction, compatibility versions, standalone server CLI, pinned versions and existing CI workflow are unchanged. There was no reason to rerun the long reliability soak: no simulation, wire format, packet scheduling or transport reliability algorithm changed.

## Actual native Mac interaction (agent-operated, not human approval)

Two separately addressable temporary `.app` wrappers launched the actual built Bevy/Metal client without arguments. They are test aids outside the repo, not packaging deliverables. Through native keyboard/mouse controls, the agent verified:

1. Main menu, Practice by mouse, Escape pause/return, compact and desktop menus, keyboard selection, numeric text entry and empty-field disabled validation.
2. Host `127.0.0.1:5000`, join from a second actual window, Player 2 and 2/8 count; leave as guest, observe host remain at 1/8, and rejoin with a fresh generation.
3. Host confirmation, stop, guest disconnect message, and successful hosting again on **the same port**. Normal host-window close also allowed an immediate fresh UDP bind of port 5000.
4. Occupied-port error in a second window while the first kept hosting; edit to port 5001 and successfully host without restarting.
5. Join an unavailable port, Cancel, retry until a real timeout, then edit to the live host address and connect successfully in the same process.
6. Resize to exactly **480 × 320 logical**; readable forms, validation, disabled buttons, practice pause, two-player host menu and Stop Hosting confirmation. Desktop checks used 1280 × 720 logical. Focus switches between the native windows released online input; fresh Escape/menu controls recovered.
7. Final-build recheck of mouse reflow preserving keyboard focus; native bracketed IPv6 entry and successful owned hosting/client connection over `[::1]:5000`; keyboard confirmation and Quit.

Offline pause captures repeatedly recorded the same simulation tick **681** across elapsed time and resizing. The live UDP regression separately verifies that online ticks continue while movement/fire become neutral; screenshots alone do not establish that timing behavior. Tool-driven input is distinct from physical held-key/mouse testing. A few initial rapid key sequences coincident with focus changes needed a fresh press; the accepted input-release contract remains intact. Native UI review found and fixed missing bundled-font arrow glyphs and stationary-pointer focus stealing, with the latter protected by a Bevy test.

Separately, `cargo run -p burnhop-client --locked -- --offline --combat-playtest` ran in the actual renderer and completed **all seven injected combat checkpoints**, ending at the F5 reset checkpoint (tick 1301). This confirms the existing scripted route; it is not a new human combat-feel approval. Two-client scripted combat and the long soak were not repeated; this milestone's two-window evidence is direct menu/connection interaction.

Fourteen nonblank, visually inspected native framebuffer PNGs are indexed in [screenshots](../screenshots/10-menu-and-hosting/README.md). They are lossless PPM-to-PNG conversions, with no compositing or image-generation changes. One manual capture raced Cancel and showed the next screen; it was excluded and replaced with the correct connecting capture. Early glyph-error images are excluded. Existing historical screenshot evidence is preserved.

## Human review checklist and limits

1. Launch without flags. Use both mouse and keyboard to enter Practice. Move/fire, press Escape, wait, resume and F5 reset. Confirm a resume click does not shoot and menu Space does not jump.
2. Host on `127.0.0.1:5000`; join from another native window. Hold movement, mouse fire or Shift while opening menus and switching focus. Release everything, then use fresh controls. Online match time should continue; offline should pause.
3. Leave as guest and rejoin. Confirm the host stays online, departed scores disappear and the replacement has fresh state. Stop Hosting, test Keep Playing first, then confirm; guest should disconnect. Rehost on the same port.
4. Try an occupied port, an empty/invalid address, an unavailable address and Cancel. Edit/retry without relaunching. Check at 480 × 320 and ordinary desktop size.
5. Judge the menu's visual fit with the approved pilot/arena style and report any remaining focus or usability issue before saving this milestone.

No new human approval is claimed. Second-computer/LAN testing is explicitly deferred. This uncommitted tree has **not** run Windows CI or Windows hardware/GPU tests; only the historical checkpoint is green on both platforms, and only the Mac target is installed locally. The preserved workflow can validate Windows when saving is separately authorized. No Mac-to-Windows, real internet, NAT/firewall traversal, public authentication, eight-human feel or new performance claim follows from these checks. Numeric LAN input requires knowing the interface address; automatic interface discovery, DNS and clipboard paste are not included. Existing unsecure direct-connect authentication and prior latency/aiming limits remain.

No implementation blocker remains for local review. Next: human menu/hosting review, then a separately authorized save and exact-SHA Mac/Windows CI. Do not infer permission to commit/push from the earlier checkpoint's historical authorization.


## Menu and hosting human approval — 2026-09-13

The user explicitly reports all six manager-requested checks passed on the local Mac setup: Practice pause/resume; Host Game and Join Game; input recovery across menus/focus; guest leave/rejoin; Stop Hosting and same-port rehosting; and usability. This is user-reported hands-on approval of milestone 10, superseding earlier statements that its human review was pending. Preserve this approved behavior and visual baseline.

The manager independently reran 111 tests, formatting, strict Clippy and the locked Mac build successfully during review. New-change Windows CI, Windows hardware/GPU, deferred second-machine LAN, Mac-to-Windows, real internet and eight-human testing remain separate outstanding validation. Next: a separately authorized save and exact-final-SHA macOS/Windows CI checkpoint, then the web-to-native feature inventory. This approval records playtest results; it does not commit or push the changes.
