---
id: yak-86f4
title: Reorganize help panel (and simplify keymaps)
type: task
priority: 1
created: '2026-04-15T00:47:25Z'
updated: '2026-04-15T14:07:27Z'
depends_on:
- yak-5f32
- yak-a8e1
- yak-6cb6
- yak-62c0
- yak-ed9c
- yak-3990
commit: 1823fd7
---

```
Movement
  j ↓ • k ↑             Move cursor
  d • u                 Half-page down • up
  PgDn • PgUp           Full-page down • up
  g • G                 Top • bottom
  Esc                   Clear search / back

Search
  /                     Search tasks / text
  n • N                 Next • prev match
  y                     Copy yak ID to clipboard

Editing
  e                     Edit task in $EDITOR     -- Make these capital to emphasize that
  m                     Add comment              -- they're mutative
  A                     Attach artifact
  D                     Delete task              -- Change to X
  b • B                 Add • remove dependency  -- Replace with B as in [[yak-69f6]]
  P • T • S • L         Change priority • type • state • labels  -- N (Title) -> S (State)

List pane
  [ ShTab • Tab ]       Previous • next tab
  l → Enter             Show detail pane
  c • C                 New root • child task
  s • x • r             Shave • shorn • regrow  -- Remove these in favor of S (change state)
  n • t • a             Next • tangled • all    -- Remove these in favor of the filter UI

Detail pane
  h ←                   Hide detail pane
  [ ShTab • Tab ]       Prev • next link
  Enter                 Follow link / open artifact
  O                     Open artifact externally
  K • J                 Prev • next task in list
  o • i                 Nav back • forward in stack

Filter pane
  -- TODO: Think through filter design first

General
  R                     Refresh  -- Change to re(F)resh
  ?                     Toggle this help
  q                     Quit
```
