---
id: yaks-1bbb
title: 'GFM extras: task lists, alerts, footnotes'
status: shaving
type: task
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T05:17:20Z'
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
