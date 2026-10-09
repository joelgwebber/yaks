#!/bin/sh
# Build a synthetic layout that exercises every `yaks discover` case, then run
# `yaks discover` from each kind of anchor. Read-only outside $BASE.
#
#   scripts/discover-fixture.sh [YAKS_BIN] [BASE]
#
# Layout (one repo, origin https://example.com/synth/repo.git):
#   $BASE/repo                               primary checkout, with a farm
#   $BASE/wt-a                               git worktree (outside the repo)
#   $BASE/repo/.wt/b                         git worktree (inside the repo)
#   $BASE/repo/.delta/worktrees/d1/repo      Delta clone, linked layout
#                                            (git dir .delta/clones/d1/repo.git,
#                                            alternates -> repo/.git/objects)
#   $BASE/managed/worktrees/m1/local_u1      Delta clone, managed layout
#                                            (git dir local_u1.git beside it,
#                                            alternates -> managed bare repo,
#                                            origin spelled without .git)
#   $BASE/managed/worktrees/m2/local_u2      managed clone of ANOTHER repo
#   $BASE/managed/worktrees/m3/local_u3      orphaned mount: files, no git
set -eu
Y=${1:-$(cd "$(dirname "$0")/.." && pwd)/target/release/yaks}
BASE=${2:-${TMPDIR:-/tmp}/yaks-discover-fixture}
URL=https://example.com/synth/repo.git
g() { git -c user.name=t -c user.email=t@t -c commit.gpgsign=false -c init.defaultBranch=main "$@"; }

rm -rf "$BASE"
mkdir -p "$BASE"
BASE=$(cd "$BASE" && pwd -P)

# Primary checkout with a committed farm.
g init -q "$BASE/repo"
g -C "$BASE/repo" remote add origin "$URL"
mkdir -p "$BASE/repo/.yaks/hairy"
printf -- '---\nid: s-0001\ntitle: t\n---\n' >"$BASE/repo/.yaks/hairy/s-0001.md"
printf '.wt/\n.delta/\n' >"$BASE/repo/.gitignore"
g -C "$BASE/repo" add -A
g -C "$BASE/repo" commit -qm init

# Plain git worktrees: one beside the repo, one inside it.
g -C "$BASE/repo" worktree add -q -b wt-a "$BASE/wt-a"
g -C "$BASE/repo" worktree add -q -b wt-b "$BASE/repo/.wt/b"

# Linked-layout Delta clone (how Delta lays out a project added from a folder).
mkdir -p "$BASE/repo/.delta/worktrees/d1" "$BASE/repo/.delta/clones/d1"
g clone -q --shared --separate-git-dir "$BASE/repo/.delta/clones/d1/repo.git" \
  "$BASE/repo" "$BASE/repo/.delta/worktrees/d1/repo"
g -C "$BASE/repo/.delta/worktrees/d1/repo" remote rename origin local
g -C "$BASE/repo/.delta/worktrees/d1/repo" remote add origin "$URL"

# Managed-layout Delta clone: a managed bare repo, a checkout named
# local_<uuid> with its git dir beside it, origin as Delta records it.
g clone -q --bare "$BASE/repo" "$BASE/managed/repository.git"
mkdir -p "$BASE/managed/worktrees/m1"
g clone -q --shared --separate-git-dir "$BASE/managed/worktrees/m1/local_u1.git" \
  "$BASE/managed/repository.git" "$BASE/managed/worktrees/m1/local_u1"
g -C "$BASE/managed/worktrees/m1/local_u1" remote rename origin local
g -C "$BASE/managed/worktrees/m1/local_u1" remote add origin "${URL%.git}"

# A managed clone of another repository, and an orphaned mount.
mkdir -p "$BASE/managed/worktrees/m2"
g init -q --separate-git-dir "$BASE/managed/worktrees/m2/local_u2.git" "$BASE/managed/worktrees/m2/local_u2"
g -C "$BASE/managed/worktrees/m2/local_u2" remote add origin https://example.com/synth/other
mkdir -p "$BASE/managed/worktrees/m3/local_u3"
echo x >"$BASE/managed/worktrees/m3/local_u3/README"

run() {
  echo
  echo "################ $1"
  shift
  "$Y" discover --root "$BASE/managed/worktrees" "$@"
}
run "primary checkout" "$BASE/repo"
run "git worktree (outside)" "$BASE/wt-a"
run "git worktree (inside), from a subdirectory" "$BASE/repo/.wt/b/.yaks"
run "Delta clone, linked layout" "$BASE/repo/.delta/worktrees/d1/repo"
run "Delta clone, managed layout" "$BASE/managed/worktrees/m1/local_u1"
run "anywhere (not a repo)" "$BASE"
