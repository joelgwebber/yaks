---
id: yaks-fbb0
title: 'TUI phase 2: a command table for keys; help and hint generated from it'
status: hairy
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:46:56Z'
parent: yaks-947a
tags:
- tui
---

Phase 2 of [the design](/wiki/design/tui-model.md#5-keys) (part of yaks-0ac1): every key binding moves into one command table (named commands, a context, default keys, a help line), dispatched in layers, with the `?` overlay and the bottom hint generated from it. A refactor: no key changes meaning.
