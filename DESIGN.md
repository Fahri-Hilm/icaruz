# ICARUZ DESIGN TOKENS
Source: UI/UX DESIGN PLAN v1.0. Direction file for anti-slop R-37.

## Dials
ENERGY 2 / RHYTHM 2 / MOTION 1.
Reason: gaming premium but calm, consistent screens with few breaks, motion limited to hover and state change. No autoplay, no loop, no heavy blur. Gaming Mode disables animation.

## Tokens
bg: #0B0F17 (page)
surface: #151C29 (card)
elevated: #1D2635 (raised)
border: #2B3546 (1px hairline)
text: #F3F6FC (contrast 17.72 on bg, PASS)
muted: #94A3B8 (contrast 7.48 on bg, PASS)
accent: #4F8CFF (contrast 5.96 on bg, PASS, key moment only)
accent-secondary: #965BFF (limited use)
success: #43C995

Contrast verified with antislop-human contrast-check.py.

## Rules
- One primary CTA per context. Play is priority on Home.
- No glow everywhere, no gradient everywhere, no glass on all surfaces.
- Accent at key moment only: Play button, active nav, focus ring.
- Icon must be relevant. No sparkle, magic, robot default.
- Nav: Home, Servers, Library, Settings. Each has real destination.
- States required: empty, loading, error, success for every data view.
- Keyboard: Tab order follows visual order, Enter/Space activate, Escape closes dialog.
- Motion: hover 120-160ms, page 150-220ms, transform/opacity only. Respect prefers-reduced-motion.
- Type: Inter or Segoe UI. 8px grid, radius 10-14px, minimal shadow.

## Screens
01 logo + app icon set. 02 splash brief. 03 first-run. 04 Home READY. 05 Home NEEDS ATTENTION.
06 servers + detail. 07 library. 08 repair dialog. 09 settings + About (Developed by Fajuu).
10 empty, loading, offline, error, success states.
