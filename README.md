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
