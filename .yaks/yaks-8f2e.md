---
id: yaks-8f2e
title: 'Docs/skills: large-farm command-line traps (xargs silent chunking, E2BIG on absolute globs)'
status: hairy
type: task
priority: 2
created: '2026-10-18T21:15:00Z'
updated: '2026-10-18T21:15:00Z'
parent: yaks-73f4
labels:
- docs
- skills
---

From yaks-b32e (scale-1): `ls .yaks/*.md | xargs wc -l | tail -1` silently gives a WRONG total once xargs splits the list (from ~3,110 absolute / ~10,070 bare paths); `cmd /abs/.yaks/*.md` fails loudly with E2BIG past ~41k files. Nesting makes the xargs trap worse, so the fix is guidance and tools, not layout. Add one short section to docs (and the yaks skill's "Running yaks"/filtering area, which agents read): prefer `yaks path <filters>` / `yaks list --json`, `rg -l` and `find -exec ... +`, never summarize through `ls | xargs`. Consider whether `yaks path` should stream (it does) and whether a `yaks path --null` (for `xargs -0`) is worth adding.
