---
id: yak-bf54.2
title: 'Sync check mode: identify yaks that likely need syncing'
type: feature
priority: 2
created: '2026-04-24T03:01:50Z'
updated: '2026-04-25T20:02:36Z'
depends_on:
- yak-bf54.1
- yak-bf54.3
- yak-bf54.4
---

A cheap 'which of my yaks with sources might have drifted?' query, so the user can pick what to /yaks:sync. Depends on bf54.1 — without reliable upstream timestamps, the only honest answer is 'all of them,' which isn't useful. When that investigation lands, revisit scope (CLI flag on list? new /yaks:sync-check command?).

---
▸ 2026-04-25T15:47:50Z
Updated dependency graph: now waits on bf54.3 (last_synced field — provides the predicate for cheap drift detection) and bf54.4 (pending-sync review pipeline — sweep produces a sidecar per drifted yak so the user can review in TUI before any writes). With those two in place, this becomes essentially: enumerate yaks with source, group by tracker, batch JQL by last_synced, write sidecars for drifted ones, surface in TUI.

---
▸ 2026-04-25T20:02:36Z
Done. yaklib/sync.py: tracker_for, upstream_key_for, iter_synced_yaks helpers. New CLI: 'yak sync check [--json] [--tracker X]' enumerates source-linked yaks with classified tracker, key, last_synced, local_drift, pending sidecar flag. New /yaks:sync-check slash command + skill 'Sweep / drift check' section drives the upstream-side drift query (single batched JQL per Jira instance with 'issuekey IN (...) AND updated > <oldest_last_synced>'). Live test against /tmp/yaks-sync-test (50 SUBTEXT yaks): 50 enumerated, JQL returned 50 updated stamps, drift classification ran cleanly — all 50 reported in-sync (correct: nothing has moved upstream since bootstrap). Demo-rolling jira-301's last_synced back to 2020 surfaced 'both'-kind drift as expected. 10 new tests; 119/119 pass.
