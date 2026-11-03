---
id: yaks-c354
title: 'TUI phase 3b: Sheds becomes a tree view (actor roots, touched yaks as children)'
status: shaving
type: task
priority: 2
created: '2026-11-03T04:10:57Z'
updated: '2026-11-03T04:42:21Z'
parent: yaks-5d08
depends_on:
- yaks-be65
tags:
- tui
---

Phase 3, second half, of [the design](/wiki/design/tui-model.md#1-rows-and-views); implements yaks-f1e6: the Sheds tab becomes an ordinary tree view. Each actor is a collapsible root row; its children are the yaks it touched (working, asking, then recent, as `Shed::rows` orders them). The tab's own cursors, key handlers and list renderer go.

---
▸ 2026-11-03T04:24:11Z [herd-tui]

Claim (yakherd herd-tui): worker `sheds-2` (Sonnet 5.5), from main at `50af0a9a` (phase 3a landed: `tree::Item`, `RowKind`, kind-aware `lookup`).

**Task.** Sheds becomes a tree view of the list, like Wiki became a flat one in 3a.

**Rows.** `tree::Item` gains `Shed(&Shed)` (and `RowKind::Shed`); `memo::Loc` gains `Shed(usize)` (index into the snapshot's sheds). `build_rows` on the Sheds view:

- one root row per shed, in today's order (attention first);
- under it, its touched yaks as `Item::Yak` children (depth 1, in `Shed::rows()` order: working, asking, recent);
- a yak no longer in the farm is skipped.

Roots are collapsible with the existing `collapsed` set and `Space`, keyed by a key that cannot collide with a yak id (e.g. `shed:<name>`). Default: expanded. `/` (the live `filter.search`) keeps a shed whose name or any child matches, and only the matching children.

**Rendering.** `render_list` draws a shed root with today's shed-row text (actor, liveness, last write, what it is shaving); reuse `render_sheds_list`'s line builder. Children draw as normal yak rows, with today's Kind label (`shaving`/`asking`/`recent`) dim at the right if it fits. The footer (farm freshness) and the tab-row `scanned 4s ago` label stay.

**Pane.**

- With focus on the pane, a **shed** row shows today's shed detail (checkpoint branch, its yaks); drop the yak-line cursor inside it.
- A **yak** row shows the normal yak detail pane, so `J`/`K` step through the tree and the shed context stays on the left.
- `Ctx::Sheds` and `Ctx::ShedDetail` go: the Sheds view is `Ctx::List`/`Ctx::Detail`, with shed-row bindings by kind:
  - `l`/`Enter` open the pane;
  - `y` copies the shed name;
  - `r` rescans.
- Yak rows get every yak key, `U`/`W` included (yaks-f1e6's complaint).

**Delete.** `ShedsTab.cursor`, `.line`, `.list_offset`, `.offset`; `move_shed`, `move_shed_yak`, `selected_shed_yak`, `open_shed_yak`, `render_sheds_list`; the shed-detail yak cursor. Keep the snapshot, the scan plumbing, `busy()` and the count.

**Out of scope.** The viewer overlay (phase 4), search semantics beyond the rule above (phase 6a), any key change other than the ones listed. Anything else: `yaks ask yaks-c354`.

**Evidence.**

- The gate: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`, fresh `TMPDIR`.
- Sheds snapshots (`sheds_tab` insta, and the `tui-sheds*` docshots in `docs/assets`) WILL change, since the layout is new. Regenerate the docshots (`cargo test -p yaks docshots -- --ignored`) and record each changed snapshot in one `Decision:` note. No non-Sheds snapshot may change.
- New tests:
  - `j` from a shed root lands on its first yak;
  - `Space` collapses a shed;
  - `Enter` on a yak child shows the yak detail, and `J` moves to the next row;
  - `U` on the Sheds view requests a sync;
  - `/` keeps only matching children.
- Docs and help parity: `docs/tui.md`'s Sheds tab section and key table are rewritten for the tree; `?` and the hint are generated.
- Attach a headless frame of the Sheds tree with a yak child selected and its detail open (the fixture's `snapshot()` in `sheds_tab.rs` tests has sheds; a scratch farm has none).

**Judge.** herd-tui at landing.

---
▸ 2026-11-03T04:27:48Z [sheds-2]

Start: Tue Nov  3 04:27:48 UTC 2026; yaks path: /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-c354.md; top-level /home/user/yaks-5; 50af0a9 yaks-be65: row kinds (yak, document); the Wiki becomes a flat view of document rows

[sheds-tree-headless](yaks-c354/sheds-tree-headless.txt)

[sheds-tree-yak-detail-fixture](yaks-c354/sheds-tree-yak-detail-fixture.txt)

![tui-sheds-yak](yaks-c354/tui-sheds-yak.svg)

---
▸ 2026-11-03T04:40:13Z [sheds-2]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)

---
▸ 2026-11-03T04:40:44Z [sheds-2]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T04:41:56Z [sheds-2]

Progress: implemented, gate green (`env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`: 30 suites ok, 882 bin tests).

- `tree::Item::Shed`, `RowKind::Shed`, `Row::collapse_key()` (yak id, or `shed:<name>`), `memo::Loc::Shed`; `App::shed_rows()` builds the view; `apply_collapse` keys by `collapse_key`.
- `render_list` draws shed roots (`shed_item`, today's text plus a `▶ N` badge when collapsed), yak children with a dim `shaving`/`asking`/`recent` tag, the farm footer under the rows, and `sheds_notice` for no scan / failure / empty / no match.
- Gone: `Ctx::Sheds`, `Ctx::ShedDetail`, `ShedsTab.cursor/line/list_offset/offset`, `move_shed*`, `selected_shed_yak`, `open_shed(_yak)`, `copy_shed_thing`, `render_sheds_list`, the detail's yak cursor.
- Headless on my build, scratch farm with two actors: tab to Sheds, `j` lands on alice's first yak, Enter shows the yak detail, `J` steps to the next yak, `J` again to bob's shed row and its pane (attached `sheds-tree-headless.txt`).
- An earlier `yaks verify` FAIL in this thread was my own mistake (I exported `YAKS_DIR`/`YAKS_ACTOR` into the gate); the PASS note below is the correct run.

---
▸ 2026-11-03T04:41:56Z [sheds-2]

Decision: snapshots changed, all Sheds, nothing else.
- `sheds_tab_list` (insta): now the tree, from a fixture farm that has the fixture's yaks (`shed_farm()` in `sheds_tab.rs` tests; the old `sample()` farm has none of the `mem-*` yaks so no child could show), 120x22 (was 120x14, the tree is taller). Its hint is the shed-row hint, and the tab strip counts differ only because the farm differs.
- `sheds_tab_detail` (insta): same walk-1 pane; only the hint changed, to `h:list  y:copy name  r:rescan  q:quit`, and the strip counts (new farm).
- `sheds_tab_yak_detail` (insta, new): `J` from walk-1's pane to its first yak, the normal yak pane.
- `docs/assets/tui-sheds.svg`, `tui-sheds-detail.svg` changed; `tui-sheds-yak.svg` is new. `docshots.rs` renames the fixture's yaks to this docshots farm's (`sheds_of_this_farm()`), because the farm's yaks are `yaks-*` and the shed rows need yaks below them.
- No non-Sheds snapshot changed: `git status` after the full gate lists only the three sheds `.snap` files. I regenerated every docshot on this tree and on a clean `git worktree` of HEAD: only `tui-sheds*.svg` differ between the two. The committed non-Sheds SVGs differ from either regeneration (toque's SVG now writes `fill-opacity`, `xml:space`), so I `git checkout`ed them rather than commit unrelated churn. The two regenerated Sheds SVGs therefore use the newer toque style while their neighbours keep the older one.

---
▸ 2026-11-03T04:41:56Z [sheds-2]

Decision: a shed's pane keys are `Ctx::Detail` bindings by kind, not a new surface. `surface_kind()` now also names the row kind in the pane; the yak-only pane bindings (line cursor, links, find, `E`, `S`...) became `bind_pane(Yak, ..)`, and a shed pane gets `h`/`Esc`/`J`/`K`/`#`/`W`/`U`/`?`/`q` (kind-less, as before) plus `y` (copy name) and `r` (rescan). Rejected: leaving the pane's keys kind-less (then the shed pane's bottom hint advertised `Tab:link  Enter:follow  E:edit`, all no-ops there).

Decision: shed-row bindings beyond the claim's open/`y`/`r`: `Space` (collapse) and `/` (search) are required by the spec's rules; `v`, `?`, `q` are repeated for shed rows with the hint labels (as the document rows do) so the Sheds hint keeps `v:views` and `q:quit` and the old Sheds literal survives with `/:search` added. Rejected: dropping `v` on a shed row (the view picker would stop working with the cursor on a root).

Decision: `/` rule. A shed is kept if any child matches or, when a search is set, its name contains it; only the matching children are shown (the claim's literal rule), so a name-only match is a root with no children. All content facets (drawer: type, priority, tags...) apply to the children via `FilterSpec::matches`, not just `search`. Rejected: showing all of a shed's yaks when its name matches (friendlier, but beyond the claim; a phase 6a question).

Decision: the cursor survives a rescan by (shed name, yak id) pair: the same yak under the same shed, else its shed row, else a clamp. A yak touched by two sheds appears under both, and only the one under its own shed is "the same row". Rejected: by yak id alone (jumps to the first shed's copy).

Decision: `Item::Yak` children carry no per-row Kind: `render_list` remembers the current shed's `Shed::rows()` as it walks the rows and looks the tag up there (shown only if the whole title still fits). Rejected: a Kind field on `tree::Row`/`CachedRow` for one view.

Decision: `Space` and the persisted `collapsed` set take `shed:<name>`; a collapsed shed shows the same `▶ N` badge as a yak. `h` (family scope) on the Sheds view says "family scope applies to tree views" instead of cycling a setting nothing reads.

Decision: the shed pane keeps no scroll or cursor (it never has more than a screenful in practice, and its yaks are the rows beside it). `selected_shed_yak`'s duty is the cursor itself: `sheds_state_header` now reports the shed the cursor is in (`shed=`); the yak is the header's own `sel=`.

---
▸ 2026-11-03T04:42:21Z [sheds-2]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T04:42:21Z [sheds-2]

Summary: the Sheds tab is an ordinary tree view. Shed roots (today's order and text, collapsible, `shed:<name>`) with their touched yaks as depth-1 yak rows (kind tag dim at the right); `/` filters; a shed row's pane is the shed, a yak row's the yak (J/K, U/W, every yak key work). `Ctx::Sheds`/`ShedDetail` and the tab's cursors, key handlers and list renderer are deleted.
- Gate: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`: 30 suites ok; `yaks verify` PASS.
- New tests (sheds_tab.rs): rows, a yak the farm lacks, `j` from a root, `Space` collapse + badge, shed vs yak pane and `J`, `Enter` on a yak child, yak keys on a yak not a shed, `U` requests a sync, `/` keeps only matching children (and typed through the box), `y`/`r`, cursor kept across a snapshot; keymap tests for the shed kind and hints.
- Docs: `docs/tui.md` (intro, `W` row, Sheds key table, Sheds tab section, new screenshot), `README.md` line; `?` and the hint are generated.
- Attached: headless frames on a real scratch farm, the fixture's yak-detail frame, and `tui-sheds-yak.svg`.
