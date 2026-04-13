---
id: yak-a6f5
title: Support external artifacts
type: feature
priority: 2
created: '2026-04-13T15:59:33Z'
updated: '2026-04-13T19:25:34Z'
commit: d65d247
---

It would be really nice if we had a simple affordance for linking things like screenshots. We'd
need to think through how to manage the files (do they live in their own dir? do we reference count them or just let them grow without bound? can you paste an image straight into the terminal?), but it sure would be useful.

## Plan

**Storage.** `.yaks/artifacts/{yak-id}/{name}.{ext}`. Per-yak subdir — obvious ownership, trivial cleanup on slaughter, no ref-counting.

**References.** Standard markdown in the task body: `![desc](artifacts/yak-a6f5/shot.png)`. Path relative to `.yaks/`. Grep-able, hand-editable, no schema change. Coding agents (Claude et al.) pick up image paths from markdown naturally.

**CLI.**
- `yak attach <id> <path> [--name foo] [--desc "..."]` — copies file into the yak's artifact dir, appends `![desc](artifacts/{id}/{name}.{ext})` to body.
- `yak attach <id> --paste [--name foo]` — pulls image from clipboard. macOS: `pbpaste -Prefer public.png` (fallback: `osascript` to write clipboard PNG). Linux: `xclip -selection clipboard -t image/png -o`. Default name `paste-{timestamp}.png`.
- `yak detach <id> <name>` — removes file and its markdown reference line.

**TUI.**
- `a` key — prompt: path or `[p]aste`. Reuses CLI plumbing.
- Detail pane parses `![](artifacts/...)` links out of the body and shows an "Artifacts" section listing them.
- `o` on a selected attachment shells out to `open` (Darwin) / `xdg-open` (Linux).

**Lifecycle.** Artifacts stay on disk indefinitely. Slaughter is reversible (via `revive`), so deleting artifact dirs there would lose data. `detach` is the explicit way to remove a file; add a GC pass later if orphans pile up.

**Explicit non-goals (v1).** Inline terminal image rendering (iTerm2/Kitty protocols), cross-yak dedup, size limits, compression, automatic thumbnails.

![testes](artifacts/yak-a6f5/paste-20260413-190412.png)

![test 2](artifacts/yak-a6f5/subtext+devtools.pdf)
