---
id: yaks-289a
title: 'Directory sources: relative links in the joined text miss the files/ prefix'
status: shorn
type: bug
priority: 3
created: '2026-11-17T21:16:48Z'
updated: '2026-11-18T00:06:15Z'
parent: yaks-4cfe
depends_on:
- yaks-535c
- yaks-e83f
tags:
- doks
- links
---

Part of yaks-4cfe. A directory source's joined `extracted.md` copies each file's text under `## <path>`, and leaves its relative links as written for the file's own location. In YASS (`sources/2026-11-05-04642f1f-...`), README's `[docs/monorepo.md](docs/monorepo.md)` sits in `extracted.md` at the source root, where `docs/monorepo.md` does not exist. The file is `files/docs/monorepo.md`. A link from a nested file (`docs/x.md` linking `../README.md`) is wrong in the same way.

**Moot if yaks-535c drops the joined copy** (its lean). If it stays, `source add` rewrites each relative link in a joined file to its real path, `files/<dir of that file>/<link>`, normalised. It leaves absolute, `#anchor` and scheme links alone. `yaks doctor` should then report none of these.

#### 2026-11-17T22:54:06Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T00:06:13Z

*sources-triage*

Closed by yaks-e83f (`sources-triage`). A directory source's tree now sits directly in `sources/<name>/`, with no `files/` level and no joined copy, so the tree's own relative links are simply right.

Shown on a migrated copy of this farm: YASS's `README.md` links `[docs/monorepo.md](docs/monorepo.md)`, and `docs/monorepo.md` is beside it. A link from a dok to a joined-text heading (`extracted.md#docs-monorepomd`) is rewritten by the migration to that file of the tree.

#### 2026-11-18T00:06:15Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
