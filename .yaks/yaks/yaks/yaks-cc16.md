---
id: yaks-cc16
title: 'TUI phase 1: notifications clear on the next key'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:48:20Z'
parent: yaks-947a
tags:
- tui
---

Phase 1 of [the design](/wiki/design/tui-model.md#phases) (yaks-953f, its phase 1): a notification lasts until the next key, and the search commit stops writing a `filter: q` toast. Today `App.notification` is never cleared, so a stale `filter: widget` toast outlives the search it describes.

---
▸ 2026-11-03T03:47:40Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T03:48:20Z [herd-tui]

Claim (yakherd herd-tui): worker `notify-1` (Sonnet 5.5), from main at `9a8c7b4`.

**Task.** A notification lasts until the next key or mouse event, and the search commit stops writing its redundant `filter: <q>` toast. Today `App.notification` is never cleared, so after `/widget` Enter Tab a stale `filter: widget` toast outlives the search (reproduced headlessly).

**Scope by symbol.**

- `handle_key` in `src/tui/handlers.rs` (the outer wrapper only): clear `app.notification` before dispatching. Do the same at the top of mouse handling (`handle_event` or `mouse.rs`'s entry point), whichever is the single entry for mouse events.
- The `Overlay::Search` commit in `src/tui/handlers.rs` (where it sets `filter: {q}`): drop that toast. Keep `search cleared`.
- `src/tui/tests.rs`: a regression test plus an insta snapshot.
- `docs/tui.md`: one sentence where notifications are described (a notification lasts until the next key).

**Out of scope.** `handle_key_inner` and every per-surface key handler: worker `keys-1` is restructuring them in parallel (yaks-fbb0). Anything else: `yaks ask yaks-cc16`.

**Evidence.**

- A test that fails before and passes after: `/widget` Enter Tab leaves no `filter:` text in the tab row and `app.notification` is `None`.
- Existing tests that encoded the sticky toast may be adjusted, each named in a `Decision:` note with why. Background results (a sync finishing, a scan) still show until the next key.
- `cargo test --workspace` passes.

**Judge.** herd-tui at landing.
