---
id: yak-6f9e.13
title: 'Web UI: show reverse deps (Blocks section)'
type: feature
priority: 2
created: '2026-04-19T00:11:28Z'
updated: '2026-04-19T00:15:32Z'
commit: f042bae
parent: yak-6f9e
---

The TUI detail pane shows a 'Blocks:' section listing tasks that depend on the current one. Port that to the web UI detail panel — compute reverse-deps from state.tasks and render as clickable yak links.

---
▸ 2026-04-19T00:15:12Z
Added Blocks section to web UI detail panel showing reverse-deps computed from state.tasks.
