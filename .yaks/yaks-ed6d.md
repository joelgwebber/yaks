---
id: yaks-ed6d
title: 'TUI overlay: shed badges on yak rows and a Sheds section in the yak detail'
status: shaving
type: feature
priority: 2
created: '2026-10-13T01:29:06Z'
updated: '2026-10-13T01:29:44Z'
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
