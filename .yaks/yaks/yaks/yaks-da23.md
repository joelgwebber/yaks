---
id: yaks-da23
title: 'Rebuild yaks sheds from note attribution: one row per actor'
status: shaving
type: task
priority: 2
created: '2026-10-31T15:17:35Z'
updated: '2026-10-31T16:18:34Z'
parent: yaks-039b
labels:
- sheds
- cli
---

Replace the checkout-based model with one built only from the farm, so it reads the same on every machine once synced.

## Model

- A row is an **actor**: the stamp on notes and moves (`▸ <ts> [actor]`).
- **Working on:** yaks `shaving` now whose most recent entry since entering `shaving` is this actor's. The most recent writer, not the mover: a coordinator moves a yak to `shaving` for a worker (`yaks brief`), and the worker writes after.
- **Recent:** other yaks it wrote, newest first, with their status now.
- **Asking:** its yaks with `needs: human`.
- **Last write + liveness:** active / stale / idle, as today (`ACTIVE_WITHIN`).
- **Header:** how fresh remote data is: farm last pulled, ahead/behind `origin/yaks` (reuse `yaks status`).

## Sketch

```text
farm: pulled 4m ago · up to date with origin/yaks

tex-1                               active · 14m ago
  shaving  mem-7aba  Add open-source textures         asking you
walk-1                              active · 9m ago
  shaving  mem-f13a  Add walk mode
delta:Set up remote yak workflow    active · 12m ago
  shaving  mem-f434  Host memcraft-viewer web build on GitHub Pages
  recent   mem-99d0  shorn  Left-button mouse lock
(3 idle actors with no claims; --all lists them)
```

## Scope

- `yaks sheds` text + `--json`, the TUI Sheds tab, and every consumer of `sheds::Shed` (inbox overlay, yak-detail shed section).
- Essentially `yaks log` grouped by actor and joined to current status; test against a fixture farm with interleaved actors.
- Docs and help: `docs/cli.md`, `docs/tui.md`, the `yaks` skill vocabulary ("Shed" is redefined as a worker in the farm, not a checkout), `yaks-coordinating` "Watching workers".

Lost on purpose: branch/HEAD/ahead/dirty/merged, unclaimed-checkout detection, orphan pins. Liveness instead rests on the heartbeat rule (sibling skills yak).

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:15Z [yakherd]

Claim (yakherd). Owner: worker `sheds-1`, together with `yaks-d3ad` as a serial arc (rebuild first, then removal). Owns `src/sheds.rs`, `src/activity.rs`, `src/discover.rs`, `src/tui/sheds_tab.rs`, plus the sheds pieces of the TUI inbox overlay and yak detail, and in `src/main.rs` the `Sheds`/`Discover` commands. Out of scope: note writing, `land`, `commits`, `.agents/skills/`. Evidence: fixture-farm tests (interleaved actors, a coordinator claim followed by a worker's writes, asks), `yaks sheds` text and JSON from the real yaks farm, a TUI Sheds tab frame attached, the full gate. Judge: yakherd at landing.

---
▸ 2026-10-31T16:18:34Z [sheds-1]

Start (sheds-1).

- date -u: Sat Oct 31 16:18:34 UTC 2026
- yaks path yaks-da23: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-da23.md
