---
id: yaks-a036
title: Consistent copy behavior across yaks/docs
status: shaving
type: bug
priority: 3
created: '2026-11-04T00:06:56Z'
updated: '2026-11-07T17:38:39Z'
tags:
- tui
---

Copy in a yak does the right thing (copying the yak id) by default, if no line is selected with [v]isual row selection. But docs always copy the current line, never the actual doc path, which would be both more consistent, and sometimes what we actually want.

---
▸ 2026-11-07T17:04:43Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:38:39Z [herd-stars]

Done by herd-stars in `3e43d41e`: `y` in the text viewer with nothing selected copies the document's path, as `y` in a yak's pane copies its id; `v` then `y` still copies lines.

- What it copies is what the Doks list's `y` copies for the same row: a dok's or session note's farm path, a source by its directory, an attachment as `<id>/<file>` ([AttachView::copy_path](code:src/tui/handlers.rs@3e43d41e#L134-L146)).
- Evidence: unit test `y_without_a_selection_copies_the_documents_path`; full gate green. Not driven headlessly, since that would write the real clipboard.
- Help line (`?`) and `docs/tui.md` (the Doks table and the `A`/`O` row) updated.
