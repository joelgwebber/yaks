---
id: yaks-640a
title: Explicit label definitions in config.yaml
status: hairy
type: feature
priority: 3
created: '2026-08-23T02:49:59Z'
updated: '2026-10-27T01:10:50Z'
labels:
- config
needs: agent
---

While the yaks CLI/TUI doesn't need explicit label definitions for anything, it could still be useful for auto-complete in the UI.
More importantly, if we can find a good way to get context into the calling agent (cf yak-011e), we can use label definitions to help nudge it in the right direction.

---
▸ 2026-09-21T18:37:07Z [agent]

Multi-herd angle: with the farm config herds: map + one-level global->herd cascade (yaks-98d5), label definitions should be per-herd-overridable (project A's labels differ from B's in a consolidated farm). Resolve label defs through that cascade rather than a single global list.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-640a` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-640a --drop-request`, which removes this request note, so no new note is left behind.
