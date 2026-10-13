---
id: yaks-1ea3
title: 'TUI overlay: the Inbox includes other sheds'' open asks, a answers them in place, and a new ask flashes'
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
