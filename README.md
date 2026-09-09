# Burnhop Native

A separate native desktop version of Burnhop, inspired by Mini Militia with Apex-influenced movement and combat. This is a learning-driven game project using Rust, Bevy, and Go.

## Start here

Read [project context](docs/PROJECT_CONTEXT.md) for agreed decisions, scope, and open questions. Read [the roadmap](docs/ROADMAP.md) for the next playable milestones. Agents should read [AGENTS.md](AGENTS.md) before working.

## Repository

Remote: `git@github.com:ayushrameja/burnhop-rust.git`

Local folder: `burnhop-native`

## Current status

Planning repository created on 2026-09-09. No game implementation, toolchain selection, dependencies, builds, hosting, or networking are set up yet.

## Intended stack

- Native client: Rust + Bevy, targeting Apple Silicon macOS and Windows.
- Shared gameplay core and authoritative match server: Rust.
- Online services: Go, starting with room discovery and invites.
- Persistence: PostgreSQL when accounts or progression require it.

The existing browser game at `../burnhop` remains a separate project and reference. This repository does not replace or modify it.
