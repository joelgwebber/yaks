---
id: yaks-4cd7
title: Note parser splits on a --- / ▸ pair inside a fenced code block
status: hairy
type: bug
priority: 2
created: '2026-10-31T15:58:01Z'
updated: '2026-10-31T15:58:01Z'
parent: yaks-039b
labels:
- store
---

`store::is_note_delim` treats any `---` line followed by a `▸ ` line as a note boundary, including inside a fenced code block. A note that shows an example of the note format splits into fake notes, with whatever actor the example names.

Found 2026-10-31: an ask on `yaks-6a11` that included example notes was parsed as extra notes by `[tex-1]`. Worked around by leaving the `---` out of the examples.

## Why it matters now

- A sheds view built from note actors (`yaks-da23`) would list a writer that never wrote to this farm.
- The note-metadata design (`yaks-6a11`) will put more structure under the marker, and its documentation will need examples.

## Fix shape

Make `parse_notes`, `description` and the TUI comment parser skip fenced code blocks (backtick and tilde fences, any length) when looking for delimiters. The same goes for the merge driver if it splits on notes. Test with a note containing a fenced example of a note.
