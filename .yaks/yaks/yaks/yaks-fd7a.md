---
id: yaks-fd7a
title: attach writes links with unencoded spaces, which markdown does not parse
status: shorn
type: bug
priority: 3
created: '2026-11-01T21:28:56Z'
updated: '2026-11-04T20:03:50Z'
parent: yaks-f301
tags:
- markdown
- doks
verify: cargo test --workspace
---

`yaks attach` writes `![name](<id>/my shot.png)` with the file name unencoded. CommonMark does not parse a destination with a space, so the TUI and GitHub show it as text, and link tools (`refs`, `doctor`, `promote`) miss it (`promote` handles the literal form as a special case).

Fix: write `<id>/my%20shot.png` (or `](<<id>/my shot.png>)`), and have `yaks migrate` or `doctor --fix` re-encode existing ones. Found in yaks-2c42.

---
▸ 2026-11-01T22:01:18Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:04:46Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: 2a42222
---

Fixed on `wiki` (2a42222). `attach` now writes `![my shot](<id>/my%20shot.png)`, encoding spaces, %, parentheses and angle brackets. The TUI image slot, rename overlay and open-attachment all decode the target first. `rename-attachment` follows both the encoded and the old literal form. The schema-7 migration gains an `attachment-link-encoding` fixup, unit-tested: encodes, is idempotent, skips fences, and leaves non-yak targets alone. `farm::tests::attach_encodes_a_name_with_spaces_and_rename_follows_it` checks that the encoded link resolves through `links::exists`. `cargo test --workspace`: 922 passed. Promote's literal-form fallback stays for farms that have not migrated yet.

---
▸ 2026-11-01T22:06:52Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T22:06:53Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
