---
id: yak-bf54.9
title: TUI interactive sidecar review (accept/reject per item)
type: feature
priority: 3
created: '2026-04-25T18:25:28Z'
updated: '2026-04-25T18:25:28Z'
---

v1 of bf54.4 ships only a list-view marker (~ before the ID). Reviewing a sidecar still requires 'yak sync show id' + a text editor.

This yak adds an interactive TUI flow: a new key opens a sidecar review dialog when the cursor is on a yak with a pending sidecar. The dialog shows one row per item from the sidecar (fields, comments_up, comments_down, attachments_up, attachments_down) with direction, current resolution, and a brief preview. Hotkeys toggle resolution states (approve/skip), with separate hotkeys to trigger apply (shells out to /yaks:sync) and discard (yak sync clear).

Out of scope: rich field-level editing (merging title strings character-by-character). Power users edit the YAML directly. The TUI just toggles resolution states.
