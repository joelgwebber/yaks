#!/bin/sh
# Build synthetic Delta layouts the way Delta does (clone git dirs with
# core.worktree, alternates into a host repo, refs/delta pins in the host),
# then run `yaks discover` from every kind of checkout. Writes only under BASE.
#
#   scripts/discover-fixture.sh [YAKS_BIN] [BASE]
#
# LINKED machine (the project added from a folder; host = the human's checkout):
#   $BASE/repo                              primary checkout, with a farm
#   $BASE/wt-a                              plain git worktree of the primary
#   $BASE/repo/.delta/worktrees/d1/repo     Delta clone (git dir in .delta/clones/d1)
#   $BASE/repo/.delta/worktrees/d2/repo     Delta clone
#   refs/delta/gone1/repo/*                 a finished thread: pinned, clone removed
#   $BASE/repo/.delta/worktrees/zz/repo     look-alike: same shape, NOT pinned
# SHARED machine (thread shared here; host = Delta's managed bare repo):
#   $BASE/managed/repository.git            the managed bare repo
#   $BASE/managed/worktrees/m1/local_u1     Delta clone
#   $BASE/managed/worktrees/m2/proj         Delta clone with a different name
set -eu
Y=${1:-$(cd "$(dirname "$0")/.." && pwd)/target/release/yaks}
BASE=${2:-${TMPDIR:-/tmp}/yaks-discover-fixture}
g() { git -c user.name=t -c user.email=t@t -c commit.gpgsign=false -c init.defaultBranch=main "$@"; }

# delta_clone HOST_GIT_DIR STORE CHECKOUTS DIR NAME
delta_clone() {
  co="$3/$4/$5"; gd="$2/$4/$5.git"
  mkdir -p "$3/$4" "$2/$4"
  g clone -q --shared --separate-git-dir "$gd" "$1" "$co"
  g -C "$co" config core.worktree "$co"
  sha=$(g -C "$co" rev-parse HEAD)
  g --git-dir="$1" update-ref "refs/delta/$4/$5/$sha" "$sha"
}

rm -rf "$BASE"
mkdir -p "$BASE"
BASE=$(cd "$BASE" && pwd -P)

g init -q "$BASE/repo"
g -C "$BASE/repo" remote add origin https://example.com/synth/repo.git
mkdir -p "$BASE/repo/.yaks/hairy"
printf -- '---\nid: s-0001\ntitle: t\n---\n' >"$BASE/repo/.yaks/hairy/s-0001.md"
printf '.delta/\n' >"$BASE/repo/.gitignore"
g -C "$BASE/repo" add -A
g -C "$BASE/repo" commit -qm init
g -C "$BASE/repo" worktree add -q -b wt-a "$BASE/wt-a"

H="$BASE/repo/.git"
delta_clone "$H" "$BASE/repo/.delta/clones" "$BASE/repo/.delta/worktrees" d1 repo
delta_clone "$H" "$BASE/repo/.delta/clones" "$BASE/repo/.delta/worktrees" d2 repo
sha=$(g -C "$BASE/repo" rev-parse HEAD)
g -C "$BASE/repo" update-ref "refs/delta/gone1/repo/$sha" "$sha"
mkdir -p "$BASE/repo/.delta/worktrees/zz"
g clone -q "$BASE/repo" "$BASE/repo/.delta/worktrees/zz/repo"

g clone -q --bare "$BASE/repo" "$BASE/managed/repository.git"
M="$BASE/managed/repository.git"
delta_clone "$M" "$BASE/managed/worktrees" "$BASE/managed/worktrees" m1 local_u1
delta_clone "$M" "$BASE/managed/worktrees" "$BASE/managed/worktrees" m2 proj

run() {
  echo
  echo "################ $1"
  shift
  "$Y" discover "$@" || true
}
run "linked: primary checkout" "$BASE/repo"
run "linked: plain git worktree" "$BASE/wt-a"
run "linked: Delta clone, from a subdirectory" "$BASE/repo/.delta/worktrees/d1/repo/.yaks"
run "shared: Delta clone" "$BASE/managed/worktrees/m1/local_u1"
run "outside any checkout" "$BASE"
