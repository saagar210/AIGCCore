# AGENTS.md (Repo Root) — AIGC Core

This file defines canonical project commands, paths, and repo-specific conventions for this workspace.

## Canonical Paths
- Rust domain core: `/Users/d/Projects/AIGCCore/core`
- Tauri shell + command handlers: `/Users/d/Projects/AIGCCore/src-tauri`
- React UI: `/Users/d/Projects/AIGCCore/src`
- Local validator CLI(s): `/Users/d/Projects/AIGCCore/tools`
- Packet-driven docs created in-repo: `/Users/d/Projects/AIGCCore/docs`

## Canonical Commands
Primary runner is `pnpm` and Rust `cargo`.

- Install deps: `pnpm install`
- Dev (desktop): `pnpm dev`
- Build (desktop): `pnpm build`
- Run all eval gates locally: `pnpm gate:all`
- Rust tests: `cargo test --workspace`

Source of truth for scripts is `/Users/d/Projects/AIGCCore/package.json`.

## Hard Rules (Packet-Aligned)
- Offline-by-default is enforced in Rust core; UI must not have direct egress.
- Adapters are loopback-only (`127.0.0.1`) and must implement Annex B v1.
- Evidence Bundle exports must comply with Annex A v1 + Phase 2.5 lock addendum.
- Determinism mode must follow Addendum A + ZIP hardening rules.
- Audit trail must be canonicalized and hash-chained per lock addendum and taxonomy.

