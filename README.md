# ICARUZ
All-in-one GTA San Andreas Multiplayer launcher for Windows 10/11 x64.
Brand: ICARUZ. Credit: Developed by Fajuu.

## Stack
Core: Rust stable, Cargo workspace, Tokio, Serde, tracing.
Desktop: Tauri 2, Windows WebView2.
UI: Svelte 5, TypeScript, Tailwind CSS 4.
Storage: SQLite (rusqlite, bundled) + TOML config.
Network: reqwest/rustls, UDP query.

## Layout
- `src-tauri/` Tauri shell, typed commands, logging, storage
- `src/` Svelte frontend, design tokens from DESIGN.md
- `crates/icaruz-core/` pure Rust logic: detector, vault, clients, query

## Run (Windows dev)
1. Install Rust stable, Node 20+, WebView2 SDK (preinstalled on Win10/11).
2. `npm install` in root.
3. `npm run tauri dev`.

## Run (Linux check, UI only)
`npm install` then `npm run dev`. Rust core checked with `cargo check`.

## Data paths (Windows)
`%LOCALAPPDATA%/Icaruz/data/icaruz.db`
`%LOCALAPPDATA%/Icaruz/config/`
`%LOCALAPPDATA%/Icaruz/logs/`
`%LOCALAPPDATA%/Icaruz/crash-reports/`

## Anti-slop
Method: anti-slop DURING, v3.2.26.
Direction: DESIGN.md. Dials: ENERGY 2 / RHYTHM 2 / MOTION 1.
Design Read: desktop game launcher for players, dark midnight editorial gaming style, dial ENERGY 2 / RHYTHM 2 / MOTION 1.

## Status P3-P6 (honest)

- P3 Servers and Library: partial. Direct connect and validation call real Tauri commands. Saved server list, cache, and offline state are honest placeholders. No dummy rows in production UI.
- P4 Repair and Crash: partial. RepairDialog validates with real `detect_gta` and reports empty, loading, error, success. CrashDialog shows the last error with redacted copy. Log redaction helpers and multi file repair steps are still planned.
- P5 Performance: code rules done, numbers pending. UI split into components, no polling timers, dynamic Tauri import, gaming mode stops animation and repeat checks. Targets are experimental: cold start at most 2s, idle 50 to 80 MB full WebView2 tree. No Windows measurement collected yet. See `docs/PERF.md`.
- P6 Docs and QA: docs added (`docs/PERF.md`, `docs/QA.md`, `docs/PRIVACY.md`). Multi PC checklist (Win10 8GB at 125 percent, Win11 16GB at 150 percent) has not run yet, so P6 is not marked done.
