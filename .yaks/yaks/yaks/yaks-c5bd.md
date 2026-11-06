---
id: yaks-c5bd
title: Links to one note of a yak
status: shaving
type: feature
priority: 2
created: '2026-11-06T04:28:29Z'
updated: '2026-11-06T04:57:29Z'
parent: yaks-1c89
depends_on:
- yaks-bca6
tags:
- cli
verify: cargo test --test links && cargo test --bin yaks a_note_link
---

There is no way to link one note of a yak: a link names the yak (`yak-0af1`), a dok section, an attachment or code lines. Session notes (yaks-1c89) want to cite the specific notes a session wrote, and other notes would use it too ("as I said in ...").

Needs: a link form for one note, resolved by `yaks peek` (prints that note), `yaks refs`, `doctor` (reports a note link that leads nowhere) and the TUI (jumps to the note).

The form is settled on the design yak for session notes. Notes are append-only but not immutable: `pickup --drop-request` removes one, so an index shifts; the note's timestamp (plus actor, if two can share a second) does not.

---
▸ 2026-11-06T04:57:01Z [herd-upkeep]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T04:57:07Z [herd-upkeep]
kind: verify
result: pass
---

verify: `cargo test --test links && cargo test --bin yaks a_note_link` -> PASS (exit 0)

[note-links](yaks-c5bd/note-links.txt)
