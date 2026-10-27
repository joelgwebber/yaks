---
id: yaks-c1f8
title: 'Sheds tab: a shed''s detail lists outstanding yaks first and folds the landed ones (Enter expands)'
status: shorn
type: feature
priority: 2
created: '2026-10-13T19:46:42Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-38dc
labels:
- ui
- delta
needs: agent
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

---
▸ 2026-10-13T19:48:26Z [fold-1]

Start: Tue Oct 13 19:47:53 UTC 2026 (worker fold-1)
yaks path yaks-c1f8: /Users/joel/src/yaks/.delta/worktrees/p76ng8mfcj6a/yaks/.yaks/yaks-c1f8.md
Plan: `ShedsTab.line` becomes an index into the detail's *stops*: outstanding yaks, then one fold stop (if any landed), then the landed yaks when `landed_open`. Tests first (frames of a perf-1 variant with landed yaks), seen failing, then the implementation.

---
▸ 2026-10-13T19:50:40Z [fold-1]

Failing first (the cursor model was in place, render still the old one): `cargo test --release sheds_tab` -> 24 passed, 4 failed: outstanding_yaks_lead_and_one_line_folds_the_landed_ones, enter_on_the_fold_line_expands_and_collapses, a_shed_with_nothing_landed_has_no_fold_line, landed_fold_frames (new snapshots). Old frame of perf-1 with two landed yaks:
```
│ Changes (3 yaks)
│ yaks-0001  [shaving -> shorn, landed]  first landed
│ yaks-0002  [shaving -> shorn, landed]  second landed
│ yaks-07fe  [shaving]  yaks sheds is slow: profile it
```
(landed yaks first by id, no fold line). The j/k and rescan tests pass at this stage only because they exercise the cursor model I had already written; the render tests are the ones that fail.

---
▸ 2026-10-13T20:00:52Z [fold-1]

Real machine (my own release binary, `yaks tui --headless --size 150x30` from my checkout; keys: `[` to the Sheds tab, `j j j` to sheds-tui-thin-sheds-tab (14 landed yaks), `l`, `j` to the fold line, then `Enter`). Detail pane only (left list trimmed); frame 1 collapsed (cursor on the fold line, `shed_yak=-`), frame 2 after Enter, expanded, first lines:

```
shed=sheds-tui-thin-sheds-tab · shed_yak=-
│ sheds-tui-thin-sheds-tab  Sheds TUI: thin Sheds tab (yaks-38dc)
│ path     /Users/joel/src/yaks/.delta/worktrees/gcmb7srbmkvv/yaks
│ git      main@d9d8a1d · +1 -0 vs here · 0 dirty
│ farm     landed (14 yaks)
│ label    who: delta-lead, Joel Webber, needs-1, perf-1, inbox-1, disco-1, badge-1, fix-1,
│          ghost-1 · shaving: yaks-38dc, yaks-7422, yaks-c1f8 · active
│ active   2026-10-13T19:47:26Z
│
│ Changes (14 yaks)
│ ▸ 14 landed
│
│
│
│
│
│
│
│
│
│
│
│
│
│
│
│
│
h:list  j/k:yak  J/K:shed  Enter:open yak  a:answer  y:copy id  r:rescan  ?:help
```

```
shed=sheds-tui-thin-sheds-tab · shed_yak=-
│ sheds-tui-thin-sheds-tab  Sheds TUI: thin Sheds tab (yaks-38dc)
│ path     /Users/joel/src/yaks/.delta/worktrees/gcmb7srbmkvv/yaks
│ git      main@d9d8a1d · +1 -0 vs here · 0 dirty
│ farm     landed (14 yaks)
│ label    who: delta-lead, Joel Webber, needs-1, perf-1, inbox-1, disco-1, badge-1, fix-1,
│          ghost-1 · shaving: yaks-38dc, yaks-7422, yaks-c1f8 · active
│ active   2026-10-13T19:47:26Z
│
│ Changes (14 yaks)
│ ▾ 14 landed
│ yaks-07fe  [hairy -> shorn, landed]  yaks sheds is slow (~1.2 s for 4 sheds): profile it and mak
│     2026-10-13T00:11:39Z  delta-lead  moved: hairy -> shaving
│     2026-10-13T00:11:39Z  delta-lead  Claim (delta-lead, yakherd): owner perf-1 (Sonnet worker,
│     2026-10-13T00:12:57Z  perf-1  Start: Tue Oct 13 00:12:57 UTC 2026
│     2026-10-13T00:26:33Z  perf-1  Profile (before binary, release, this machine: 4 live sheds in
│     2026-10-13T00:49:16Z  perf-1  Decision: no persistent cache. The wall time was 80 serial `gi
│     2026-10-13T00:49:16Z  perf-1  Timings, wall clock, 7 runs each (min / median / max, seconds)
│     2026-10-13T00:50:05Z  perf-1  verify: `cargo test -p yaks` -> PASS (exit 0)
│     2026-10-13T00:50:12Z  perf-1  moved: shaving -> shorn
│ yaks-1ea3  [new, shorn, landed]  TUI overlay: the Inbox includes other sheds' open asks, a answe
│     2026-10-13T01:29:43Z  delta-lead  moved: hairy -> shaving
│     2026-10-13T01:29:44Z  delta-lead  Claim (delta-lead, yakherd): owner inbox-1 (Sonnet worker)
│     2026-10-13T01:31:25Z  inbox-1  Start: Tue Oct 13 01:31:22 UTC 2026
│     2026-10-13T01:34:51Z  inbox-1  Tests written first, seen failing before any implementation (
│     2026-10-13T01:40:03Z  inbox-1  Real machine (own binary, `yaks tui --headless --size 150x30`
│     2026-10-13T01:44:17Z  inbox-1  Decision: the Inbox rule lives in one new predicate `App::fla
│     2026-10-13T01:47:55Z  inbox-1  verify: `cargo test -p yaks` -> PASS (exit 0)
h:list  j/k:yak  J/K:shed  Enter:open yak  a:answer  y:copy id  r:rescan  ?:help
```

---
▸ 2026-10-13T20:01:09Z [fold-1]

Decision: the fold line joins the cursor model as a *stop*, not a separate cursor. `ShedsTab.line` is now an index into `detail_stops` (`Vec<Option<&YakChange>>`: outstanding yaks, then one `None` = the fold line when any yak landed, then the landed yaks when open). `selected_shed_yak` returns `None` on the fold line, so `a`, `y` (falls back to the shed path, as outside the detail) and Enter-open need no special case, and `j/k/g/G/d/u` clamp over the stops unchanged. Rejected: a second `fold_selected: bool` beside `line` (two cursors, every move/clamp/render path has to reconcile them) and making the fold a synthetic `YakChange` (would leak a fake yak into `a`/`y`/Enter).
Decision: `landed_open` is one bool on `ShedsTab` (per App, not per shed, not persisted, as the claim said). Rejected per-shed state (a map keyed by shed path that has to survive rescans and drop gone sheds, for a view toggle). J/K to another shed keeps the flag.
Decision: Enter on the fold line toggles and leaves the cursor on the fold line (the landed yaks are below it); Enter on any yak, outstanding or landed, still opens it. A collapse while the cursor is on a landed yak cannot happen from the keyboard (Enter there opens the yak), but a rescan that lands the selected yak while folded drops the cursor onto the fold line (`apply_sheds_snapshot`: by id if still a stop, else the fold line when the id is a landed yak, else 0). `on_fold` is remembered across a rescan too.
Decision: the header count stays `Changes (N yaks)` with N = all yaks (outstanding + landed), as in `yaks changes`; the fold line carries the landed count (`▸ 14 landed`, dim; `▾ N landed` open). Landed yak blocks keep their dim title and `, landed` bracket as today.
Decision: the block rendering moved into `push_yak_block` unchanged (so a shed with nothing landed is byte-identical: the existing `sheds_tab_detail` and `inbox_shed_ask_row` snapshots pass untouched). The detail's `starts` vector is now per stop, same indexing as `line`.
Decision: help bar left as `Enter:open yak` (it already truncates on narrow terminals and the fold hint is in the `?` overlay and docs); `help_content` gets `Enter (on ▸ N landed)`. Rejected a longer bar string (would change the existing bar in every Sheds detail snapshot for a case most sheds never show).
Surprise: `cargo test --release` failed once, `status::tests::status_agrees_with_the_commit_message`, in a full run while my yak file was being edited by notes (that test reads git state); it passes alone and in the second full run. Unrelated to this change, the test reads a temp repo.
Summary: Sheds tab detail leads with outstanding yaks, folds landed ones behind one dim `▸ N landed` stop (Enter expands to `▾`, again collapses), via `sheds::split_landed`. Files: src/tui/sheds_tab.rs (model, render, 6 new tests + 3 insta snapshots: folded, expanded, all-landed), src/tui/render.rs (help_content), docs/tui.md (key table + Sheds tab section). docs/assets not regenerated (coordinator at landing; `tui-sheds-detail.svg` is unchanged in content since the fixture's perf/asker sheds have nothing landed).

---
▸ 2026-10-13T20:04:57Z [fold-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T20:05:05Z [fold-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-c1f8` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-c1f8 --drop-request`, which removes this request note, so no new note is left behind.
