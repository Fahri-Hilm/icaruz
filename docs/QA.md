# ICARUZ QA Checklist (P6)

Manual pass before any user build. Check each box with hardware noted. Skip nothing silently.

## 1. First run, no tutorial

- Fresh profile (`%LOCALAPPDATA%/Icaruz` renamed or empty).
- App opens to Home with a clear headline and one primary action: Play.
- Empty states explain what to do next (set GTA folder, type a trusted address).
- No tutorial overlay, no coach marks, no forced tour.

## 2. Play is the clearest action

- Home has exactly one primary CTA: Play.
- Servers has exactly one primary CTA: Join and Play.
- Library has exactly one primary CTA: Validate.
- Settings has exactly one primary CTA: Save nickname.
- Dialogs have exactly one primary CTA each (Repair: Validate now. Crash: Open repair).
- Primary button uses accent `#4F8CFF` on dark `#0B0F17` only at these key moments.

## 3. No dummy servers in production

- Servers view shows an honest empty saved list, never fake rows.
- Direct connect only sends what the user typed.
- Sample addresses like `127.0.0.1:7777` appear only as placeholder text, never as a saved row.
- Fail states keep the typed address for retry and change nothing on disk.

## 4. Every data view covers four states

For Home quick play, Servers direct connect, Library validation, Settings save, Repair dialog:

- Empty: what is missing and what to do.
- Loading: what is running now.
- Error: what failed, with files left untouched.
- Success: what worked, with next step.

## 5. Keyboard and focus

- Tab order follows visual order on every screen.
- Enter and Space activate buttons and the gaming mode checkbox.
- Escape closes Repair and Crash dialogs.
- Focus ring is visible on all interactive elements (2px accent outline).
- Focus starts on the primary or first action when a dialog opens.
- Closing a dialog returns focus to the button that opened it (verify manually).

## 6. Contrast

Tokens locked from DESIGN.md:

- Text `#F3F6FC` on bg `#0B0F17`.
- Muted `#94A3B8` on bg `#0B0F17`.
- Accent `#4F8CFF` on bg `#0B0F17` for key actions only.

Verify with a contrast tool after any CSS change. Fail the build note if any token drifts.

## 7. Reduced motion

- OS reduced motion on: no transitions, no animation.
- Gaming mode on: same freeze, plus no repeat status checks.
- Check hover states still read as static color change only.

## 8. Scaling 125 and 150 percent

On Windows Display settings:

- 125 percent: nav plus main fit without horizontal scroll at 1366x768.
- 150 percent: same at 1920x1080, mobile single column rule may apply.
- Inputs keep 40px minimum touch height.
- Dialogs stay centered with padding, never clipped.

## 9. Multi PC checklist

Run the full list on at least:

- PC A: Windows 10 x64, 8 GB RAM, 125 percent scaling.
- PC B: Windows 11 x64, 16 GB RAM, 150 percent scaling.
- Optional PC C: 4 GB RAM low tier for the PERF matrix.

Log per PC: OS build, RAM, GPU, scaling, WebView2 runtime version, cold start time, idle working set sum, pass or fail per section.

## Current honest state

Checklist is written and UI hooks exist (dialogs, focus rings, empty states, gaming mode). Full multi PC run has not been done in this Linux pass. Do not mark P6 as done until PC A and PC B logs exist.
