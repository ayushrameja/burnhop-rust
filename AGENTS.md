# Working on Burnhop Native

Read `docs/PROJECT_CONTEXT.md` and `docs/ROADMAP.md` before planning or implementing changes. These files carry the user-approved direction across tasks; do not assume earlier chat history is available.

## Project boundaries

- Work in this repository. The sibling `../burnhop` browser project is a reference; do not change it unless the user explicitly requests that.
- Target native Apple Silicon macOS and Windows with Rust + Bevy. No browser shell, Tauri, or Bun runtime.
- Share gameplay rules between client prediction and the authoritative Rust server. Keep rendering, simulation, networking, and Go services separate.
- Go services handle room discovery, invites, and later accounts; do not duplicate movement or combat in Go or route every gameplay update through its HTTP API.
- Official match servers are planned for India. Player-hosted matches support friends in other regions.
- Use an existing engine. A custom engine is not an agreed deliverable.
- Build small playable milestones and validate on both target platforms. Clearly distinguish automated checks from actual hardware playtests.
- Select and verify compatible dependency versions at implementation time; versions are not yet agreed.

## Continuity

- Update the context document when the user changes a decision. Mark proposals and unresolved questions explicitly.
- Update the roadmap as work completes, including checks performed, limitations, and the next concrete action. Do not mark untested features complete.
- Keep secrets and generated artifacts out of Git.

## Communication

Explain changes in plain language. The user values learning Rust, Go, and game development and is tired of working only in TypeScript. Keep progress visible and avoid overwhelming infrastructure work before playable results. Light, situational humor is welcome; avoid forced motivational filler.
