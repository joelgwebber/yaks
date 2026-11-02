---
id: yaks-da23
title: 'Rebuild yaks sheds from note attribution: one row per actor'
status: shorn
type: task
priority: 2
created: '2026-10-31T15:17:35Z'
updated: '2026-10-31T17:24:18Z'
parent: yaks-039b
tags:
- sheds
- cli
verify: GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace
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

---
▸ 2026-10-31T16:38:39Z [sheds-1]

Progress (sheds-1): milestone 1 of 3.

- New `src/sheds.rs` (the model: `build`, `collect`, `render`, `to_json`, `ago`) over `store::parse_notes`; the old checkout-discovery module is `git mv`ed to `src/gitrepo.rs` (pruned in the second commit).
- `yaks sheds [--json] [--all]` is on the new model; the TUI Sheds tab is reworked on it; the 3 insta snapshots it alters are updated.
- `cargo test --workspace` green apart from docs left to do. Next: docs/help parity, docshots, real-farm output, then `yaks-d3ad`.

[sheds-real-all](yaks-da23/sheds-real-all.txt)

[sheds-real](yaks-da23/sheds-real.json)

[sheds-real](yaks-da23/sheds-real.txt)

[sheds-tab-detail](yaks-da23/sheds-tab-detail.txt)

[sheds-tab-list](yaks-da23/sheds-tab-list.txt)

---
▸ 2026-10-31T16:40:12Z [sheds-1]

Decision: a claim nobody has written to is the mover's, unless the move carries `for:`.

- Rule: claimant = `for:` of the latest move into `shaving`, else the actor of the most recent note since that move (the move itself included).
- Rejected: "no claimant until a worker writes". It would hide a yak a human or solo agent just moved to shaving.

---
▸ 2026-10-31T16:40:12Z [sheds-1]

Decision: a `for:` claimant with no writes yet is stamped with the move's timestamp.

- The `for:` actor gets a row, `last` = the move's ts, so it reads `active`, and the yak is listed under it, not the coordinator.
- Rejected: no row until the worker writes (a freshly briefed worker would be invisible, which is the exact moment the coordinator wants to see it).

---
▸ 2026-10-31T16:40:13Z [sheds-1]

Decision: an actor's own yak with `needs: human` appears once, as `asking` unless it holds the claim.

- Claimed and asking: one `shaving` line ending `asking you`, as in the sketch. Unclaimed and asking: an `asking` line. Never both.
- "Asking" means any actor who wrote the yak, not only the asker; `needs: human` is a property of the yak, and this keeps the rule "the actor's yaks with needs: human".
- Rejected: restricting to the writer of the `asked: needs` note (needs a new parse of ask notes for little gain).

---
▸ 2026-10-31T16:40:14Z [sheds-1]

Decision: text hides only idle actors with no claim AND no open ask; `--json` and the TUI list everyone.

- Brief: "text, --json and the TUI show the same data" and "idle actors only as a count by default, with --all". Reconciled: the count is a text-table default only; JSON always carries every actor (a script wants all); the TUI tab is a scrollable list, so it shows all, idle dimmed (docs say so).
- The tab count is the actors that are not `quiet()` (open ask, active or holding a claim).

---
▸ 2026-10-31T16:40:14Z [sheds-1]

Decision: TUI Sheds tab keeps its shape (list of sheds, then a shed's yaks, Enter opens a yak) and drops what the new data makes meaningless.

- Dropped: the `*` current-checkout mark, ahead/behind, dirty, `state:` (merged/uncommitted), path, git line, the orphaned-pins footer, `title`/slug naming, and `y` copying a path (now copies the actor name).
- Added: the farm freshness line as the list's footer (where the orphan line was), and `shaving`/`asking`/`recent` groups in the detail.
- `Snapshot::scan` no longer needs a cwd; `ShedsTab::scan: bool` replaces `cwd` as the "may scan" switch (`cwd` stays for `source_base`).
- Rejected: a separate `asking` section ordered before `shaving`; the CLI order (claimed, then asks, then recent) is the same data in the same order.

---
▸ 2026-10-31T16:40:15Z [sheds-1]

Decision: `ago` is a small pure function in `sheds.rs` (`14m ago`, `3h ago`, `2d ago`), not `tui::when`.

- `tui::when` formats absolute dates past a day ("yesterday 14:05 UTC") and is TUI-internal; the sketch and the CLI need a plain relative age that the TUI shares verbatim.
- Rejected: moving `when.rs` out of `tui/` (touches other owners' files).

---
▸ 2026-10-31T16:40:16Z [sheds-1]

Decision: the header uses `status::sync_state` as is, and is omitted unless the farm has an upstream.

- Text `farm: up to date with origin/yaks` / `farm: N ahead, M behind origin/yaks (as of the last fetch)`; JSON `farm` is the `SyncState` fields or `null`.
- No "pulled 4m ago" from the sketch: git records no pull time without reading reflogs, and the brief says reuse `yaks status`. Rejected: parsing `FETCH_HEAD`'s mtime (a per-machine file, breaks "same on every machine").

---
▸ 2026-10-31T16:40:17Z [sheds-1]

Decision: the first commit renames `src/sheds.rs` to `src/gitrepo.rs` as a whole and a new `src/sheds.rs` is written beside it; pruning gitrepo.rs and the other deletions are the second commit.

- Keeps the first commit buildable and gated (the old discovery code is still in `gitrepo.rs`, behind `#[allow(dead_code)]` on its `mod`, which the removal commit drops) without carrying the removal in the rebuild.
- Rejected: pruning in commit one (then yaks-da23's commit would contain yaks-d3ad's work, against the brief's order).

---
▸ 2026-10-31T16:41:36Z [sheds-1]

verify: `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace` -> FAIL (exit 101)

---
▸ 2026-10-31T16:43:35Z [sheds-1]

verify: `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-31T16:43:45Z [sheds-1]

Summary (sheds-1): `yaks sheds` is rebuilt from note attribution, one row per actor.

- `src/sheds.rs` (new): `build` over `store::parse_notes` (claimant rule, recent, asking, liveness, ordering), `collect` (+ `status::sync_state` for the farm header), `render(--all)`, `to_json`, `ago`; 12 unit tests on fixture farms with interleaved actors, a coordinator claim then worker writes, `for:`, a re-claim, asks, ordering, cap, JSON.
- `src/main.rs`: `Sheds { json, all }` and its `--help`; the old checkout-discovery module is `git mv`ed to `src/gitrepo.rs` (callers in `store`, `branchfarm`, `init`, `privatefarm`, `tui/source`, `discover` updated); pruning it is yaks-d3ad.
- TUI: `src/tui/sheds_tab.rs` reworked (list of actors, detail = shaving/asking/recent, Enter opens the yak, farm freshness footer); help overlay + hint text; 3 insta snapshots updated (sheds_tab_list/detail, help overlay), each reviewed.
- Tests: `tests/sheds.rs` replaces `shed_activity.rs` and `shed_title.rs` (which tested the activity log and `yaks.shed.title`); `tests/dash_c.rs` sheds test now seeds a note.
- Docs: `docs/cli.md`, `docs/tui.md`, `README.md`, the two `docs/assets/tui-sheds*.svg` regenerated (the other docshots SVGs were reverted: the renderer's output drifted, unrelated).
- Evidence: `yaks verify` PASS (full workspace gate with the fsmonitor env); real-farm `yaks sheds` text, `--all`, `--json` and the TUI frames are attached.

---
▸ 2026-10-31T16:43:46Z [sheds-1]

moved: shaving -> shorn

---
▸ 2026-10-31T16:44:01Z [sheds-1]

Committed a60d67c (rebuild).

---
▸ 2026-10-31T16:50:04Z [delta:Cloud Runners]

kind: landed
commit: 7393e452d215d409cebfb1c721d62506e3bd6b54
source: a60d67cf1ad2a15316b9202da8fff4c06966b25e
branch: main
---

landed: 7393e45 yaks-da23: rebuild yaks sheds from note attribution, one row per actor

---
▸ 2026-10-31T17:24:18Z [yakherd]

kind: landed
commit: 088275173b8a23948526668392647c629b1cfa66
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): 0882751 yaks-da23: rebuild yaks sheds from note attribution, one row per actor (was `7393e45`)
