---
id: yak-bf54.2
title: 'Sync check mode: identify yaks that likely need syncing'
type: feature
priority: 2
created: '2026-04-24T03:01:50Z'
updated: '2026-04-25T15:47:50Z'
depends_on:
- yak-bf54.1
- yak-bf54.3
- yak-bf54.4
---

A cheap 'which of my yaks with sources might have drifted?' query, so the user can pick what to /yaks:sync. Depends on bf54.1 — without reliable upstream timestamps, the only honest answer is 'all of them,' which isn't useful. When that investigation lands, revisit scope (CLI flag on list? new /yaks:sync-check command?).

### 2026-04-25T15:47:50Z
Updated dependency graph: now waits on bf54.3 (last_synced field — provides the predicate for cheap drift detection) and bf54.4 (pending-sync review pipeline — sweep produces a sidecar per drifted yak so the user can review in TUI before any writes). With those two in place, this becomes essentially: enumerate yaks with source, group by tracker, batch JQL by last_synced, write sidecars for drifted ones, surface in TUI.
