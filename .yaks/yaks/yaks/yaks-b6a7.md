---
id: yaks-b6a7
title: 'TUI: following a note link lands on its note'
status: shorn
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:28:26Z'
parent: yaks-1c89
tags:
- tui
---

A note link (`yaks-1a2b#<timestamp>`, yaks-c5bd) opens its yak in the TUI but leaves the cursor at the top. It should land on the note, as a `#heading` link lands on its heading in the text viewer. Found while building yaks-c5bd.

---
▸ 2026-11-06T23:12:43Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:40Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `tui-1`.

- **Scope:** following a note link (`yaks-1a2b#<ts>`, `links::Link::Yak { note: Some(_) }`) in the TUI opens the yak with the cursor on that note, the way a `#heading` farm link lands on its heading. Today `mdrender::link_target` maps it to `Target::Task(id)` and drops the note. A note that no longer exists opens the yak at the top with a status message saying so. Back (`o`) returns to where you were (`App::jump`, `src/tui/history.rs`).
- **Files:** `src/tui/detail.rs` (`Target`), `src/tui/mdrender.rs`, `src/tui/detail_nav.rs`/`handlers.rs` where `Target::Task` is followed, the detail view's note rows. Anchor by symbol. Shared render paths can move insta snapshots: review every one you accept.
- **Evidence:** a test driving the TUI (toque/headless) to follow a note link and asserting the selected row is that note, plus a text frame attached to the yak. `docs/tui.md` (links), if it describes what following a yak link does.
- **Judge:** `cargo test -p yaks` and the attached frame; the yakherd looks at it at landing.

---
▸ 2026-11-06T23:15:10Z [tui-1]

Start: Fri Nov  6 23:15 UTC 2026

- farm: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-b6a7.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/371r3jvw781f/yaks

---
▸ 2026-11-06T23:27:21Z [tui-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

[b6a7-note-link-frame](yaks-b6a7/b6a7-note-link-frame.txt)

---
▸ 2026-11-06T23:27:36Z [tui-1]

Decision: a new `Target::Note { id, anchor }` variant, not an `Option` note on `Target::Task`.

- Rejected: widening `Task(String)`: it is matched in many places (autolinks, ref rows, history) that never carry a note.
- Rejected: resolving the note in `mdrender::link_target` to a row: the row depends on the pane width, which only the detail model knows.

Decision: the landing row is the note's header row, found in the built detail lines by `Src { block: i + 1, lines: None }` where `i = links::find_note(parse_notes(body), anchor)`.

- Rejected: matching on the header text: it shows an actor and a relative time, not the timestamp.

Decision: `goto_task` and the new `goto_note` share one `goto_yak`, so the cursor move happens inside the same `App::jump` closure and `o` returns to the linking row (also from a link in the text viewer).

Decision: a missing note opens the yak at the top, notification `→ <id> (no note <anchor>)`; a landed one says `→ <id>#<anchor>`.

- Rejected: refusing to open the yak, since the claim note says to open it.

---
▸ 2026-11-06T23:27:37Z [tui-1]

Done: following a note link in the TUI lands the detail cursor on that note.

- `Target::Note` (`src/tui/detail.rs`), produced by `mdrender::link_target`; `goto_note`/`goto_yak` in `src/tui/detail_nav.rs`; `link_dest` shows `id#anchor` for a collapsed link; handlers match arms.
- Tests (`src/tui/tests.rs`): `following_a_note_link_lands_on_that_note_and_o_returns` (a `-1` suffixed note, asserts the cursor row is that note's header and `o` returns), `following_a_link_to_a_missing_note_opens_the_yak_at_the_top`.
- Docs: `docs/tui.md` (history section, yak-link bullets), `docs/cli.md` (note-link line). No key changed, so no `keymap.rs` edit.
- Evidence: frame attached (`b6a7-note-link-frame.txt`); `yaks verify` PASS (full `cargo test`).
- No insta snapshot changed.

---
▸ 2026-11-06T23:27:38Z [tui-1]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjb85IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-06T23:28:26Z [herd-upkeep-2]
kind: landed
commit: 6006e665e18eada8f3e71526a8430003c9f78893
source: fcce895f48fb36292df815df0104bc114f3b79cb
branch: main
---

landed: 6006e665 yaks-b6a7: following a note link in the TUI lands on its note
