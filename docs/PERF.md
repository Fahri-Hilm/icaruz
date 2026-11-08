# ICARUZ Performance Notes (P5)

Status: experimental targets, not certified results. Measure on real Windows hardware before release claims.

## Targets

- Cold start: at most 2s from click to interactive Home on a mid range SSD PC.
- Idle RAM: 50 to 80 MB for the full app tree when idle on Home with no game running.
- Idle CPU: near 0 percent when the window is visible and idle, 0 when minimized or in Gaming mode.
- Game running: UI must not add load. Polling and animation stop when the game runs or Gaming mode is on.

## What counts as total

The full WebView2 tree means the sum of:

1. `ICARUZ.exe` UI process.
2. All child `msedgewebview2.exe` processes for this app (renderer, GPU, utility).
3. No game process. `gta_sa.exe` and SA-MP or open.mp processes are excluded.

Measure the tree, not only the parent PID. WebView2 splits work across child processes, so a single PID number will mislead.

## How to measure

### Working set (RAM)

1. Close other apps. Open Task Manager, Details tab, add column `Working set (memory)`.
2. Start ICARUZ, wait 60 seconds on Home without input.
3. Sum working set for `ICARUZ.exe` plus every `msedgewebview2.exe` owned by ICARUZ.
4. Record three samples 30 seconds apart. Report median and max.

PowerShell alternative for one snapshot:

```powershell
Get-Process icaruz, msedgewebview2 -ErrorAction SilentlyContinue |
  Measure-Object WorkingSet64 -Sum
```

Divide the sum by 1MB for the MB value.

### CPU idle

1. Use Task Manager or Performance Monitor, 60 second window.
2. Stay on Home, no mouse or keyboard input.
3. Pass if average stays below 1 percent and no repeating spikes appear.

### Cold start

1. Reboot or use `taskkill` to stop all ICARUZ and related WebView2 processes.
2. Start with a stopwatch or PowerShell `Measure-Command { Start-Process icaruz.exe }` plus manual check of first interactive paint.
3. Pass if Home accepts input in at most 2s on SSD hardware. HDD hardware is noted separately, not counted as fail for the SSD target.

## RAM matrix

Run the idle working set procedure on each tier if hardware is available:

| Tier | RAM | Expectation |
| ---- | --- | ----------- |
| Low | 4 GB | App idle stays in target range, no swap growth over 10 minutes |
| Base | 8 GB | Target range, cold start at most 2s |
| High | 16 GB | Same as base, extra headroom only, no new background work allowed |

If 4 GB hardware is missing, say so in the test log. Do not extrapolate.

## Frontend rules that protect the budget

- One Tauri `get_status` call on mount. No `setInterval` polling in `App.svelte`.
- Tauri API is loaded with dynamic `import()` only when a command runs.
- Components are split (`Home`, `Servers`, `Library`, `Settings`, `RepairDialog`, `CrashDialog`) so future routes can lazy load.
- CSS animates `opacity`, `background-color`, and `border-color` only, 120 to 160 ms. No blur loops, no autoplay, no infinite motion.
- `prefers-reduced-motion` disables all transitions and animation.
- Gaming mode adds `body.gaming` plus `data-gaming="on"`, which disables transitions and animation and skips repeat status checks while active.
- No dummy server rows, no fake ping timers, no background refresh threads in the UI.

## Current honest state

- Code follows the rules above. Timer based polling: none in `src/`.
- Real measured numbers on Windows hardware: not collected yet in this Linux UI pass.
- Next step: run this doc on two Windows PCs (8 GB and 16 GB), record the table, then keep or revise the targets.
