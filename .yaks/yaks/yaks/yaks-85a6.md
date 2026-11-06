---
id: yaks-85a6
title: Sessions not visible in doks
status: shaving
type: feature
priority: 3
created: '2026-11-06T21:28:20Z'
updated: '2026-11-06T23:09:20Z'
parent: yaks-3f35
tags:
- tui
---

We get doks and sources, but not sessions, which we've recently begun adding to the mix.

---
▸ 2026-11-06T21:37:39Z [schism-feedback]

Context from yaks-3f35: the schism farm's doks cite session notes 62 times as `[session](/sessions/....md)` (the `yaks-upkeep` skill's review example, copied literally), which is the stray `(session)` Joel saw in `doks/howto/dev-workflow.md`. They are session back-links that lead nowhere until the TUI can open sessions.

- Opening `/sessions/...` links from a dok or yak belongs here, with the sessions list.
- The citation's wording is yaks-5f70.

---
▸ 2026-11-06T22:55:18Z [schism-feedback]
kind: move
from: hairy
to: shaving
thread: ksQQNtlWqzuUT5COVXyFQVm8cZLOAEgKKZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

![tui-doks](yaks-85a6/tui-doks.png "The Doks tab with the fixture's session note after the doks and the source")

[session-frame](yaks-85a6/session-frame.txt)

---
▸ 2026-11-06T23:09:20Z [schism-feedback]

Built on branch `sessions-in-doks` (not landed yet).

- The Doks tab lists every session note after the doks and sources, oldest first, kind `session` in its own colour. The tab's count stays doks only. [fn read_entries](code:src/tui/doks_tab.rs#L77)
- `DoksEntry.source: bool` became `group: DocGroup` (`Dok`, `Source`, `Session`), rather than a second boolean.
- A session note opens like a dok (`AttachView::farm_doc`): frontmatter as field rows, rendered sections, sibling screenshots drawn inline; `E` edits it as a dok.
- Links to `/sessions/...` already opened through the generic farm-file path; they now land on a dok-style view instead of raw text.
- `docs/tui.md` (Doks tab, All view, link targets) and the four Doks docshots updated.

## Evidence

- Tests: `a_session_note_is_listed_and_opens_like_a_dok`, plus the Doks tests and both insta frames updated for the new row; full `cargo test` green.
- Headless release build on Joel's schism farm: 42 rows (23 doks, 19 session notes); `Load a GLB table model` opens with its frontmatter, Summary and screenshot ([frame](yaks-85a6/session-frame.txt)); Tab and Enter on the first `(session)` link in `doks/howto/dev-workflow.md` open `sessions/2025/08/...-dev-workflow-tooling-and-bevy-0-16-1.md`.
