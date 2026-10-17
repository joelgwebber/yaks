---
id: yaks-597d
title: In-place block editing in the detail pane (E on body, M for new comment)
status: shaving
type: feature
priority: 2
created: '2026-10-17T17:02:15Z'
updated: '2026-10-17T17:02:15Z'
parent: yaks-a893
labels:
- tui
- markdown
---

Implements the design Joel approved on yaks-a893 (read its design note and mock frame first).

- `E` on a body row (description or a comment) turns that content block into an edtui editor at the same position in the detail pane; the rest stays rendered and scrolls as usual. The editing block keeps its header, accented, with an `editing · Ctrl-S save · Ctrl-C cancel` hint and a left gutter bar. It shows raw source.
- The editor's cursor lands on the source line behind the row the detail line cursor was on (`DLine.src` from yaks-649d). This delivers yaks-4fa6.
- `M` opens an empty editor under a provisional `▸ <you> · now` header after the last comment.
- Height: the block's source lines + 1 spare row, capped so ~4 rows of context stay visible; edtui scrolls inside beyond that. The detail scroll moves only as much as needed to show the whole editor.
- `Ctrl-S` saves only that block, through `content::assemble` (other blocks round-trip byte-identical; `M` appends via the existing comment path, attributed as today). `Ctrl-C` / `Esc` cancel with the existing dirty-confirm. After saving, the block re-renders in place and the line cursor stays on the same source line.
- Docs/help: `help_content`, `help_hint`, docs/tui.md, the docshots (replace the `M`/`E` frames).

---
▸ 2026-10-17T17:02:15Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T17:02:15Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **inplace-1**.

Scope: src/tui/render.rs (`render_detail` + a helper that lays out the editor region inside the row list), src/tui/handlers.rs (key routing while a block editor is open; commit/cancel), src/tui/actions.rs (`open_edit_at_cursor` / `open_comment` routing body targets to the new overlay; header targets still open the form), src/tui.rs (a new `Overlay` variant, e.g. `BlockEdit { id, block: Option<usize>, editor: Editor, ... }`), src/tui/detail_nav.rs (scroll arithmetic so the editor region counts as N rows), src/tui/mouse.rs if hit-testing needs it, src/tui/editor.rs (reuse `Editor`, `editor_theme`, `set_md_highlights`; no changes to their behaviour). Tests in src/tui/tests.rs. Docs: docs/tui.md, help_content/help_hint, docshots for the new states.

Out of scope: removing the form's content blocks (the follow-up yak), mdrender.rs, palette.rs, the edtui fork.

Evidence: failing-first tests (E on a comment row opens the block editor at that block with the cursor on the right source line; Ctrl-S saves only that block, others byte-identical; M adds a comment at the end; Esc with edits asks to confirm; the rendered rows above the editor stay put); headless frames of E and M with context visible; docshot SVG → PNG via Firefox, looked at. Judge: coordinator at landing, then Joel by trying `~/go/bin/yaks`.
