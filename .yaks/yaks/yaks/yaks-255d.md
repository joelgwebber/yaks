---
id: yaks-255d
title: Can't edit empty description.
status: shaving
type: bug
priority: 3
created: '2026-11-04T00:00:46Z'
updated: '2026-11-07T17:05:33Z'
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
