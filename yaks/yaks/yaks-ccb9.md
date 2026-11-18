---
id: yaks-ccb9
title: 'Commits phase 3: a derived Code: row in the yak detail pane'
status: hairy
type: feature
priority: 2
created: '2026-11-18T03:02:46Z'
updated: '2026-11-18T03:03:14Z'
parent: yaks-3515
depends_on:
- yaks-3f90
tags:
- tui
- git
---

Phase 3 of yaks-3515; needs phase 2.

A `Code:` row in the detail pane, built on read and never stored. It shows when a yak's notes hold commit links, `commit:` fields or checkpoint `branch:` fields, for example `Code: 2 landed, 1 branch`. Enter opens the list, newest first; each entry opens its commit view or names its branch. This is the TUI side of `yaks commits <id>`, so share its collection code rather than copying it.

#### 2026-11-18T03:03:14Z

*sources-triage*

Phase 3 depends on phase 2 (yaks-3f90): the rows open commit views.
