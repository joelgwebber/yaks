---
id: yaks-8f2e
title: 'Docs/skills: large-farm command-line traps (xargs silent chunking, E2BIG on absolute globs)'
status: shorn
type: task
priority: 2
created: '2026-10-18T21:15:00Z'
updated: '2026-10-22T00:59:43Z'
parent: yaks-73f4
depends_on:
- yaks-b048
labels:
- docs
- skills
---

From yaks-b32e (scale-1): `ls .yaks/*.md | xargs wc -l | tail -1` silently gives a WRONG total once xargs splits the list (from ~3,110 absolute / ~10,070 bare paths); `cmd /abs/.yaks/*.md` fails loudly with E2BIG past ~41k files. Nesting makes the xargs trap worse, so the fix is guidance and tools, not layout. Add one short section to docs (and the yaks skill's "Running yaks"/filtering area, which agents read): prefer `yaks path <filters>` / `yaks list --json`, `rg -l` and `find -exec ... +`, never summarize through `ls | xargs`. Consider whether `yaks path` should stream (it does) and whether a `yaks path --null` (for `xargs -0`) is worth adding.

---
▸ 2026-10-19T04:30:31Z [delta:herd-pass]

Herd pass 2026-10-19: fold into the skills rewrite (yaks-b048) when it happens; it is one short section.

---
▸ 2026-10-22T00:59:43Z [delta:herd-dirs]

Folded into the yaks skill's Filtering section by yaks-b048 (567c64c, refined in 51549a7): the xargs chunking and E2BIG traps, with a single-stream counting recipe. No `path --null`: not needed so far.

---
▸ 2026-10-22T00:59:43Z [delta:herd-dirs]

moved: hairy -> shorn
