---
id: yaks-f0db
title: 'TUI: auto-sync the farm on an interval'
status: hairy
type: feature
priority: 2
created: '2026-11-01T20:09:16Z'
updated: '2026-11-01T20:09:16Z'
parent: yaks-200f
labels:
- wiki
- tui
---

Decided on yaks-63c3: the TUI syncs the farm on a regular interval (fetch, merge with the driver, push) so upstream changes appear even with no local edits. Interval in config; off for a private farm (it never pushes, but still pulls when it has an upstream).
