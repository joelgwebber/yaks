---
id: yaks-92e5
title: Edit form keeps only header fields (drop content blocks and the accordion)
status: shaving
type: task
priority: 3
created: '2026-10-17T17:02:15Z'
updated: '2026-10-17T17:52:40Z'
parent: yaks-a893
depends_on:
- yaks-597d
labels:
- tui
---

Second half of the yaks-a893 design: with body editing in place, `E` on a header row opens the form with title/type/priority/labels/source/herd only. Remove the per-comment `ContentBlock`s in edit mode, `render_content_stack`'s accordion, `EditTarget::Content` routing to the form, and yaks-c6f4's problem with them. Create (`c`/`C`) keeps its single description block. Depends on the in-place editing yak landing first.

---
▸ 2026-10-17T17:52:40Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T17:52:40Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **formtrim-1**. In-place editing (yaks-597d) has landed (41394a2): `E` on a body row in the detail pane edits that block in place (`Overlay::BlockEdit`, src/tui/blockedit.rs). Now trim the form.

Scope:
- src/tui/create.rs: in edit mode (`CreateForm::for_edit`), no `ContentBlock`s; the form is the header rows only. Create (`CreateForm::new`) keeps its one description block. Remove what only the edit accordion needed (`render_content_stack`'s multi-block accordion, the description preview, `render_block_separator` if unused, block navigation in edit mode). `assembled_body`/`commit_edit_form` (src/tui/handlers.rs) must then leave the body untouched on an edit.
- src/tui/actions.rs: `EditTarget::Content` no longer routes into the form. `E` on the list pane: opens the form (header fields only) — keep it, but its focus starts at title.
- Remove dead code you orphan (`EditTarget::Content` handling in the form, block-nav keys in the form handler). Keep `detail::comment_label` (the detail pane uses it).
- Tests: adjust/replace tests that assumed form content blocks (grep `blocks`, `content_index`, `Content(`); add failing-first tests that (a) `E` on a list row opens a form with no content rows, (b) a form edit of the title leaves the body byte-identical, (c) create still has its description block.
- Docs/help: docs/tui.md's edit-form description, `help_content`, any docshot showing the edit form with content blocks (regenerate only those).

Out of scope: blockedit.rs, mdrender.rs, the edtui fork. When done, also slaughter nothing: the yakherd handles yaks-c6f4 / yaks-4fa6 / yaks-a893.

Evidence: tests above; a headless frame of the trimmed edit form; docshot PNG looked at. Judge: coordinator at landing.
