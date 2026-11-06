---
id: yaks-c5bd
title: Links to one note of a yak
status: shorn
type: feature
priority: 2
created: '2026-11-06T04:28:29Z'
updated: '2026-11-06T04:58:08Z'
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

---
▸ 2026-11-06T04:58:06Z [herd-upkeep]

Done on branch `upkeep` (505d2058): a link can name one note of a yak by its timestamp.

- The form is `yaks-1a2b#2026-11-04T22:13:03Z`; later notes of the same second take `-1`, `-2` ([note_anchor](code:src/links.rs@505d2058#L211-L245)). `Link::Yak` now carries the note.
- `yaks peek` prints just that note, `doctor` reports a note link that leads nowhere, and `yaks update --note` prints the new note's link (`Updated <id>: note <id>#<ts>`).
- The TUI follows one to its yak but not yet to the note: yaks-b6a7.
- Docs: `docs/cli.md` (Links), the `yaks` skill's Links, `peek` and `update --note` help.

Evidence: `yaks verify` PASS (unit and CLI tests), and a run on this farm: [note-links](yaks-c5bd/note-links.txt).

---
▸ 2026-11-06T04:58:08Z [herd-upkeep]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
