---
name: yaks-coordinating-private
description: "Farm-mode companion to yaks-coordinating for a PRIVATE or OUT-OF-TREE farm (`.yaks/` gitignored, a `.yaks` pointer file, or a symlink): one live farm shared by every lane, live claims, no yak ids in anything shared, PR-driven landing. Load after yaks-coordinating when `git ls-files .yaks` prints nothing. Repo-internal; not shipped."
---

# Coordinating a private or out-of-tree farm

Read `yaks-coordinating` first. This file is only what changes when the farm is not in git.

## What the farm is
There is **one live farm** for every lane. A gitignored `.yaks/` is never copied into another
checkout; `yaks` finds it by walking UP the filesystem, so any checkout inside the repo's
directory tree reaches the same farm with no setup, and yak surgery is live and shared. There
is no yak state to merge. An out-of-tree farm is reached by a `.yaks` pointer file (`path:` and optional `herd:`, resolved
relative to the pointer's own directory) or a symlink; `ls -ld .yaks` tells them apart (`d` is the
farm itself, `l` a symlink, `-` a pointer file). It is
kept out of git with `.git/info/exclude`; a worktree OUTSIDE the repo tree needs the pointer or
`ln -s <repo>/.yaks <worktree>/.yaks`. `yaks merge` consolidates separate farms.
**Hazard:** the walk-up has no stop at the git top-level, so a checkout that lacks the farm
for any reason reads and WRITES whatever farm sits above it. This is being changed (yaks-b4dc):
discovery will stop at the git top-level unless a `.yaks` pointer file or `YAKS_DIR` names the
farm, and `yaks init --mode private` will write that pointer. Until then the walk-up applies, so
check which farm you are in before the first write.

## Claim
`shave` the yaks and add the assignment notes. They are visible to everyone instantly and there
is **nothing to commit**. The claim commit of team mode does not exist here.

## Live visibility, live asks
A worker's notes, shear and `needs: human` ask appear in your `yaks list`, `show` and `inbox`
the moment they are written, with no landing step. Answer an ask directly in the farm while the
worker is idle, then wake the worker with a one-line message: the note is the record, the message
only the nudge. Do not chase the worker with edits to its yak while it is running.

## One writer per yak
Writes to DIFFERENT yaks are safe at any concurrency. Concurrent updates to the SAME yak can
silently drop a note, because the update is a read-modify-write with no lock (32 parallel
writers lost 4 of 800 notes; yaks-800d). **An append is a write**: `yaks update --note`
rewrites the whole file: two writers read the same version, the second rename replaces the first
writer's note, nothing reports an error and the file stays valid. "It only appends" does not make
two writers safe. So no yak has two writers, and the coordinator does not write a yak a worker is actively using.
The trap is a shared parent yak that several workers report to: give each worker its own yak,
and summarise onto the parent yourself after they finish.

## Nothing about yaks reaches git or the outside
Yak ids and the word "yaks" stay out of commit messages, code, comments, PR titles and bodies,
and external trackers. Workers stage only their own source files and never anything under
`.yaks`; commit messages are plain English ("implement retry"). Preflight every PR body and the
landed commit range with `yaks scan-ids` (non-zero on any real id). With an external tracker, put
the key in with `yaks rollup --keys`, never an id. Evidence attaches to the farm and is not
committed, so paste the key output into notes too.

## Landing
Workers produce committed branches or commits; you land them (squash by default), and you own `gh`
and the PRs, one auth and one privacy checkpoint. After the merge, **stamp the landed SHA onto the
yak** with a note: the branch SHAs a worker knew are orphaned by the squash, and this note is the
only yak-to-commit link, since git cannot know about yaks by design. `yaks commits` finds nothing
here, correctly. Run `yaks doctor` after the batch.

## Keeping the farm safe
- `git clean -fdx` in the outer repo deletes an ignored `.yaks/`. Push or back up the farm.
- Several repos can share one out-of-tree farm: give each repo a pointer with its own `herd:`.
- A farm nested as its own git repo (for sync across machines) must be hidden with
  `.git/info/exclude`, never `.yaks/.gitignore` containing `*` (that blinds the nested repo too).
