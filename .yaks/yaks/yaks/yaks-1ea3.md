---
id: yaks-1ea3
title: 'TUI overlay: the Inbox includes other sheds'' open asks, a answers them in place, and a new ask flashes'
status: shorn
type: feature
priority: 2
created: '2026-10-13T01:29:06Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-7204
labels:
- ui
- delta
needs: agent
---

Overlay part (d) + the arrival flash of yaks-7204: the Inbox view (and its tab count) also lists a yak of this farm whose copy in another shed is awaiting a human (the TUI form of `yaks inbox --sheds`); `a` on such a row answers it in that shed (`yaks answer <id>@<shed>`, as the Sheds tab's `a` does); and when a scan brings an ask that was not in the previous snapshot, a notification names it. Yaks that exist only in a shed are not rows here (design pending on yaks-7204); the flash and the Sheds tab still show them.

---
▸ 2026-10-13T01:29:43Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-13T01:29:44Z [delta-lead]

Claim (delta-lead, yakherd): owner inbox-1 (Sonnet worker). Parallel with badge-1 (yaks-ed6d) and disco-1 (yaks-93fe).

Goal:
1. Inbox membership: the Inbox view (key `inbox`, `view::inbox_view`, flat, `needs_only`) also shows a yak of THIS farm whose copy in another shed is awaiting a human: an id in `snapshot.open_asks()` (src/tui/sheds_tab.rs; it leaves out this shed and landed changes) that `self.task(id)` finds. Its tab count (`view_count`) agrees. The drawer's `inbox` chip on other views: leave as is (home `needs:` only) unless it is trivial to share the rule; say which in a Decision note. A shed-only yak (not in this farm) is not a row (ghost rows are pending design on yaks-7204).
2. `a` on such a row (list or detail): when the home copy is not awaiting a human but a shed copy is, answer it in that shed: reuse `EditAction::AnswerInShed` and `App::commit_answer_in_shed` from sheds_tab.rs (several sheds asking about one yak: the first in `open_asks()` order, and the prompt names the shed). Otherwise `a` behaves exactly as today (`open_ask_or_answer` in src/tui/handlers.rs).
3. Arrival flash: in `App::apply_sheds_snapshot`, compare the new snapshot's open asks with the previous snapshot's (key: shed path + yak id + the ask note's timestamp, so a re-ask counts again) and, for ones not seen before, set `self.notification` to `⏳ new ask in <shed>: <id> <title>` (one line; with several, the first plus `(+N more)`). No flash on the FIRST snapshot (startup), and none for this shed's own asks.

Scope by symbol: src/tui/viewmodel.rs (`flat_rows`, `view_count`, or a small helper they share), src/tui/handlers.rs `open_ask_or_answer` (+ a helper), src/tui/sheds_tab.rs (`apply_sheds_snapshot`, new helpers, tests in its `mod tests`; you may add a fixture VARIANT in your tests, do not change `fixture::snapshot`, badge-1's tests use it). Docs: docs/tui.md (the Inbox paragraph near the screenshots, the `a` row of the Edit key table, the Sheds tab section if it mentions the flash), `help_content`'s `a` entry text in src/tui/render.rs (only that one line: badge-1 edits `list_item` in the same file), README's TUI paragraph, the yaks skill's TUI paragraph (.agents/skills/yaks/SKILL.md, the `In yaks tui` line).
Out of scope: src/tui/render.rs beyond that help line, src/tui/detail.rs, src/tui/detail_nav.rs, src/tui/tests.rs (badge-1), ghost rows, regenerating docs/assets (do NOT run the docshots generator; the coordinator does it at landing).

Evidence (fail first): unit tests that (i) the Inbox rows and count include a home yak whose shed copy asks and exclude a shed-only one; (ii) `a` on it opens the AnswerInShed prompt naming the shed, and on a home-blocked yak still opens the plain answer; (iii) the flash fires for a new ask, not on the first snapshot, not again for the same ask, again for a re-ask (new timestamp); an insta snapshot of the Inbox with such a row. Real machine: `yaks tui --headless` on this repo with a scratch `git worktree` shed you create in your own scratch dir (ask there, check the Inbox row and the flash, answer it from the Inbox, check the shed's file; remove the worktree after). Full `cargo test --release`. Judge: delta-lead at landing.

---
▸ 2026-10-13T01:31:25Z [inbox-1]

Start: Tue Oct 13 01:31:22 UTC 2026
yaks path: /Users/joel/src/yaks/.delta/worktrees/hers5ay2699w/yaks/.yaks/yaks-1ea3.md

---
▸ 2026-10-13T01:34:51Z [inbox-1]

Tests written first, seen failing before any implementation (cargo test --release sheds_tab):
```
test tui::sheds_tab::tests::inbox_includes_a_home_yak_whose_shed_copy_asks ... FAILED   left: ["yaks-07fe"]  right: ["yaks-07fe", "yaks-8814"]
test tui::sheds_tab::tests::a_on_a_shed_ask_row_answers_it_in_that_shed ... FAILED     (row yaks-8814 not in the Inbox)
test tui::sheds_tab::tests::a_new_ask_flashes_once_and_a_re_ask_flashes_again ... FAILED  left: None  right: Some("⏳ new ask in perf-1-profile: yaks-07fe yaks sheds is slow: profile it")
test tui::sheds_tab::tests::several_new_asks_name_the_first_and_count_the_rest ... FAILED
test tui::sheds_tab::tests::inbox_frame_with_a_shed_ask_row ... FAILED  (new snapshot)
test result: FAILED. 16 passed; 5 failed
```

---
▸ 2026-10-13T01:40:03Z [inbox-1]

Real machine (own binary, `yaks tui --headless --size 150x30`, scratch `git worktree add --detach $DELTA_SCRATCH_DIR/wt HEAD`; discovery names that shed `main` today, yaks-93fe):
- `YAKS_ACTOR=inbox-scratch yaks ask yaks-0012 --note "which way, scratch?"` in the scratch shed.
- Inbox (view picker, 5th row) showed 4 rows; the new last row `yaks-0012 p2 task 🦬 Op model + compile: yak operations, Lamport clocks, snapshot` was NOT awaiting a human at home (only the shed asks), the other three are home `needs:` rows. `G` then `a` opened the prompt `Answer yaks-0012 in shed main (Enter save · Ctrl-C cancel): `; typed text + Enter gave `answered yaks-0012 in main: uncommitted there, for its worker to commit`, and the row dropped (rows=3).
- Scratch shed's file after: `git diff` there = `+needs: agent`, the inbox-scratch ask note and the answer note "scratch answer from the inbox", uncommitted. 
- Flash: with the TUI running (first snapshot taken), `yaks ask yaks-00c4` in the scratch shed; next scan frame header showed `⏳ new ask in main: yaks-00c4 Affordance for auto-creating a yak from an upstream issue`, and it stayed (no flash at startup, because the earlier frames carried none).
- Worktree removed (`git worktree list` shows only the clone).

---
▸ 2026-10-13T01:44:17Z [inbox-1]

Decision: the Inbox rule lives in one new predicate `App::flat_matches` (viewmodel.rs) used by both `flat_rows` and `view_count`: `spec.matches`, or, when the spec is `needs_only`, a yak whose id is in `Snapshot::asked_ids()` (ids of `open_asks()`) and that matches the spec with `needs_only` off (so search/type/etc. still apply). Rejected: adding a field to `FilterSpec` (on-disk saved views, filter.rs is outside scope) and duplicating the check in rows and count.
Decision: the drawer's `inbox` chip on other views is left as is (home `needs:` only). Sharing the rule would need the chip's FilterSpec to know about sheds; not trivial here, so not done. (Side effect: `needs_only` set live in any flat view, e.g. via the chip on Recent, also takes the shed asks, because `flat_matches` keys on the spec; tree views use `tree::build` and do not.)
Decision: `a` gives the plain answer when the home copy is awaiting a human, else the shed answer if `open_asks()` has the yak (first in open_asks() order), else the plain ask. This applies in any view where the selected yak has a shed ask, not only the Inbox (a yak with an open shed ask on Hairy now answers in the shed on `a`, rather than raising a home ask). Rejected: gating on the active view being the Inbox: the badge overlay already shows the ask on every view, and `a` answering the visible ask is the same rule.
Decision: the prompt is shared: `open_answer_in_shed_prompt(id, &Shed)` in sheds_tab.rs is used by the Sheds detail `a` and the Inbox `a`; label `Answer <id> in shed <name> (Enter save · Ctrl-C cancel): `.
Decision: flash key is (shed path, yak id, timestamp of the newest ask-note in the shed copy; empty if none). `seen_asks` is only updated from a scan without error; the first good scan sets the baseline with no flash. A failed scan neither flashes nor resets the baseline. Rejected: keying on the yak id alone (a re-ask would not count).
Decision: with several new asks the one named is the first in `open_asks()` (attention) order, plus `(+N more)`.
Decision: `fixture::snapshot` untouched; tests derive variants by mutating a copy.

---
▸ 2026-10-13T01:47:55Z [inbox-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T01:48:05Z [inbox-1]

Summary: Inbox rows and tab count include a home yak whose shed copy asks (`App::flat_matches`); `a` answers it in the first asking shed via the shared `open_answer_in_shed_prompt`; `apply_sheds_snapshot` flashes `⏳ new ask in <shed>: <id> <title>` (+N more) for asks not in the previous good scan, keyed by shed path + id + ask-note ts. Docs: docs/tui.md (Inbox paragraph, `a` row), README TUI paragraph, yaks skill TUI line, `a` help_content line. Gate: full `cargo test --release` ok (563 passed + integration suites), `yaks verify` PASS. New insta snapshot inbox_shed_ask_row read by eye (row yaks-8814 listed under the home-blocked yaks-07fe). Not run: docshots generator (coordinator). Caveat: test `a_outside_the_inbox_is_as_before` is weak (Hairy's selection is whatever the tree holds); the unit tests (ii) are the two a_on_* tests.

---
▸ 2026-10-13T01:48:05Z [inbox-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-1ea3` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-1ea3 --drop-request`, which removes this request note, so no new note is left behind.
