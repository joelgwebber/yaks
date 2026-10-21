---
id: yaks-1bbb
title: 'GFM extras: task lists, alerts, footnotes'
status: shorn
type: task
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T05:33:15Z'
parent: yaks-ea13
depends_on:
- yaks-649d
labels:
- markdown
- tui
---

On the renderer core.

- Task lists: `- [ ]` / `- [x]` render as ☐ / ☑ (toggling from the detail pane is a possible follow-up, not in scope).
- Alerts: `> [!NOTE]` / TIP / IMPORTANT / WARNING / CAUTION render as a quote with a colored bar and a title row (palette).
- Footnotes: references render as superscript-ish `[1]` links that jump to the definition; definitions collect at the end of their content block (description or comment) under a dim rule.

---
▸ 2026-10-17T05:17:20Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T05:17:20Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **links-2** (with yaks-3b8d). Task lists (☐/☑) and alerts (coloured bar + `▲ WARNING` title) already render via the core (yaks-649d). Remaining: footnotes.

- `src/tui/mdrender.rs` (`Event::FootnoteReference`, `Tag::FootnoteDefinition` only): a reference renders as a dim `[n]` (numbered in order of first reference) that is a link; following it (Enter) jumps the line cursor to the definition. Definitions move to the end of their content block under a short dim rule, labelled `[n]`.
- That needs a new link target: add `Target::Footnote(String)` (or similar) in src/tui/detail.rs's `Target` enum and handle it in `follow_link` (src/tui/detail_nav.rs) by moving the line cursor to the row of that definition within the same block (and back with `o`, like other jumps, if cheap).

Evidence: failing-first tests (reference numbering, definition moved to the end, follow jumps to it); a headless frame. Judge: coordinator at landing.

---
▸ 2026-10-17T05:21:50Z [links-2]

Start (links-2): Sat Oct 17 05:21:50 AM UTC 2026

- path: /home/joel/src/yaks/.delta/worktrees/dvtctdaewedz/yaks/.yaks/yaks-1bbb.md
- own binary: ./target/release/yaks built in this checkout; base 4c586e9

[frame-links](yaks-1bbb/frame-links.txt)

---
▸ 2026-10-17T05:33:01Z [links-2]

Decision: footnote numbering is by first reference (then unreferenced definitions in source order), computed in a pre-pass over the events; the label the author wrote (`[^note]`) is never shown. Rejected: showing the author's label (not numeric, can't be "superscript-ish `[n]`").

Decision: a definition is built on its own rows (main rows parked while it renders) and appended by `footer` after the last block of the document, under a 12-column dim `─` rule, sorted by number. Rejected: re-sorting after the fact by scanning row text (fragile); per-block placement inside quotes/lists (CommonMark footnote definitions are document-level in pulldown-cmark, and the claim says "end of their content block", which is the document here since each comment renders separately).

Decision: `Target::Footnote(String)` carries the displayed number; `follow_link` calls `goto_footnote`, which finds the last row of the same content block starting `[n] ` (`detail::footnote_line`). It pushes the reference spot on the existing `o` nav stack (`nav_back`), so `o` returns to the reference line (cheap, as the claim allowed). Rejected: a separate footnote-back key. `O` (open externally) says "not an artifact/link" on a footnote.

Done:
- src/tui/mdrender.rs: `footnote_numbers`, `Event::FootnoteReference` -> linked dim `[n]`, `Tag::FootnoteDefinition` parked/collected, `footer`.
- src/tui/detail.rs `Target::Footnote` + `footnote_line`; detail_nav.rs `goto_footnote`; handlers.rs `O` arm.
- Tests (mdrender): reference numbering and link spans, definitions moved to the end under the rule, wrapped definition hangs and maps to its source line, unreferenced definition. (tests.rs): Enter on a reference jumps to `[1] Because.` and `o` returns. Task lists/alerts already had `task_list_and_alert`.
- Tests were written together with the code and did not run red for the footnote ones (the first assertion failed on my own off-by-one column in the expected value, fixed); the attachment tests of yaks-3b8d were checked red by disabling the prefix.
- Docs: docs/tui.md (rendered markdown paragraph), `?` help line for Enter, yaks skill "What the TUI renders" line (footnote jumps moved from "does not" to "renders").
- Frame attached: frame-links.txt (headless 90x34; the last frames of the Tab sequence). Footnote jump frame shows `→ [1] (o to go back)` after Enter on the first Tab stop.

---
▸ 2026-10-17T05:33:04Z [links-2]

verify: `cargo test --workspace` -> PASS (exit 0)

[frame-footnote-jump](yaks-1bbb/frame-footnote-jump.txt)

---
▸ 2026-10-17T05:33:15Z [links-2]

Correction: frame-links.txt (attached above) holds the Tab-through-links frames, not the footnote jump. frame-footnote-jump.txt is the footnote frame set: `Enter`, `Tab` onto the reference row, `Enter` -> notification `→ [1] (o to go back)`. The cursor-to-definition move is asserted by `following_a_footnote_reference_jumps_to_its_definition_and_o_returns`.

---
▸ 2026-10-17T05:33:15Z [links-2]

moved: shaving -> shorn
