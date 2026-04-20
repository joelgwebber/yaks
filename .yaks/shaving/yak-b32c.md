---
id: yak-b32c
title: Update UI with new design
type: task
priority: 2
created: '2026-04-20T18:29:45Z'
updated: '2026-04-20T19:27:58Z'
---

Fetch this design file, read its readme, and implement the relevant aspects of the design. https://api.anthropic.com/v1/design/h/HHmb7rqARpu1XbgnEZbuUg?open_file=Yaks.html

Implement: Yaks.html atop the existing codebase. Mostly the same functionality, except for a
filter window we'd like to try out.

### 2026-04-20T18:54:31Z
Started implementation. Plan: rewrite docs/index.html to adopt the Claude Design visual system (Inter Tight + JetBrains Mono, warm paper palette, oklch accents, slide-in panel, meta-grid). Keep current vanilla JS (no React), drop pico.css, drop tweaks/accent/tree/density variants. New: filter dialog (type/priority/labels/search/parent/ready-tangled). Keep all existing functionality (GH tree API, raw CDN, hash routing, Blocks, artifact paths, yak-linking, branch picker, refresh).

![Hairy tab: warm paper palette, emoji tab icons, monospace IDs, tinted badges](artifacts/yak-b32c/hairy.png)

![Shorn yak-6f9e detail: eyebrow status, meta-grid, children numeric-sorted](artifacts/yak-b32c/detail.png)
