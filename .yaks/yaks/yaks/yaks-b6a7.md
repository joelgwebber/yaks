---
id: yaks-b6a7
title: 'TUI: following a note link lands on its note'
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:13:40Z'
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
