---
id: yaks-fd7a
title: attach writes links with unencoded spaces, which markdown does not parse
status: hairy
type: bug
priority: 3
created: '2026-11-01T21:28:56Z'
updated: '2026-11-01T21:28:56Z'
parent: yaks-f301
labels:
- wiki
- markdown
---

`yaks attach` writes `![name](<id>/my shot.png)` with the file name unencoded. CommonMark does not parse a destination with a space, so the TUI and GitHub show it as text, and link tools (`refs`, `doctor`, `promote`) miss it (`promote` handles the literal form as a special case).

Fix: write `<id>/my%20shot.png` (or `](<<id>/my shot.png>)`), and have `yaks migrate` or `doctor --fix` re-encode existing ones. Found in yaks-2c42.
