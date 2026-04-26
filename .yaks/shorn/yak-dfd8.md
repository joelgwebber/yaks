---
id: yak-dfd8
title: Collapsible tree view in tui and web
type: feature
priority: 2
created: '2026-04-22T18:12:42Z'
updated: '2026-04-22T20:09:25Z'
---

'nuff said.

---
▸ 2026-04-22T20:09:25Z
TUI + web: collapsible tree via Space. TUI persists to ~/.cache/yaks/<slug>.json; web persists to localStorage keyed on owner/repo/branch. Chevron '▶ N' on collapsed parents (clickable in web). Filter-active wins: collapse ignored when filters/search active. Cursor snaps to parent on collapse; deep-link expands ancestors.
