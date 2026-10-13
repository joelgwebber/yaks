---
id: yaks-c1f8
title: 'Sheds tab: a shed''s detail lists outstanding yaks first and folds the landed ones (Enter expands)'
status: shaving
type: feature
priority: 2
created: '2026-10-13T19:46:42Z'
updated: '2026-10-13T19:46:42Z'
parent: yaks-38dc
labels:
- ui
- delta
---

Joel, 2026-10-13: the Sheds tab's detail mirrors `yaks changes` and lists every yak the shed touched since it forked; a coordinator's long-lived shed that is `landed (12 yaks)` still shows twelve bright-looking blocks. Lead with what is outstanding and fold the landed ones behind one line, as `yaks sheds` does.

---
▸ 2026-10-13T19:46:42Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-13T19:46:42Z [delta-lead]
Claim (delta-lead, yakherd): owner fold-1 (Sonnet worker). Parallel with changes-1 (the CLI half, src/sheds.rs render + src/main.rs).

Design (delta-lead, within Joel's ask; say so in a Decision note if something does not hold up):
1. In the Sheds tab's shed detail (src/tui/sheds_tab.rs `detail_lines`, `render_shed_detail`), list the outstanding yaks first, then ONE dim fold line `▸ N landed` when there are landed ones. Use `sheds::split_landed` (in src/sheds.rs, tested) for the order, so it matches `yaks changes`.
2. The fold line is a cursor stop like a yak block (j/k reach it). Enter on it expands the landed yaks below it (then `▾ N landed`, each landed yak's block dim as today); Enter again collapses. Collapsed by default; the state is per App, not per shed, not persisted (say if you choose otherwise).
3. Everything else keeps working: `a`, `y` and Enter on a yak block; `J`/`K`; selection survives a rescan (`apply_sheds_snapshot` keeps the yak by id: a landed yak selected while expanded stays selected; if it is folded away by a collapse, the cursor lands on the fold line).
4. A shed with nothing landed renders exactly as today; one with only landed yaks shows just the fold line under `Changes (N yaks)`.

Scope: src/tui/sheds_tab.rs (the detail model: `selected_shed_yak`, `move_shed_yak`, `detail_lines`, `render_shed_detail`, `handle_shed_detail_key`, tests in its `mod tests`, insta snapshots), `help_content` / `help_hint` in src/tui/render.rs only for the Enter-on-fold line, docs/tui.md (the Sheds tab section and its key table).
Out of scope: src/sheds.rs and src/main.rs (changes-1), the yak views and their overlay, regenerating docs/assets (do NOT run the docshots generator; the coordinator does it at landing).

Evidence (fail first): tests that (i) outstanding blocks come first and one `▸ N landed` line follows; (ii) Enter on it expands and collapses; (iii) j/k reach the fold line and the landed yaks when expanded; (iv) a shed without landed yaks is byte-identical to before (insta); (v) selection survives a rescan as in 3. Insta snapshots of the collapsed and expanded detail (use the fixture; add landed yaks to a VARIANT in your tests, do not change `fixture::snapshot`). Real machine: `yaks tui --headless` from your checkout, the Sheds tab, the coordinator's shed `sheds-tui-thin-sheds-tab` (all landed) collapsed and expanded, pasted. Full `cargo test --release`. Judge: delta-lead at landing (renders and LOOKS at the frames).
