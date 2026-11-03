---
id: yaks-cc16
title: 'TUI phase 1: notifications clear on the next key'
status: shorn
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:59:16Z'
parent: yaks-947a
tags:
- tui
verify: env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace
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

---
▸ 2026-11-03T03:52:10Z [notify-1]

Start: Tue Nov  3 03:52:10 UTC 2026; farm path: /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-cc16.md; checkout: /home/user/yaks-2

---
▸ 2026-11-03T03:55:33Z [notify-1]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)

[search-commit-tab-frame](yaks-cc16/search-commit-tab-frame.txt)

---
▸ 2026-11-03T03:57:13Z [notify-1]

Decision: the mouse clear sits in `App::handle_mouse` (`src/tui/mouse.rs`), the wrapper over `handle_mouse_inner`, which is the single mouse entry (`handle_event` calls it). It clears even when an overlay is open and the mouse is otherwise ignored, since "next mouse event" is the rule. Rejected: clearing in `handle_event`, which would miss tests and callers that call `handle_mouse` directly.

Decision: a kept search (`Enter` with a non-empty query) writes no notification at all, and the empty commit keeps `search cleared`. Rejected: keeping `filter: q` and relying on the new clear. The yak says to drop it, and a toast that repeats the filter is noise.

Decision: three existing snapshots encoded a stale toast that an earlier key had left on screen and a later key never cleared. Only the toast row changed, to blank, in each:
- `backrefs_picker` (`→ a1`)
- `the_viewer_with_its_cursor_on_a_link` (`viewing t0/notes.md`)
- `the_viewer_following_a_page_link_with_a_peek` (`viewing wiki/topics/hub.md`)

Rejected: changing the tests to hold the toast, since the stale toast is the bug.

Decision: Tab after a committed search resets the filter (a view switch resets it), so the regression frame shows the Shaving tab, not a filtered one. That is existing behaviour and unrelated to this yak.

Surprise: `YAKS_DIR` set in the shell leaks into `cargo test --workspace`. 17-22 preflight tests fail under it (`preflight::tests::*`), because they discover the real farm. So `yaks verify yaks-cc16`, which needs `YAKS_DIR`, records FAIL (exit 101) through no fault of the change. Re-run with `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`: exit 0, 30 `test result: ok` lines, none FAILED. A second leak: a reused `$TMPDIR` makes `tests/init_modes.rs` fail ("share2-web-wt already exists"), so each gate run needs a fresh one.

Regression check: with `handlers.rs` and `mouse.rs` reverted to HEAD, `search_commit_leaves_no_stale_filter_toast` fails (`left: Some("filter: widget")`) and `notification_clears_on_next_key_and_next_mouse_event` fails (`left: Some("sync finished")`). With the change both pass, and so does `empty_search_commit_still_says_cleared`.

Summary: `handle_key` and `App::handle_mouse` clear `notification` first. The `Overlay::Search` commit no longer toasts `filter: q`. 3 new tests plus the snapshot `search_commit_then_tab_no_toast`, which is attached. `docs/tui.md` has one added sentence. Docs parity: grepped for `filter:` toast mentions in `docs`, `README.md`, `.agents/skills` and `src`, and found none to update.

---
▸ 2026-11-03T03:57:19Z [notify-1]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)

---
▸ 2026-11-03T03:57:34Z [notify-1]

Decision: set this yak's `verify:` to `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace`, because the default gate fails under the `YAKS_DIR` this job must export (see the surprise note). Rejected: leaving it and recording FAIL, which says nothing about the change. The coordinator may reset it with `yaks update yaks-cc16 --verify ''`.

---
▸ 2026-11-03T03:58:03Z [notify-1]
kind: verify
result: pass
---

verify: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T03:58:07Z [notify-1]
kind: move
from: shaving
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAD9uc5IBxBCtY3f3KUhDW5IL-PWT3Nbm
---

moved: shaving -> shorn

---
▸ 2026-11-03T03:59:16Z [herd-tui]
kind: landed
commit: f18b13db770c4631ed8a32d99a3390b6076e23e1
source: c51051199ce08daf9e1181a75352169730ccd1d5
branch: main
---

landed: f18b13db yaks-cc16: notifications clear on the next key or mouse event; a kept search no longer toasts filter: q
