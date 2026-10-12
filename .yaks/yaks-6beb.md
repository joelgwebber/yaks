---
id: yaks-6beb
title: 'Delta: yaks sheds from a primary that is not the Delta host (managed repos) finds no sheds'
status: hairy
type: task
priority: 4
created: '2026-10-10T15:44:44Z'
updated: '2026-10-10T18:55:33Z'
labels:
- cli
- delta
---

Found 2026-10-10 on Joel's machine with ~/src/j15r. `yaks sheds` there prints `No other checkouts found.`, and `yaks discover` shows 0 pinned dirs. The same command run from inside one of its Delta clones finds the other four (and reports one gone). Nothing is broken: this is a layout the git-only chain (yaks-c635) cannot see, and the evidence the earlier "no discovery from outside" decision asked for.

## What the machine has
- ~/src/j15r is a plain primary checkout: no `.delta/`, no `refs/delta/*` in its `.git`, no alternates, only `origin`.
- Its Delta threads are MANAGED (Delta imported the repo by URL; it never adopted the folder): each clone lives under `~/Library/Application Support/delta/worktrees/<dir>/j15r`, with git dir `j15r.git` beside it, `objects/info/alternates` naming `.../delta/user_*/managed-repositories/<uuid>/repository.git` (a bare repo), and `local` pointing at that bare repo, not at ~/src/j15r. The pins are all in the bare repo.
- So the clones find each other through the bare host (and `yaks discover` there prints the chain), but nothing, in git, connects ~/src/j15r to that host: it is the same project only by `origin` URL (`https://github.com/joelgwebber/j15r.git` there, `https://github.com/joelgwebber/j15r` in the clones: trailing `.git` differs).
- `delta worktree list` is empty and `delta thread list-worktrees` takes a shared thread id, so the Delta CLI gives no thread-to-path map either.

## Consequence
The human's own checkout is the one place the human is likely to run `yaks sheds`, and on a managed repo it shows nothing. Two limits now exist, mirror images: from a clone on a shared machine the human's checkout is not visible (documented), and from the human's checkout the managed clones are not visible (not documented; this yak).

## Options (not decided)
A. Name the host once: `yaks sheds --host <bare repo path>` or a config / `YAKS_HOST` / git config `yaks.host` in the primary (a derived, never-committed setting, like `yaks.shed.title`). The pins and `core.worktree` chain then work unchanged; the clone store for a bare host needs the same `<dir>/<name>.git` convention (the clones sit in `worktrees/<dir>/`, not under the host).
B. Let a clone teach the primary: when a yaks command runs in a managed clone it records `yaks.host` into the primary found by origin URL. Rejected earlier as a registry; weaker here, but it needs a primary to find.
C. Look in Delta's known managed-repositories directory and match `remote.origin.url` (normalized). This is the root scan Joel dropped; here it would be used only to find the HOST, never the clones. Needs his decision to revisit.
D. Document the limit and the workaround: run `yaks -C <any clone> sheds`, which lists the other clones (the primary is then the missing one).

Check first: whether Delta can adopt ~/src/j15r as the project folder (then it becomes the host, pins appear there, and nothing here is needed): that is a Delta setting, not ours.

---
▸ 2026-10-10T18:55:33Z [delta-lead]
Decision (Joel, 2026-10-10): promote to a top-level, low-priority yak ("make this better in Delta"), out from under yaks-7eea. Today it may be unfixable from yaks alone: nothing in git links a plain primary checkout to Delta's managed bare host, and the Delta CLI has no thread-to-path map. What is workable now: `yaks -C <any clone> sheds` (option D), or naming the host once (option A, `yaks.host`), neither built. Joel is first finding out why ~/src/j15r came up managed at all (the thread was created on one machine, in the Delta UI, so a linked host was expected); if that is a Delta setup issue, this yak shrinks to documentation or disappears. Do not investigate further until he reports. A Delta-side request ("the folder is the host", or a way to ask Delta for a thread's checkout paths) belongs here.
