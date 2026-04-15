---
id: yak-8cef
title: 'Consolidate dep keys: B for add/remove, context-aware'
type: task
priority: 1
created: '2026-04-15T04:44:49Z'
updated: '2026-04-15T04:47:28Z'
commit: 60adc1a
---

Closes yak-69f6 and yak-e8a2.

- Remove lowercase b binding (accidental-press hazard; list view can't disambiguate which dep anyway).
- B replaces the current b/B pair:
  - List pane: fuzzy-pick a new dep for the current task.
  - Detail pane, cursor on a 'Depends on:' link: remove that dep.
  - Detail pane, cursor elsewhere: fuzzy-pick a new dep for the task being viewed.
- Drop the digit-indexed remove_dependency picker — detail cursor is the selection UI.
- DetailLine gets a way to mark 'this link is a dep' so B can detect it (tag dep rows with a distinct kind, or carry a flag). Parent/Children/Blocks links stay inert under B.

Closes both [[yak-69f6]] and [[yak-e8a2]].
