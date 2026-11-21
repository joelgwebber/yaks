---
id: yaks-2e7c
title: Take the command table mechanism from toque
status: hairy
type: task
priority: 2
created: '2026-11-21T05:22:42Z'
updated: '2026-11-21T05:22:42Z'
tags:
- tui
---

The generic half of src/tui/keymap.rs (Key, Family, Binding, Keymap lookup and overrides, popup and reference rendering) moved to toque's keys feature (toque farm yak-7658) so canon's TUI can share it. yaks keeps its commands, contexts, table and help text; no key, help text or snapshot changes.
