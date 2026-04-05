---
id: yak-76be
title: 'TUI: show reverse deps ("Blocks") in detail pane'
type: task
priority: 2
created: '2026-04-05T14:34:31Z'
updated: '2026-04-05T14:42:34Z'
commit: 362a36e
---

Forward deps (this task's depends_on) already render as navigable links. Add a 'Blocks:' section listing tasks whose depends_on includes this one — navigable via Tab/Enter like parents/children. Useful when shearing a task to see what downstream work it unblocks. Requires a reverse-lookup over all tasks; cache per reload is fine.
