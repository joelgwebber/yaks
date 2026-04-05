---
id: yak-5aec
title: 'TUI: simplify detail-pane nav bindings'
type: task
priority: 2
created: '2026-04-05T15:26:30Z'
updated: '2026-04-05T15:27:17Z'
commit: 7515b45
---

Now that nav uses plain i/o (no literal Ctrl-I needed), Tab is free again. Restore Tab/Shift-Tab as link cycler, drop ]/[ aliases in detail, keep i/o + Enter/Backspace for nav.
