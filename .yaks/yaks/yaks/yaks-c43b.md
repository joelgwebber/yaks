---
id: yaks-c43b
title: 'TUI: mark the yaks yaks status lists as dirty (uncommitted)'
status: hairy
type: feature
priority: 3
created: '2026-10-21T21:59:33Z'
updated: '2026-10-21T21:59:33Z'
labels:
- tui
- git
---

Follow-up to yaks-ce5d (Joel, 2026-10-21). The tab-row chip says `● N uncommitted` but not WHICH yaks; today you have to drop to `yaks status` to see them.

Simple affordance: a small dirty marker on each list row whose yak `yaks status` would list (created, moved, notes, edited, artifacts), plus a `Dirty:` line in the detail pane naming the change kinds (`moved hairy -> shaving, notes +2`, the same tokens as `yaks status`). Optionally a filter or view for "uncommitted" so `W` can be checked first.

Notes for whoever builds it:
- The data is already computed for the chip: `farmsync::FarmSync::read` calls `changes::read` + `changes::classify`, whose `yaks` map is keyed by id. Keep those ids on `FarmSync` rather than re-reading git per row.
- Same refresh points as the chip (load, reload, after W/U); local git only, never a fetch.
- Pick a marker that does not collide with the existing row badges (shed `🛖`, needs `⏳`/`✅`, star); document it in the `?` overlay and docs/tui.md.
- Evidence: an insta snapshot with one dirty and one clean row, and a headless frame on a real dirty farm.
