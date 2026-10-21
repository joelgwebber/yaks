---
id: yaks-b1e9
title: 'yaks init --mode branch [--branch <name>]: start a new farm on an orphan branch at farm_home'
status: shorn
type: feature
priority: 3
created: '2026-10-18T05:15:36Z'
updated: '2026-10-21T22:49:46Z'
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

---
▸ 2026-10-21T22:10:52Z [init-1]

Start (init-1): 2026-10-21T22:10:49Z
- `yaks path yaks-b1e9`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-b1e9.md

---
▸ 2026-10-21T22:20:25Z [init-1]

Implemented `yaks init --mode branch [--branch <name>]` (src/init.rs `ensure_branch_farm`, `Mode::Branch`; main.rs `Init` args + dispatch; branchfarm `git`/`ref_exists` made `pub(crate)`).
- Tests (tests/init_modes.rs, 8 `branch_mode_*`): all failed before (`invalid value 'branch' for '--mode'`), pass now: fresh orphan worktree at farm_home + `yaks path` + `yaks create`; re-run no-op; `--branch personal-joel` records `yaks.branch`; existing `origin/yaks` only gets its worktree (history stays 1 commit); in-tree farm refused; `--shared` clone refused; flag misuse (`--path`, `--branch` without mode, bad name, code branch); skills land in the code checkout, not the farm.
- Real run in scratch repo (temp XDG_DATA_HOME/HOME), saved verbatim at /tmp/init-1-real-run.txt. Key lines: `branch: created the orphan branch `yaks` at <tmp>/data/yaks/farms/code-de84654d`; second run ends `Nothing to change: already set up this way.`; `yaks path` from the code checkout -> `<tmp>/data/yaks/farms/code-de84654d/.yaks`; `yaks create --title hello` -> `Created yak-28b9: hello`; `git worktree list` shows the farm worktree on `[yaks]` (unborn, 0000000); code `git status --short` empty; second repo `--branch personal-joel` printed `config: set yaks.branch = personal-joel`.
- Gate: `cargo test --release` all suites green (unit 785 passed/8 ignored, init_modes 27 passed).

---
▸ 2026-10-21T22:20:25Z [init-1]

Decision: a Delta/`--shared` clone is refused only when init would have to CREATE a worktree (new branch, or existing branch with no worktree). An existing worktree is reported "unchanged" (re-run no-op works from a clone). Rejected: refusing every run from a clone (breaks idempotent re-runs, nothing is changed anyway).
- Decision: new branch = `git worktree add -q --orphan -b <b> <farm_home>`, left UNBORN (no commit), as the claim says; idempotence checks `worktree_of_branch` (which lists unborn branches) before refs. Rejected: an initial empty commit (claim: do not commit).
- Decision: an existing branch must hold `.yaks/` at its tip (`git cat-file -e <tip>:.yaks`) else refuse ("not a farm; pick another with --branch"); also refuse a name equal to the checked-out code branch, and run in a farm worktree itself. Rejected: reusing blindly (`--branch main` would make a worktree of code).
- Decision: reuse `branchfarm::bootstrap` for an existing branch (passing the already computed farm_home), and `ensure_farm` for the config clash check (`--herd other` on an existing farm errors, like the other modes). Made `branchfarm::git`/`ref_exists` `pub(crate)` rather than duplicating them.
- Decision: `--branch` without `--mode branch` is an error (not silently ignored), and `--branch` alone no longer counts as plain init. `--mode branch --path` errors naming farm_home.
- Decision: `yaks.branch` is written only when it differs from `configured_farm_branch` (same rule as migrate), after the worktree exists. Skills in branch mode are ordinary project files in the code checkout (no exclude lines), like team mode. A `.yaks/` holding only leftovers (.lock/.gitignore) is not an in-tree farm.
- Decision: docs: cli.md row + init section paragraph, README/docs/README mode lists, clap help, yaks skill init sentence + command-table row (kept to the init clause; yaks-b048 owns the rewrite).

---
▸ 2026-10-21T22:24:22Z [init-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-21T22:24:29Z [init-1]

moved: shaving -> shorn

---
▸ 2026-10-21T22:49:46Z [delta:herd-dirs]

Landed (delta:herd-dirs): ad0165d (single-parent, Squashed-from 2a3af01). merge_worktrees' 3-way merge from the worker's older base rolled back d138472 in store.rs/AGENTS.md/docs/cli.md/SKILL.md; restored those from HEAD and applied only the worker's hunks, then re-ran the gate (20 suites).
