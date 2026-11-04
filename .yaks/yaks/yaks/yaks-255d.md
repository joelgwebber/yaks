---
id: yaks-255d
title: Can't edit empty description.
status: hairy
type: bug
priority: 3
created: '2026-11-04T00:00:46Z'
updated: '2026-11-04T00:00:46Z'
tags:
- tui
---

When a yak's description is empty, there's no way to select it for editing in the TUI. You can only add a comment and edit those. It's interesting that there's such a significant difference between "create" and "edit" for the yak as a whole (ignoring comments) -- when creating a yak, you can tab/ctrl-n your way into the description, but not when editing.

ISTR this was an explicit simplification in the inline editor design, but I think the patient might have lost something important in this particular surgery. Let's see if there's a way to better unify the create/edit modes so that they both naturally get the right behavior here. I don't see a strong need to _also_ extend this to comment editing for the time being, which I hope will simplify things somewhat.
