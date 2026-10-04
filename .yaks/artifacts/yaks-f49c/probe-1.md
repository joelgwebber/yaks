# probe-1 output (yaks-f49c)
start: Sun Oct  4 07:29:08 PM UTC 2026
first note: ls .yaks/*/yaks-f49c.md => .yaks/shaving/yaks-f49c.md

$ pwd
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks
[exit 0]

$ cat .git
gitdir: /home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks.git
[exit 0]

$ git rev-parse --git-dir --git-common-dir --show-toplevel
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks.git
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks.git
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks
[exit 0]

$ git config --list --local
core.repositoryformatversion=0
core.filemode=true
core.bare=false
core.logallrefupdates=true
core.alternaterefscommand=: "$@"
core.worktree=/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks
remote.origin.url=https://github.com/joelgwebber/yaks.git
remote.origin.fetch=+refs/heads/*:refs/remotes/origin/*
branch.main.remote=origin
branch.main.merge=refs/heads/main
remote.local.url=/home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git
remote.local.fetch=+refs/heads/*:refs/remotes/local/*
remote.local.receivepack=git -c receive.denyCurrentBranch=updateInstead receive-pack
[exit 0]

$ cat $(git rev-parse --git-dir)/objects/info/alternates
/home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git/objects
[exit 0]

$ git remote -v
local	/home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git (fetch)
local	/home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git (push)
origin	https://github.com/joelgwebber/yaks.git (fetch)
origin	https://github.com/joelgwebber/yaks.git (push)
[exit 0]

$ git worktree list
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks.git e4c2bf5 [main]
[exit 0]

$ git log --oneline -3
e4c2bf5 yaks-f49c: file and claim the worker-view probe for cross-machine threads
5301dac yaks-99ec, yaks-df61: file cross-machine Delta threads (O50)
4fc959e yaks-5c9f: land.sh, the Delta landing procedure as a tested script; selftest runs in cargo test
[exit 0]

$ git status -sb
## main...origin/main [ahead 2]
[exit 0]
## 2

$ git for-each-ref refs/delta
[exit 0]
alt repo: /home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git

$ git -C /home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git rev-parse --is-bare-repository
true
[exit 0]

$ git -C /home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git for-each-ref refs/delta
4fc959ebf0f6009749f618be330ed6352ac7f17e commit	refs/delta/3d4gr26p09ng/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
e4c2bf52e404fb3bf5f18751a1d069610aad0bde commit	refs/delta/3d4gr26p09ng/yaks/e4c2bf52e404fb3bf5f18751a1d069610aad0bde
04e75a353eb722a7aa084b4251a00dfa5ec0a173 commit	refs/delta/kb369frnzkrr/yaks/04e75a353eb722a7aa084b4251a00dfa5ec0a173
4fc959ebf0f6009749f618be330ed6352ac7f17e commit	refs/delta/kb369frnzkrr/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
4fc959ebf0f6009749f618be330ed6352ac7f17e commit	refs/delta/p4m7aet4wa31/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
5301dac96a5edfd91668e8e21e00bf5b4805b8df commit	refs/delta/p4m7aet4wa31/yaks/5301dac96a5edfd91668e8e21e00bf5b4805b8df
650d041652bb3fdc23bb8290600abba9effe2e20 commit	refs/delta/p4m7aet4wa31/yaks/650d041652bb3fdc23bb8290600abba9effe2e20
8d2546fb52b5f00f1c19764c4a08239413e5bd70 commit	refs/delta/p4m7aet4wa31/yaks/8d2546fb52b5f00f1c19764c4a08239413e5bd70
e4c2bf52e404fb3bf5f18751a1d069610aad0bde commit	refs/delta/p4m7aet4wa31/yaks/e4c2bf52e404fb3bf5f18751a1d069610aad0bde
[exit 0]
## 3

$ ls -la ..
total 0
drwxr-xr-x 1 joel joel  24 Oct  4 15:28 .
drwxr-xr-x 1 joel joel 144 Oct  4 15:28 ..
drwxr-xr-x 1 joel joel 274 Oct  4 15:28 yaks
drwxr-xr-x 1 joel joel 150 Oct  4 15:29 yaks.git
[exit 0]

$ ls -la ../..
total 0
drwxr-xr-x 1 joel joel 144 Oct  4 15:28 .
drwxr-xr-x 1 joel joel 430 Oct  4 09:47 ..
drwxr-xr-x 1 joel joel  24 Oct  4 15:28 3d4gr26p09ng
drwxr-xr-x 1 joel joel  28 Oct  4 10:05 7mpvztrz9hcq
drwxr-xr-x 1 joel joel  28 Oct  4 15:28 8d660yj5zmcn
drwxr-xr-x 1 joel joel  28 Oct  4 10:17 d3ze93g0qb52
drwxr-xr-x 1 joel joel  24 Oct  4 10:17 kb369frnzkrr
drwxr-xr-x 1 joel joel  24 Oct  4 10:05 p4m7aet4wa31
[exit 0]

$ ls -ld /home/joel/.local/share/delta/worktrees/p4m7aet4wa31/yaks
drwxr-xr-x 1 joel joel 274 Oct  4 12:47 /home/joel/.local/share/delta/worktrees/p4m7aet4wa31/yaks
[exit 0]

$ ls /home/joel/.local/share/delta/worktrees/p4m7aet4wa31/yaks
AGENTS.md
Cargo.lock
Cargo.toml
docs
LICENSE
npm
README.md
RELEASING.md
rust-toolchain.toml
scripts
src
target
tests
[exit 0]

$ ls -l /home/joel/.local/share/delta/worktrees/p4m7aet4wa31/yaks/target/release/yaks
-rwxr-xr-x 2 joel joel 4494680 Oct  4 12:47 /home/joel/.local/share/delta/worktrees/p4m7aet4wa31/yaks/target/release/yaks
[exit 0]
## 4

$ env | grep -E '^(DELTA|YAKS)' | sed 's/=.*/=<value elided>/'
DELTA_THREAD_TITLE=<value elided>
DELTA_CURRENT_THREAD_ID=<value elided>
DELTA_SCRATCH_DIR=<value elided>
DELTA_DATABASE_DIR=<value elided>
[exit 0]

$ echo "DELTA_THREAD_TITLE=$DELTA_THREAD_TITLE"
DELTA_THREAD_TITLE=Probe worker view on shared-thread machine
[exit 0]
## 5

$ git fetch local
[exit 0]

$ git branch -r
  origin/HEAD -> origin/main
  origin/docs
  origin/main
[exit 0]

$ git push --dry-run local HEAD:refs/heads/probe-1-dryrun
To /home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git
 * [new branch]      HEAD -> probe-1-dryrun
[exit 0]
## 6

$ YAKS_ACTOR=probe-1 ./target/release/yaks path yaks-f49c
/home/joel/.local/share/delta/worktrees/3d4gr26p09ng/yaks/.yaks/shaving/yaks-f49c.md
[exit 0]

$ YAKS_ACTOR=probe-1 ./target/release/yaks preflight
preflight: ok
[exit 0]

$ YAKS_ACTOR=probe-1 ./target/release/yaks doctor
All clear: no farm-integrity issues found.
[exit 0]

$ YAKS_ACTOR=probe-1 ./target/release/yaks list --status shaving
  [S] yaks-5c9f  p2 task     Restructure coordinator/worker skills by farm mode (public / private / out-of-tree) [skills] (deps: yaks-80d5)
  [S] yaks-64c4  p4 idea     Yak archaeology: fast retrieval of past reasoning & provenance stitching [cli,search]
  [S] yaks-77a5  p3 task     Case study: reconstruct the 2026-09-23 UI lightning round (11 lanes + 3 scouts) [ui]
  [S] yaks-b5a0  p2 task     Parallel development: finish and harden coordinator/worker skills + tooling (git worktrees, Delta) [agent,skills]
  [S] yaks-df61  p2 task     Delta observation log: how threads, worktrees, clones and the farm interact [delta,agent]
  [S] yaks-f04d  p3 feature  Known labels in configuration [config,labels]
  [S] yaks-f49c  p3 task     Probe: a worker's clone, remotes and sibling visibility on a shared-thread machine [delta,agent]
[exit 0]
## extra (read-only follow-ups to explain section 5)

$ git for-each-ref refs/remotes
4fc959ebf0f6009749f618be330ed6352ac7f17e commit	refs/remotes/origin/HEAD
02d95a3ef0514e90e0982af9dcaae74e599ba79d commit	refs/remotes/origin/docs
4fc959ebf0f6009749f618be330ed6352ac7f17e commit	refs/remotes/origin/main
[exit 0]

$ git ls-remote local
4fc959ebf0f6009749f618be330ed6352ac7f17e	refs/delta/3d4gr26p09ng/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
e4c2bf52e404fb3bf5f18751a1d069610aad0bde	refs/delta/3d4gr26p09ng/yaks/e4c2bf52e404fb3bf5f18751a1d069610aad0bde
04e75a353eb722a7aa084b4251a00dfa5ec0a173	refs/delta/kb369frnzkrr/yaks/04e75a353eb722a7aa084b4251a00dfa5ec0a173
4fc959ebf0f6009749f618be330ed6352ac7f17e	refs/delta/kb369frnzkrr/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
4fc959ebf0f6009749f618be330ed6352ac7f17e	refs/delta/p4m7aet4wa31/yaks/4fc959ebf0f6009749f618be330ed6352ac7f17e
5301dac96a5edfd91668e8e21e00bf5b4805b8df	refs/delta/p4m7aet4wa31/yaks/5301dac96a5edfd91668e8e21e00bf5b4805b8df
650d041652bb3fdc23bb8290600abba9effe2e20	refs/delta/p4m7aet4wa31/yaks/650d041652bb3fdc23bb8290600abba9effe2e20
8d2546fb52b5f00f1c19764c4a08239413e5bd70	refs/delta/p4m7aet4wa31/yaks/8d2546fb52b5f00f1c19764c4a08239413e5bd70
e4c2bf52e404fb3bf5f18751a1d069610aad0bde	refs/delta/p4m7aet4wa31/yaks/e4c2bf52e404fb3bf5f18751a1d069610aad0bde
4fc959ebf0f6009749f618be330ed6352ac7f17e	refs/remotes/origin/HEAD
02d95a3ef0514e90e0982af9dcaae74e599ba79d	refs/remotes/origin/docs
4fc959ebf0f6009749f618be330ed6352ac7f17e	refs/remotes/origin/main
e7fb837c816590380c93dfd2466f0a4cfdbac3ee	refs/tags/v0.0.1
4ad0a117e56fc2431c1345cf9086faf009f7f2d3	refs/tags/v0.0.1^{}
0caf27ad91e693b2621086322c1c342f5c5c5fcb	refs/tags/v0.0.10
f38acd897bf00af9974614e0fa33365bf51fca0f	refs/tags/v0.0.10^{}
f16182771bcaed2f6e0edc762f45756587bb1c8e	refs/tags/v0.0.11
6cd797122397543595d8117157602a65a505e4ca	refs/tags/v0.0.12
35f6960fa9b9c8507c2386347559a3b1282d9625	refs/tags/v0.0.2
c83c3a56fa95d779b28726bb95ae1a15ab121db2	refs/tags/v0.0.2^{}
e24ef39249bc7571a89f342bccfd06b5f1b39d79	refs/tags/v0.0.3
0dc5feb62fbd235057eb0a687e018af2df5940be	refs/tags/v0.0.3^{}
5237a031d1b3db09b6f5edf932e7dd3a57c67fc6	refs/tags/v0.0.4
ccf0623e809805e84acde9d3b569fcacae1f88d1	refs/tags/v0.0.4^{}
d68bd39d69962b742815d8969911cc2fb17ca208	refs/tags/v0.0.5
c47a7866a5ce498ce0044bc47a5e6b21652d18c4	refs/tags/v0.0.5^{}
d6f44cd9a1210457a2aa17eecb873eaccaa66320	refs/tags/v0.0.6
d7ff4ed7963a099b9eccc6c39d1af14794bab21c	refs/tags/v0.0.7
230d5613e92d80410a9b2976fff3df1c0528ff03	refs/tags/v0.0.8
2c607393dd09d6662e3db0ae516537db08267ddc	refs/tags/v0.0.8^{}
6dc5db10a9d69e771c88b69013927388215bfe2b	refs/tags/v0.0.9
2b59ee3032b1777c3e363b48eec358dd2b860f18	refs/tags/v0.0.9^{}
[exit 0]

$ git -C /home/joel/.local/share/delta/user_01jrxs42eest3dk5825m868ym9/managed-repositories/577a78bf-4891-4acd-9008-84073cc6a744/repository.git for-each-ref --count=10 refs/heads
[exit 0]

$ git rev-parse --verify local/main
fatal: Needed a single revision
[exit 128]

$ git --version
git version 2.56.0
[exit 0]

## 7 Surprises (vs yaks-coordinating-delta)
- Layout: the skill says clone `<repo>/.delta/clones/<id>/<repo>.git` and checkout `<repo>/.delta/worktrees/<id>/<repo>`. On this machine the checkout is `~/.local/share/delta/worktrees/<id>/yaks` and the clone is its SIBLING `.../worktrees/<id>/yaks.git` (`.git` is a `gitdir:` file; `core.worktree` is set; `git worktree list` shows the git dir itself as the worktree). There is no `.delta/` anywhere.
- `local` is NOT the user's checkout here: it is a Delta managed bare repo (`.../user_*/managed-repositories/<uuid>/repository.git`, `rev-parse --is-bare-repository` = true), also wired as the clone's `objects/info/alternates`. It holds no `refs/heads/*` at all (only `refs/delta/*` pins), so `git fetch local` is a silent no-op, `git ls-remote local` lists only `refs/delta/*`, a copy of `refs/remotes/origin/*` and `refs/tags/*` (no `refs/heads/*`), `refs/remotes/local/*` never exists and `local/main` does not resolve. The skill's "git fetch local, then git merge local/main" and "git push local <branch>:main" have no base to merge with here; the dry-run push of HEAD to a new branch is accepted (it would create the first branch in the managed repo).
- All sibling threads' pins live in the one managed repo as `refs/delta/<thread-dir>/yaks/<sha>` (3d4gr26p09ng = me, p4m7aet4wa31 = coordinator, kb369frnzkrr another); a worker can enumerate them from its own clone via the alternates path, though its own `git for-each-ref refs/delta` is empty.
- Isolation is nil: sibling checkouts are plain directories under the same parent (`../..`); I could `ls` the coordinator's checkout and `ls -l` its `target/release/yaks` (4494680 bytes). I did not try writing to them. Separately, `core.alternaterefscommand=: "$@"` is set in the clone config (I did not test its effect; my clone's own `refs/delta` is empty and the alternate's refs are not advertised as local refs, which is consistent with it hiding them).
- `DELTA_THREAD_TITLE` = "Probe worker view on shared-thread machine" (not the worker name), so un-prefixed yaks notes would be stamped `delta:Probe worker view...`. `main` shows `[ahead 2]` of origin/main: the coordinator's two commits (claim of this probe, filing) were already in my snapshot as commits, i.e. the clone received the coordinator's committed history.
