---
id: yaks-255d
title: Can't edit empty description.
status: shaving
type: bug
priority: 3
created: '2026-11-04T00:00:46Z'
updated: '2026-11-07T17:24:32Z'
tags:
- tui
---

When a yak's description is empty, there's no way to select it for editing in the TUI. You can only add a comment and edit those. It's interesting that there's such a significant difference between "create" and "edit" for the yak as a whole (ignoring comments) -- when creating a yak, you can tab/ctrl-n your way into the description, but not when editing.

ISTR this was an explicit simplification in the inline editor design, but I think the patient might have lost something important in this particular surgery. Let's see if there's a way to better unify the create/edit modes so that they both naturally get the right behavior here. I don't see a strong need to _also_ extend this to comment editing for the time being, which I hope will simplify things somewhat.

---
▸ 2026-11-07T17:04:41Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:05:33Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-form`, who then takes yaks-000b in the same checkout (both touch the create/edit form).

**Scope.** `E` on an existing yak opens the form with the same shape as create: the header rows plus the description editor, seeded with the yak's current description. Tab / Ctrl-N reach it as in create. Saving writes the header fields and the description together, through `Farm` under the farm lock; notes are untouched. An empty description is then editable.

- In-place editing of a body block in the pane (`E` on a description or note row) stays as it is.
- Files: `src/tui/create.rs`, the form's save path in `src/tui/actions.rs`/`handlers.rs`, render of the form, `docs/tui.md`, the `E` help lines in `src/tui/keymap.rs`.

**Out of scope.** Comment editing; the list and keymap otherwise.

**Evidence.** A test that fails first: a yak with an empty description, `E`, move to the description row, type, save; the file has the description and its notes intact. A headless frame of the edit form with the description row, attached. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-07T17:07:48Z [stars-form]

Start: Sat Nov  7 17:07:47 UTC 2026

- farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-255d.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/e3chjzn4e2pq/yaks

[frame-255d-edit-form-description](yaks-255d/frame-255d-edit-form-description.txt "Headless frame: E on a yak with an empty description and one note, Tab to the description row, typed text; the note is not in the form")

![edit-herd](yaks-255d/edit-herd.svg "docs scene tui-edit-herd (regenerated): the multi-herd edit form now ends in the description editor")

---
▸ 2026-11-07T17:24:25Z [stars-form]

Decision: the description is saved through the existing `Farm::update` (`TaskEdit::description` + `base`), not a new `Farm` setter. It already writes a description under the one farm lock together with the header fields, and its `base` keeps comments appended since the form opened. Rejected: a new `Farm::set_description` (a second lock or a duplicate of `update`'s fields), and `content::parse`/`assemble` (it rewrites the comments to the canonical shape, so they are not byte for byte).

Decision: new pure helper `store::with_description(body, new)` beside `store::description`: the comments are the body's lines from the first note start on, verbatim, set off by one blank line. A description that was not changed is not sent at all (`CreateForm::changed_body` returns `None`), so a header-only edit leaves the file's body alone.

Decision: seed with `content::parse(body)[0].text` (trailing whitespace trimmed, leading indentation kept), not `store::description` (which trims both ends and would strip the indent of a leading code block on save).

Decision: a description conflict (the farm refuses: the body changed other than by appended comments) closes the form like any save, writes nothing (header fields too), and puts the typed description on the clipboard, as `reopen_after_conflict` does for a gone block. Rejected: reopening the form with a refreshed base (it would let a re-save overwrite text the user never saw).

Decision: `CreateForm::description` is now a plain `RefCell<EditorState>` (was `Option`), as the form has one shape. `seed_body` holds the body at open time: the `base`, and what a changed description is compared with.

Decision: regenerated only `docs/assets/tui-edit-herd.svg`, the one scene showing the form. The regeneration also changed its tab strip and opacity styling, because the renderer moved since it was last made (`Doks` tab, `fill-opacity`); the other 24 regenerated SVGs were discarded.

---
▸ 2026-11-07T17:24:32Z [stars-form]

Fail-first: on the unchanged source (create.rs, handlers.rs, store.rs checked out; tests.rs adapted to the old `Option`), the four new tests fail:

- `edit_form_has_the_description_row_seeded_without_the_comments`: `left: 8, right: 9` (row count)
- `edit_form_sets_an_empty_description_and_keeps_the_notes`: never reaches the description row
- `edit_form_saves_the_header_and_a_changed_description_together`: body not rewritten
- `edit_form_description_conflict_writes_nothing`: `left: "Zmine solo"`, `right: "solo"` (the header was saved, the description was not on the form at all)

With the change they pass, plus `store::note_tests::with_description_replaces_the_description_and_keeps_the_notes_verbatim`. The existing `edit_form_title_edit_leaves_the_body_byte_identical` and `edit_form_without_edits_is_not_dirty_and_keeps_the_body` still pass. Snapshot `edit_form_panel` changed only by gaining the `description` rule and `First line.` row; accepted.

Docs parity: `docs/tui.md` (Edit form paragraph), the two `E` help lines in `keymap.rs` (`bind_row` and `bind_pane`), the `docshots.rs` scene comment, the `CreateForm` doc comment.
