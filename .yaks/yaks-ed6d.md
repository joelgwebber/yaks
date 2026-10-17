---
id: yaks-ed6d
title: 'TUI overlay: shed badges on yak rows and a Sheds section in the yak detail'
status: shorn
type: feature
priority: 2
created: '2026-10-13T01:29:06Z'
updated: '2026-10-13T01:54:13Z'
parent: yaks-7204
labels:
- ui
- delta
---

Overlay part (a)+(b) of yaks-7204, after the Sheds tab (yaks-98f1): on the yak views, a row whose yak another shed has changed (outstanding, not landed) carries a small shed badge, and the yak's detail pane gains a Sheds section listing each such shed's change (the TUI form of `yaks show <id> --sheds`). Annotations only: the copy in this checkout's farm decides where a row is and what it shows; yaks that exist only in a shed (ghost rows) are NOT in scope here (design pending on yaks-7204).

---
▸ 2026-10-13T01:29:43Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-13T01:29:44Z [delta-lead]

Claim (delta-lead, yakherd): owner badge-1 (Sonnet worker). Parallel with inbox-1 (yaks-1ea3) and disco-1 (yaks-93fe).

Goal: two annotations on the yak views from the Sheds-tab snapshot (`App.sheds.snapshot`, refreshed every 10 s whatever view is shown since the prep commit).
1. Row badge: in `render::list_item` (and its one caller `render_list`), a yak row with `snapshot.copies(id)` non-empty shows a dim badge `🛖N` (N = number of sheds), placed with the other right-hand badges (near ⏳/✅/star; pick the spot that keeps titles readable at 96 columns, and say where in a Decision note). When any copy is `needs: human`, the badge is the warning accent (yellow) so an ask elsewhere is visible from the tree.
2. Detail section: a `Sheds` section in the yak's detail pane, after the fields and before the body (or wherever `detail::build` puts sections; follow its pattern), one line per copy: shed name, then the same bracket words as `yaks changes` (`hairy -> shaving`, `new, hairy`, `edited`) plus `needs: <v>` and `+N notes`, then (dim) the newest note's first line with its actor. Section absent when there are no copies. Non-interactive lines are fine for now (no new keys).

Data: ONLY through `Snapshot::copies(id)` in src/tui/sheds_tab.rs (it leaves out the shed the TUI runs in and landed changes); do not add queries to sheds_tab.rs (inbox-1 owns that file). If you need a helper, put it in render.rs/detail.rs.

Scope by symbol: src/tui/render.rs `list_item`, `render_list`; src/tui/detail.rs (a new section builder) and src/tui/detail_nav.rs `detail_dlines` (wherever `detail::build(t, &self.all)` is called: append or pass the copies); tests appended at the END of src/tui/tests.rs (inbox-1 puts its tests in sheds_tab.rs, so no conflict); docs: docs/tui.md "Signals" (a bullet for the badge) and the "Detail pane" bullet of "Layout" (the Sheds section). No new key, so no `help_content` change; check `help_hint` needs none.
Out of scope: src/tui/sheds_tab.rs, src/tui/viewmodel.rs, src/tui/handlers.rs (inbox-1), Inbox membership, ghost rows for shed-only yaks, the fixture in sheds_tab.rs (build variants in your tests instead), regenerating docs/assets (the coordinator does it at landing: do NOT run the docshots generator).

Evidence: insta snapshots (fail first) of (i) a tree view with a badged row, one of them yellow for an ask, using `App::new(tasks with ids yaks-8814 and yaks-07fe)` + `app.apply_sheds_snapshot(sheds_tab::fixture::snapshot())`, at 96 and 150 columns; (ii) the detail pane of yaks-8814 with its Sheds section. A test that a row with no copies is byte-identical to before (no badge, no layout shift). Real machine: `yaks tui --headless` on this repo shows badges on the yaks the running workers touch (paste the frame in a note). Full `cargo test --release`. Judge: delta-lead at landing (renders and LOOKS at the frames).

---
▸ 2026-10-13T01:33:35Z [badge-1]

Start: Tue Oct 13 01:33:35 UTC 2026
yaks path: /Users/joel/src/yaks/.delta/worktrees/f34qg6ja8z10/yaks/.yaks/yaks-ed6d.md
Plan: tests first (tests.rs tail), then render.rs list_item badge, detail.rs sheds section, detail_nav.rs wiring, docs.

---
▸ 2026-10-13T01:35:30Z [badge-1]

Failing first (tests appended at the end of src/tui/tests.rs, run before any implementation, `cargo test --release -- shed_badges an_ask_elsewhere a_row_with_no the_detail_pane`):

```
test tui::tests::shed_badges_on_tree_rows_96 ... FAILED     assertion failed: line_of(&frame, "yaks-8814").contains("\u{1f6d6}1")
test tui::tests::shed_badges_on_tree_rows_150 ... FAILED    snapshot assertion for 'shed_badges_on_tree_rows_150' failed (new snapshot)
test tui::tests::an_ask_elsewhere_makes_the_badge_yellow_and_a_note_does_not ... FAILED   Option::unwrap() on None (no badge cell)
test tui::tests::the_detail_pane_lists_the_sheds_that_changed_the_yak ... FAILED   assertion failed: frame.contains("Sheds:")
test tui::tests::a_row_with_no_copies_is_byte_identical_to_before ... ok   (guard: passes before and must pass after)
```

---
▸ 2026-10-13T01:48:47Z [badge-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T01:52:00Z [badge-1]

Decision: badge glyph `🛖N` (the Sheds tab's own glyph, N = count, no space; the issue's form), dim, bold yellow when any copy is `needs: human`. Rejected: reusing ⏳ (means "this yak awaits a human" in THIS farm: a different axis); a bare dot (no count, no link to the Sheds tab); coloring the whole row (shifts attention off the title).
Decision: placement = the right-hand badge cluster, between the ⏳/✅ badges and the star/collapse badge, in the same separator-by-one-space scheme. It sits after the labels, so titles keep the same width as before for a row without copies; for a row with copies the title loses 3 columns (space + 🛖 width 2 + digit => 4) which at 96 columns is still ~45 chars. Rejected: left of the title (shifts every row's id/priority/type columns, a layout change for rows without badges).
Decision: the Sheds section sits after the Labels/Parent/Children/Source/Other-fields block and before the body (the notes live in the body, so the Sheds lines read as "what is elsewhere" above "what is here"), and its lines use a new `detail::Kind::Shed` styled through `md` spans (yellow bracket, bold yellow needs, dim newest note) rather than reusing `Field`/`Warn`: `Field` would grey the first 13 columns as a label (wrong for a shed name) and `Warn` would paint the whole line yellow. `edit_target_at`/`block_index_per_line` ignore it (they only look at Field/Body).
Decision: `detail::build` takes the copies as a third parameter (`&[(&Shed, &YakChange)]`) instead of reaching into App: detail.rs stays App-free; the one caller (`detail_dlines`) passes `Snapshot::copies(id)`; the 12 existing detail tests pass `&[]`.
Decision: row badges and the Sheds section both come only from `Snapshot::copies` (so the TUI's own shed and landed changes are left out); landed-only yaks show neither. No new key, so help_content/help_hint unchanged (checked: neither lists the ⏳/✅ badges).
Decision: only the newest note is shown per shed line (the claim note says "the newest note's first line with its actor"); `+N notes` carries the count. Lines are not truncated; the detail's soft wrap handles the width.
Snapshots (read by eye, accepted by hand): shed_badges_on_tree_rows_96 / _150 (badges `🛖 1` on yaks-07fe and yaks-8814; the emoji is width 2 in the snapshot text so a space shows after it, the span itself is `🛖1`), the_detail_pane_lists_the_sheds_that_changed_the_yak (Sheds: `needs-1-superseded  [shaving]  needs: human  +2 notes  needs-1: asked: needs human`, then the body). Colour is asserted by `an_ask_elsewhere_makes_the_badge_yellow_and_a_note_does_not` (buffer fg), no-layout-shift by `a_row_with_no_copies_is_byte_identical_to_before` (rows for yaks-0000 and a landed-only yaks-ed5e equal the no-snapshot render).
Gate: full `cargo test --release`: all test binaries ok, 0 failed (557 unit + integration). One earlier run had `sheds_layouts::a_flat_shed_forked_before_the_migration...` fail while other workers were building; it passes alone and in the re-run (timing flake, unrelated to the TUI).
Docs parity: docs/tui.md (Layout "Detail pane" bullet, Signals bullet), .agents/skills/yaks/SKILL.md (the TUI/Sheds paragraph). README.md, docs/cli.md and the other skills do not describe TUI row badges for sheds; `grep -rn "⏳" README.md docs/ .agents/skills` shows only needs-badge mentions, unchanged.
Real machine (`printf 'key Tab\nsnapshot\n' | yaks tui --headless --size 150x30`, after the first scan landed; Shaving view, rows where other workers' sheds changed the yak; the Hairy view shows none because those yaks are shaving):
```
 yaks-7204   p3 idea     🪒 Shed deltas as an overlay across the TUI and CLI, not just a shed picker         [ui, cli, delta] 🛖1
   yaks-1ea3 p2 feature  🪒 TUI overlay: the Inbox includes other sheds' open asks, a answers them in place... [ui, delta] 🛖1
   yaks-93fe p3 bug      🪒 sheds: a plain git worktree of a Delta clone is taken for the primary...        [cli, delta] 🛖1
```
(first frame right after start has no badges: the first scan has not landed yet; frames 1-3 after it do. Yellow not exercised on the real machine: no other shed holds an open ask besides the sheds-tab clone's yaks-7204 `needs: human`, which is on yaks-7204 above and shows 🛖1 from that shed's ask.)

---
▸ 2026-10-13T01:52:08Z [badge-1]

Correction to the previous note: a row with a badge loses 4 title columns (separator space + 🛖 at width 2 + one digit), not 3. And on the real machine yaks-7204's badge IS the yellow one in principle (the sheds-tab clone holds `needs: human` on yaks-7204), but headless text frames carry no colour, so yellow is evidenced only by the buffer-fg unit test, not by the pasted frame.

---
▸ 2026-10-13T01:54:06Z [badge-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T01:54:13Z [badge-1]

Summary: shed overlay on the yak views. Row badge `🛖N` (yellow when a copy is `needs: human`) in `render::list_item` via `shed_badge`; `Sheds:` section in the detail (`detail::sheds_section`, `Kind::Shed`, copies passed from `detail_nav::detail_dlines`); data only from `Snapshot::copies`. 4 new insta/unit tests + 3 snapshots at the end of tests.rs; docs/tui.md and the yaks skill updated. Not touched: sheds_tab.rs, viewmodel.rs, handlers.rs, docs/assets, ghost rows.

---
▸ 2026-10-13T01:54:13Z [badge-1]

moved: shaving -> shorn
