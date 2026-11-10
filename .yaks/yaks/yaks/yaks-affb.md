---
id: yaks-affb
title: Copy dok path with `y`
status: shorn
type: feature
priority: 3
created: '2026-11-10T17:36:26Z'
updated: '2026-11-10T18:44:24Z'
tags:
- doks
- tui
---

Just like on yak list & details. At present, `y` just copies the current line of a dok. It should require that the line be selected first (like yaks) in order to copy it.

---
▸ 2026-11-10T17:49:47Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:30Z [herd-unlock]

Claim (yakherd herd-unlock): worker `dokcopy-1`, Sonnet 5.5.

## Scope

In the text viewer (a dok open in the right pane), `y` with no selection copies the dok's path (farm-root form, `/doks/...`), as `y` does on a yak; with a selection, it copies the selection. Today it copies the current line. Find why the current line counts (the `SelectYank` path in the `Viewer` context, `src/tui/keymap.rs` and its handler) and fix the rule, not the key. Help line and hint, `docs/tui.md`.

## Out of scope

The Doks tab row `y` (already copies the path), other contexts.

## Evidence

A test: viewer open, no selection, `y` puts the path on the (test) clipboard; with a `v` selection it copies the selection. Gate `cargo test --workspace`. Judge: the yakherd.

---
▸ 2026-11-10T17:51:32Z [dokcopy-1]

Start: 2026-11-10T17:51:29Z

- yaks path yaks-affb: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-affb.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/am99yre814ns/yaks

---
▸ 2026-11-10T17:55:22Z [dokcopy-1]
kind: ask
needs: human
---

asked: needs human

Cannot reproduce the reported bug at base `1a3388a8` (v0.0.35), and the claim's path form differs from what ships. Need a decision before I change anything.

Observed (a throwaway probe test driving the real `handle_key`, since removed; the tree is clean):

- Doks tab, `Enter` on a dok, `y` with no selection: notification `copied doks/howto/release.md`, anchor `None`. It copies the path, not the current line.
- `j j v v y` (select then unselect, then `y`): same, the path.
- Cause of the shipped behaviour: yaks-a036 (commit `3e43d41e`, in v0.0.35) made `yank_attachment` (`src/tui/handlers.rs`) copy `AttachView::copy_path()` when `nav.selection()` is `None`; with a selection it copies the rows. The `SelectYank` route in `Viewer` already goes through `run_viewer_cmd`, so there is no "current line counts" rule left to fix. The help line already reads "Copy the selection ..., else the file's path".

Questions:

1. Is the yak stale (done by yaks-a036)? Then the remaining work is only a test pinning "no selection copies the path, `v` selection copies the selection" (no test exercises the key today, only `copy_path()`), plus any wording fix.
2. Or do you want the path in farm-root form `/doks/...`, as the claim says? Today the viewer and the Doks list row both copy `doks/...` (no leading slash), and `copy_path_tests` pins that. Changing the viewer alone would make it differ from the list `y`, which is out of scope; changing both touches the Doks row `y`. Which do you want?
3. Or was the report about a different surface (e.g. a pane peeking a dok under the viewer, or an older installed binary)? Tell me where it happens.

My recommendation: 1, adding the key-level test and a clarified help line only.

---
▸ 2026-11-10T18:36:28Z [sched-1]
kind: answer
needs: agent
---

Whoops, forgot to rebuild. Working fine.

---
▸ 2026-11-10T18:36:37Z [sched-1]

Already done.

---
▸ 2026-11-10T18:36:37Z [sched-1]
kind: move
from: shaving
to: shorn
---

moved: shaving -> shorn

---
▸ 2026-11-10T18:44:24Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: Joel confirmed it works after a rebuild (yaks-a036 already did it); nothing more to do. Worker dokcopy-1 made no commit.
