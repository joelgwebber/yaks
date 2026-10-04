#!/bin/sh
# land.sh: land a Delta worker's commit by SHA (see yaks-coordinating-delta, "What lands, and how").
#
#   land.sh <worker-sha> [--dry-run] [--force-discard]
#   land.sh --selftest
#
# Delta applies a returning worker's result to your working tree and may leave MERGE_HEAD set, but
# that applied tree is not a trustworthy merge (stale files, reverted files, unfinished edits). The
# commit is. This script: reports the pending state; picks `merge` when the worker's base is still an
# ancestor of HEAD, else `cherry-pick` (history was rewritten); computes the correct merged tree;
# refuses if the working tree has changes the worker's commit does not explain (your edits, or
# Delta's mistakes) unless --force-discard; clears the applied state; lands the commit; and checks
# the result equals the computed tree. It never pushes. Needs git 2.38+ (merge-tree --write-tree).
set -u

say() { printf '%s\n' "$*"; }
die() { printf 'land: %s\n' "$*" >&2; exit "${2:-1}"; }

if [ "${1:-}" = "--selftest" ]; then SELFTEST=1; else SELFTEST=0; fi

land() {
  sha=${1:-}; [ $# -gt 0 ] && shift
  dry=0; force=0
  for a in "$@"; do
    case $a in --dry-run) dry=1 ;; --force-discard) force=1 ;; *) die "unknown argument: $a" 2 ;; esac
  done
  [ -n "$sha" ] || die "usage: land.sh <worker-sha> [--dry-run] [--force-discard]" 2
  git rev-parse --git-dir >/dev/null 2>&1 || die "not inside a git repository"
  full=$(git rev-parse --verify -q "$sha^{commit}") || die "no such commit: $sha"
  short=$(git rev-parse --short "$full")
  subject=$(git log -1 --format=%s "$full")
  git rev-parse --verify -q HEAD >/dev/null || die "no HEAD"

  pending=""
  git rev-parse -q --verify MERGE_HEAD >/dev/null && pending="merge"
  git rev-parse -q --verify CHERRY_PICK_HEAD >/dev/null && pending="${pending:+$pending+}cherry-pick"
  say "worker commit: $short $subject"
  say "pending state: ${pending:-none}"

  parent=$(git log --format=%p -1 "$full" | cut -d' ' -f1)
  [ -n "$parent" ] || die "the worker commit has no parent"
  if git merge-base --is-ancestor "$parent" HEAD; then
    mode=merge
    say "worker base $(git rev-parse --short "$parent") is an ancestor of HEAD: trying a merge by SHA"
  else
    mode=cherry-pick
    say "worker base $(git rev-parse --short "$parent") is NOT an ancestor of HEAD (history was rewritten): cherry-pick"
  fi

  tree=""
  if [ $mode = merge ]; then
    out=$(git merge-tree --write-tree --name-only HEAD "$full" 2>&1)
    if printf '%s\n' "$out" | grep -q '^CONFLICT'; then
      say "the merge would conflict, so landing by cherry-pick instead (you resolve the files named here, then 'git cherry-pick --continue'):"
      printf '%s\n' "$out" | grep '^CONFLICT' | sed 's/^/  /'
      mode=cherry-pick
    else
      tree=$(printf '%s\n' "$out" | head -1)
      say "correct merged tree: $(printf '%s' "$tree" | cut -c1-12)"
    fi
  fi

  tmp=$(mktemp -d) || die "mktemp failed"
  trap 'rm -rf "$tmp"' EXIT
  git diff-tree --no-renames --name-only -r "$parent" "$full" | sort > "$tmp/worker"
  git status --porcelain --untracked-files=all | sed 's/^...//' | sort > "$tmp/dirty"
  comm -13 "$tmp/worker" "$tmp/dirty" > "$tmp/unexplained"
  if [ -s "$tmp/unexplained" ]; then
    say "changes in the working tree that the worker's commit does NOT explain:"
    sed 's/^/  /' "$tmp/unexplained"
    say "(your own uncommitted edits, or Delta's applied state reverting or resurrecting files)"
    if [ $force = 0 ]; then
      [ $dry = 1 ] && { say "dry run: stopping here"; return 0; }
      die "refusing to continue; commit or stash your edits, or rerun with --force-discard" 3
    fi
    say "--force-discard: tracked changes will be discarded"
  fi
  if [ $dry = 1 ]; then say "dry run: would clear the applied state and land $short by $mode"; return 0; fi

  [ -n "$pending" ] && { git merge --abort 2>/dev/null; git cherry-pick --abort 2>/dev/null; }
  git checkout -q -- . || die "could not restore tracked files"
  # Untracked leftovers of the applied state: remove only if identical to the worker's file.
  while IFS= read -r p; do
    [ -n "$p" ] && [ -e "$p" ] && ! git ls-files --error-unmatch -- "$p" >/dev/null 2>&1 || continue
    if git cat-file -e "$full:$p" 2>/dev/null && git show "$full:$p" | cmp -s - "$p"; then
      rm -f -- "$p"
    else
      die "untracked $p is not identical to the worker's version; resolve it by hand" 4
    fi
  done < "$tmp/dirty"

  if [ $mode = merge ]; then
    git merge --no-ff -q -m "Merge worker commit $short ($subject)" "$full" \
      || die "merge stopped; resolve, commit, then run yaks doctor" 5
    if [ "$(git rev-parse 'HEAD^{tree}')" = "$tree" ]; then say "result is identical to the computed merge"
    else say "WARNING: result differs from the computed merge tree"; fi
  else
    git cherry-pick "$full" >/dev/null 2>&1 \
      || die "cherry-pick stopped with conflicts; resolve, then 'git cherry-pick --continue', then run yaks doctor" 5
    say "cherry-picked $short onto HEAD"
  fi
  say "landed: $(git log --oneline -1)"
  top=$(git rev-parse --show-toplevel)
  for y in yaks ./target/release/yaks; do
    if [ -d "$top/.yaks" ] && command -v "$y" >/dev/null 2>&1; then say "doctor: $("$y" doctor 2>&1 | tail -1)"; break; fi
  done
}

selftest() {
  d=$(mktemp -d) || die "mktemp failed"; fails=0
  ok() { say "  ok   $1"; }
  bad() { say "  FAIL $1"; fails=$((fails + 1)); }
  g() { git -C "$d" -c user.name=t -c user.email=t@t "$@"; }
  setup() {
    rm -rf "$d"; mkdir -p "$d"; g init -q -b main
    printf 'root\n' > "$d/root.txt"; g add -A; g commit -q -m root; root=$(g rev-parse HEAD)
    printf 'a\n' > "$d/a.txt"; printf 'c\n' > "$d/c.txt"; g add -A; g commit -q -m base
    base=$(g rev-parse HEAD)
    g checkout -q -b worker; printf 'a2\n' > "$d/a.txt"; mkdir -p "$d/new"; printf 'n\n' > "$d/new/f.txt"
    g add -A; g commit -q -m "worker change"; w=$(g rev-parse HEAD)
    g checkout -q main
  }
  run() { ( cd "$d" && land "$@" ) ; }

  say "1. applied state with a stale untracked leftover; merge by SHA"
  setup; printf 'b\n' > "$d/b.txt"; g add -A; g commit -q -m "main moves"
  g merge -q --no-commit --no-ff "$w" >/dev/null 2>&1; g reset -q          # Delta-like: MERGE_HEAD, unstaged
  out=$(run "$w" 2>&1); rc=$?
  [ $rc = 0 ] && ok "exit 0" || { bad "exit $rc: $out"; }
  [ "$(g rev-list --parents -1 HEAD | wc -w | tr -d ' ')" = 3 ] && ok "a merge commit" || bad "not a merge commit"
  [ "$(cat "$d/a.txt")" = a2 ] && [ -f "$d/new/f.txt" ] && ok "worker change present" || bad "worker change missing"
  [ ! -e "$d/.git/MERGE_HEAD" ] && ok "no MERGE_HEAD" || bad "MERGE_HEAD left"
  printf '%s' "$out" | grep -q "identical to the computed merge" && ok "tree equals merge-tree" || bad "tree check"

  say "2. an unexplained uncommitted edit is refused, and kept"
  setup; printf 'mine\n' > "$d/c.txt"
  out=$(run "$w" 2>&1); rc=$?
  [ $rc = 3 ] && ok "exit 3" || bad "exit $rc"
  [ "$(cat "$d/c.txt")" = mine ] && ok "edit preserved" || bad "edit lost"
  printf '%s' "$out" | grep -q "c.txt" && ok "names the file" || bad "does not name the file"

  say "3. --force-discard drops it and lands"
  out=$(run "$w" --force-discard 2>&1); rc=$?
  [ $rc = 0 ] && [ "$(cat "$d/c.txt")" = c ] && [ "$(cat "$d/a.txt")" = a2 ] && ok "discarded and landed" || bad "rc=$rc: $out"

  say "4. dry run changes nothing"
  setup; printf 'mine\n' > "$d/c.txt"; before=$(g rev-parse HEAD)
  out=$(run "$w" --dry-run 2>&1); rc=$?
  [ $rc = 0 ] && [ "$(g rev-parse HEAD)" = "$before" ] && [ "$(cat "$d/c.txt")" = mine ] && ok "no changes" || bad "rc=$rc"

  say "5. worker base no longer an ancestor: cherry-pick"
  setup; g reset -q --hard "$root"
  printf 'a\n' > "$d/a.txt"; printf 'c\n' > "$d/c.txt"; g add -A; g commit -q -m "base (rewritten)"
  g merge-base --is-ancestor "$base" HEAD && bad "setup: base still an ancestor" || ok "setup: base is not an ancestor"
  out=$(run "$w" 2>&1); rc=$?
  [ $rc = 0 ] && printf '%s' "$out" | grep -q "cherry-pick" && [ "$(cat "$d/a.txt")" = a2 ] && ok "cherry-picked" || bad "rc=$rc: $out"

  rm -rf "$d"
  if [ $fails = 0 ]; then say "selftest: all passed"; return 0; fi
  say "selftest: $fails failure(s)"; return 1
}

if [ $SELFTEST = 1 ]; then selftest; exit $?; fi
land "$@"
