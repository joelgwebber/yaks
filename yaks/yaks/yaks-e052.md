---
id: yaks-e052
title: 'Notes on doks: the note log as a capability any kind can switch on'
status: hairy
type: feature
priority: 3
created: '2026-11-27T02:09:10Z'
updated: '2026-11-27T02:09:10Z'
parent: yaks-7e32
tags:
- doks
---

Let a dok (and later other kinds) carry a note log like a yak's: `#### <timestamp>` headings with the italic metadata line ([note-format](/doks/design/note-format.md)), appended with `yaks update <path> --note`, unioned by the merge driver, shown in the TUI pane. A design dok could then hold its own review thread instead of keeping it on a yak.

Joel, 2026-11-26: clearly useful, not necessarily day one, unless it falls out of the substrate work for free. Note-kind schemas (yaks-4a0d) become a `notes:` section of a kind schema. Design: section 7 of [the dok](/doks/design/one-document-model.md).
