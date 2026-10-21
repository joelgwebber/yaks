---
id: yaks-b1e9
title: 'yaks init --mode branch [--branch <name>]: start a new farm on an orphan branch at farm_home'
status: shaving
type: feature
priority: 3
created: '2026-10-18T05:15:36Z'
updated: '2026-10-21T22:10:10Z'
parent: yaks-6a34
labels:
- cli
---

Tooling for the setup yaks-6a34 does by hand: `yaks init --mode live [--branch farm] [--path ~/src/<repo>-farm]` creates the branch and the sparse worktree (idempotent, like the other modes), writes `live_branch:` into config and prints the result; `yaks farm status` says where the live farm is, whether `farm` is pushed/behind `origin/farm`, and the uncommitted drift count (the two facts needed before a machine switch). Consider a per-user registry (`~/.config/yaks/farms`, keyed by `remote.origin.url`) only if `~/`-relative `YAKS_DIR` proves insufficient on a machine whose host is Delta's bare repo.

---
▸ 2026-10-19T04:28:53Z [delta:herd-pass]

Rescoped in the 2026-10-19 herd pass: 'live' and the sparse worktree are gone (yaks-73f4). Today a new farm needs `git worktree add --orphan -b yaks <path>` by hand; `yaks init --mode branch` should do it at store::farm_home (the same place migrate and the first-use bootstrap, yaks-a260, use), idempotently. `yaks farm status`: the farm worktree, its branch, ahead/behind origin, uncommitted yak count. Probably the default `yaks init` mode once the skills recommend it (yaks-b048).

---
▸ 2026-10-21T19:54:36Z [delta:herd-dirs]

2026-10-21: the 'pushed / behind origin' half of `yaks farm status` is part of the sync design, yaks-472e (question 8).

---
▸ 2026-10-21T22:10:10Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `init-1`. Rescoped: `yaks farm status` is dropped (covered by `yaks status`'s sync line + `yaks path`); Joel (yaks-472e, answer 5): a farm's branch name is chosen at creation.

**Goal:** `yaks init --mode branch [--branch <name>] [--herd <h>] [--skills ...]` in a code checkout creates the farm on an orphan branch in its own worktree at `store::farm_home`, so a new repo never needs hand git.

**Scope (`src/init.rs` + the init clap args in `src/main.rs`):**
- New `Mode::Branch`. Steps, idempotent like the other modes (re-run adds only what is missing, prints one line per step):
  1. Refuse inside a Delta/--shared clone (as `branchfarm::bootstrap` does) and when the checkout already has an in-tree `.yaks/` farm (point at `yaks migrate --to-branch`).
  2. Branch name: `--branch`, else `git config yaks.branch`, else `yaks`. A non-default name is recorded with `git config yaks.branch <name>` (as `branchfarm` does).
  3. If the branch exists (local or `origin/<b>`): ensure its worktree exists (reuse `branchfarm::bootstrap` when there is none), print where, done (a farm already exists).
  4. Else `git worktree add --orphan -b <b> <farm_home>` and `store::init` the farm in `<worktree>/.yaks` with the herd/type/priority flags; do NOT commit or push (print: `yaks sync` publishes it; `yaks private` keeps it local).
  5. `--skills`: as the other modes (project-local in the code checkout).
- `yaks init` with no mode keeps its current behaviour (do not change the default).
- `--path` stays pointer-only; `--mode branch` with `--path` is an error (the worktree location is farm_home; say so).
- Docs: `docs/cli.md` init row, clap help for `--mode`/`--branch`, the yaks skill's init paragraph (one clause; the skills rewrite is yaks-b048).

**Out of scope:** sync, private, migrate, discovery.

**Evidence:** tests in a scratch repo (with XDG_DATA_HOME in a temp dir): fresh init creates the worktree at farm_home on an orphan branch whose tree is only `.yaks/`, `yaks path` from the code checkout resolves it; re-run is a no-op; `--branch personal-joel` records `yaks.branch`; an existing `origin/yaks` only gets its worktree; an in-tree farm and a `--shared` clone are refused. A real run in a scratch repo pasted in a note.

**Judge:** the yakherd.

---
▸ 2026-10-21T22:10:10Z [delta:herd-dirs]

moved: hairy -> shaving
