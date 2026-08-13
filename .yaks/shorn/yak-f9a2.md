---
id: yak-f9a2
title: 'Embedded TUI demo: cast-builder + asciinema-player page'
type: feature
priority: 2
created: '2026-07-26T01:31:18Z'
updated: '2026-07-26T01:37:59Z'
---

Show what yaks is FOR via an animated, lightly-interactive two-pane terminal demo embedded on the GitHub Pages site (agent pane + yaks TUI board, evolving in lockstep). Approach: a Python 'director' in ./demo that emits an asciicast v2 .cast deterministically (no tmux/asciinema recording), pulling constants/helpers from scripts/yaklib + scripts/yaktui to resist implementation drift. Player (asciinema-player) vendored locally under docs/. This yak covers the scaffolding: cast-builder helper (castkit), yaks-flavored two-pane renderer, a short placeholder screenplay, docs/demo.html wired to the vendored player + themed to match docs/index.html. Full screenplay + README wiring are follow-ups.

---
▸ 2026-07-26T01:37:52Z
Scaffolding done + verified. Added ./demo (build tool, not shipped): castkit.py (dep-free asciicast v2 builder + wide-char-aware virtual Screen — handles emoji so the two panes stay column-aligned), yakscreen.py (two-pane compose: agent transcript | divider | yaks board; imports status_emoji + status constants from scripts/yaklib to resist drift, mirrors render.py row/tab layout + colors.py palette as ANSI), build_demo.py (Director over Cast+Board; writes docs/demo.cast). Placeholder screenplay: ask -> shave -> shorn, 3 chapter markers, ~10s, 82KB. Vendored asciinema-player 3.6.3 under docs/vendor/. docs/demo.html: themed to match docs/index.html, autoplay+loop, links back to board/GitHub/PyPI. Verified: cast is valid asciicast v2; final frame composes with aligned divider across the emoji tab row; local http.server serves all assets 200; full pytest suite green (117). No version bump (docs/ + demo/ aren't in the shipped PyPI payload; README wiring is a follow-up that will bump).
