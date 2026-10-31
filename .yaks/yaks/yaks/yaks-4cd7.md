---
id: yaks-4cd7
title: Note parser splits on a --- / ▸ pair inside a fenced code block
status: shorn
type: bug
priority: 2
created: '2026-10-31T15:58:01Z'
updated: '2026-10-31T16:52:20Z'
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

---
▸ 2026-10-31T16:08:35Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:16:34Z [yakherd]

Fixed in the prep commit for the `yaks-039b` family: `store::note_starts` skips `---` / `▸` pairs inside closed code fences, and every body scanner uses it (`parse_notes`, `description`, `refs`, the TUI comment parser in `tui/content.rs`). A fence that never closes is ignored, so a broken note can't swallow the notes after it.

## Evidence

- Tests: `a_fenced_example_of_a_note_is_not_a_note`, `an_unclosed_fence_does_not_swallow_later_notes` (store) and `a_fenced_example_of_a_note_is_not_a_comment` (TUI content round trip). The full `cargo test --workspace` passes (761 + integration). The only failure without the fsmonitor workaround is `yaks-9b9f`.
- On the real farm, `yaks log --since 400d` from the old and new binaries differs in exactly two places, both fake notes now read as part of their real note:

```text
< ▸ 2026-10-17T03:05:33Z  yaks-2362  …  [Joel Webber]   (an example inside a fence)
< ▸ 2026-10-31T15:20:00Z  yaks-6a11  …  [tex-1]         (an example inside a fence)
```

---
▸ 2026-10-31T16:16:34Z [yakherd]

Landed in 7f78a28.

---
▸ 2026-10-31T16:16:34Z [yakherd]

moved: shaving -> shorn

---
▸ 2026-10-31T16:52:20Z [yakherd]
kind: landed
commit: 7f78a28c82f9f04486208814cdbfacaf3d08d937
branch: main
---

landed: 7f78a28 yaks-4cd7, yaks-6a11: fence-aware note boundaries; note metadata (key: value lines under the marker, c (recorded by hand: landed with a yaks binary from before `yaks-735a`)
