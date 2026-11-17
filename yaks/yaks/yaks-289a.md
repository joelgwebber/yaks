---
id: yaks-289a
title: 'Directory sources: relative links in the joined text miss the files/ prefix'
status: hairy
type: bug
priority: 3
created: '2026-11-17T21:16:48Z'
updated: '2026-11-17T21:16:48Z'
parent: yaks-4cfe
tags:
- doks
- links
---

Part of yaks-4cfe. A directory source's joined `extracted.md` copies each file's text under `## <path>`, and leaves its relative links as written for the file's own location. In YASS (`sources/2026-11-05-04642f1f-...`), README's `[docs/monorepo.md](docs/monorepo.md)` sits in `extracted.md` at the source root, where `docs/monorepo.md` does not exist. The file is `files/docs/monorepo.md`. A link from a nested file (`docs/x.md` linking `../README.md`) is wrong in the same way.

**Moot if yaks-535c drops the joined copy** (its lean). If it stays, `source add` rewrites each relative link in a joined file to its real path, `files/<dir of that file>/<link>`, normalised. It leaves absolute, `#anchor` and scheme links alone. `yaks doctor` should then report none of these.
