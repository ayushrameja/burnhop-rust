# 10 — Menu and hosting native screenshots

2026-09-12, actual macOS Bevy/Metal client. These are lossless native framebuffer PPM-to-PNG conversions, not mockups, composited pictures or generated art. Native keyboard/mouse actions exercised the shipping menu flows. Each retained PNG was opened for visual inspection and checked for nonblank pixel data. Dimensions are 1280 × 720 desktop and 480 × 320 compact (scale 1 for these app-control-attached runs).

| Image | What was observed |
| --- | --- |
| [01 Main desktop](01-main-desktop.png) | Ordinary no-argument main menu, existing arena backdrop and keyboard focus |
| [02 Host desktop](02-host-desktop.png) | Default local-only bind configuration and editable address |
| [03 Join desktop](03-join-desktop.png) | Address retains keyboard focus while Back is hovered, after the final focus fix |
| [04 Main compact](04-main-compact.png) | Minimum-size main menu |
| [05 Host compact](05-host-compact.png) | Bind configuration and editing instructions at minimum size |
| [06 Practice paused](06-practice-paused-compact.png) | Offline pause, including the same tick across elapsed time/resize |
| [07 Match menu](07-match-menu-compact.png) | Compact host menu, real address and two actual native clients |
| [08 Stop confirmation](08-stop-hosting-compact.png) | Minimum-size explanation and distinct confirmation actions |
| [09 Occupied port](09-occupied-port.png) | Real occupied localhost port while another native window hosts |
| [10 Timeout](10-timeout.png) | Real unsuccessful connection attempt and retry action |
| [11 Host disconnected](11-host-disconnected.png) | Actual guest after confirmed host shutdown |
| [12 Native guest](12-native-guest-connected.png) | Second actual native client, assigned Player 2 and 2/8 count |
| [13 Connecting](13-connecting.png) | Actual attempt to unavailable port 5999, Cancel focused |
| [14 Validation compact](14-validation-compact.png) | Empty field, readable validation, disabled Start Hosting, final native build |

Images 01–02 and 04–12 were captured after the typography correction; 03, 13 and 14 were refreshed from the final keyboard-focus build. Hover behavior was then rechecked in the actual final window. Asynchronous screenshot requests can capture a later screen: a manual request racing Cancel showed Main Menu and was excluded; 13 uses the correct inspected automatic capture. Early unsupported-arrow-glyph captures were excluded. No all-black image is retained, and original evidence from earlier milestones was not replaced.

Raw captures/test wrappers and full noisy native logs remain outside Git under `/tmp/burnhop-menu-captures`, `/tmp/burnhop-menu-final-review` and `/tmp/Burnhop Menu*.app`. Durable verification notes and a human checklist are in [handoff 10](../../handoffs/10-menu-and-hosting.md). These images document observed native rendering; they do not prove physical held-input behavior, new human approval, Windows execution or cross-machine/internet reachability.
