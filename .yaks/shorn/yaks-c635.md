---
id: yaks-c635
title: 'yaks sheds: find Delta sheds from refs/delta pins (alternates -> host repo -> pins -> sibling git dir''s core.worktree), plus the host''s git worktrees; drop the directory scans'
type: task
priority: 3
created: '2026-10-09T03:48:59Z'
updated: '2026-10-09T20:12:45Z'
parent: yaks-7eea
labels:
- cli
- delta
---

Follows yaks-2177 (yaks discover). Its findings 2 and 3: from a linked Delta clone the primary's plain git worktrees are missed (run git worktree list in the repo alternates/local names, when it is a checkout); from a managed clone only the root scan finds siblings. Wire the root scan into sheds::discover; then the -C/--shed reference-shed selector (yaks-7eea).

---
▸ 2026-10-09T20:01:45Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:01:45Z [delta:Delta :Yaks (cont'd)]
Replan (Joel, 2026-10-09): give up on "magical" Delta discovery from outside a checkout. Discover every shed from inside any sibling using git data only:
1. Own git dir: `git rev-parse --absolute-git-dir`.
2. Host repo: the first line of `objects/info/alternates`, whose parent is the host. That is the human's `.git` on a linked machine and Delta's managed bare repo on a shared one. A checkout without alternates is its own host.
3. Every Delta clone of the repo on this machine: `git for-each-ref refs/delta` in the host gives `refs/delta/<dir>/<name>/<sha>`.
4. Resolve each pin: the sibling git dir `<store>/<dir>/<name>.git` (store = the parent of the parent of our own git dir when we are a clone; `<main checkout>/.delta/clones` when the host is a checkout), and its `core.worktree` gives the checkout. If the git dir is missing, the clone is gone.
5. When the host is a checkout, `git worktree list` in it adds the primary and its plain git worktrees. This closes the gap where a clone could not see the primary's git worktrees.
Verified by hand on the Mac (this clone: 37 pinned dirs, 36 gone, 1 live; the j15r managed clones resolve each other despite different names). A probe worker showed that Delta pins a NEW clone's starting commits at creation, before it makes any commit (refs/delta/dv4fmmmdm74z/yaks/* existed for a worker that never committed), so a fresh worker is visible at once.
Remove: the Delta root scan, `--root`, the name-based sibling scan, the alternates/origin matching for Delta candidates, and `yaks discover`'s outside-a-repo mode. `yaks discover` becomes a view of this chain.

![pins-mac](artifacts/yaks-c635/pins-mac.txt)

![pins-fixture](artifacts/yaks-c635/pins-fixture.txt)

---
▸ 2026-10-09T20:12:45Z [delta:Delta :Yaks (cont'd)]
Done. `sheds::shed_paths` is now the only discovery path: git worktrees here; the host from alternates; refs/delta pins resolved through `<store>/<dir>/<name>.git` and core.worktree; and the host checkout's git worktrees when the host is not bare. Removed: the name-based sibling scan, the `.delta/worktrees` scan, the Delta root scan, `--root`, `repo_match`/`same_repo`/`normalize_url`, and `yaks discover`'s outside-a-repo mode (it now exits 1 with "run it from inside a checkout").
Decision: the clone store is the parent of our own git dir's parent when we are a clone, else `<host checkout>/.delta/clones`. That is the one structural fact used, and it only resolves names the pins provide. Rejected alternative: keep a directory scan as a fallback; it would bring back look-alike matching and the "found by layout" ambiguity.
Evidence (attached):
- pins-mac.txt, the real Mac. From this clone the sheds are `~/src/yaks` plus the probe worker's clone, which never committed and was found by its creation pins. From `~/src/yaks` both clones are found. From a shared-layout j15r clone, its differently named sibling is found. From `~` it refuses with an error.
- pins-fixture.txt (scripts/discover-fixture.sh): a linked machine (primary, git worktree, 2 clones, a gone pin, an unpinned look-alike) and a shared machine (bare host, 2 clones with different names). Every anchor finds exactly the right set; the look-alike and the gone pin are never sheds.
Tests: 5 new sheds tests (linked, managed, fresh clone before its first commit, look-alikes ignored, found-by-both is one worktree), 3 discover tests, and the shallow fixtures moved onto pins. Full suite: 485 + 8 + 11 + 28 + 19 + 5 + 3 + 3 + 2 pass.
Docs: docs/cli.md (sheds discovery and the discover row), README.md, the yaks skill table, and the yaks-coordinating-delta model paragraph (which now gives the macOS shared-layout path and says sheds come from pins and are visible from inside any checkout).

---
▸ 2026-10-09T20:12:45Z [delta:Delta :Yaks (cont'd)]
moved: shaving -> shorn
