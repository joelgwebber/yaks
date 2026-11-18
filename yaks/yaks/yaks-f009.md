---
id: yaks-f009
title: 'Commits phase 4: writers emit host commit links; yaks update --commit'
status: shorn
type: feature
priority: 2
created: '2026-11-18T03:03:10Z'
updated: '2026-11-18T15:00:28Z'
parent: yaks-3515
depends_on:
- yaks-274c
tags:
- cli
- git
---

Phase 4 of yaks-3515; needs phase 1 (yaks-274c).

- **`yaks land`:** the landed note's body names the commit as a host commit URL link (`[230f026e](https://github.com/o/r/commit/<sha>)`) when the code repo has a known host, else as `commit:<sha>`.
- **`yaks update <id> --commit [<rev>]`:** default `HEAD` in the code checkout. It records `commit: <full sha>` in the note's metadata, refuses a rev that does not resolve, and, as yaks-ab81 asks, warns or refuses when the commit is not on any remote branch. It needs `--note`, like `--meta`.
- **Docs and skills:** the checkpoint convention in `yaks-working` and `yaks-coordinating` uses `--commit` instead of `--meta commit=$(git rev-parse HEAD)`.

#### 2026-11-18T14:40:33Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T14:41:37Z

*commits-cli*

Start 2026-11-18T14:41:37Z. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-f009.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/1akb8qzsr51j/yaks

#### 2026-11-18T14:47:41Z

*commits-cli*

Decision: unpushed commit with a remote that lacks it is REFUSED (nothing written), naming `git push <remote> <branch>` and `git fetch <remote>`; a checkout with NO remote only WARNS and records. Rejected: always warn (agents ignore warnings: yaks-ab81's whole complaint); always refuse (a local-only repo could never use `--commit`; and there is nothing to push to). Offline is not special-cased: the check reads the remote-tracking refs as last fetched/pushed and never touches the network, so offline works, and "pushed from elsewhere" needs a `git fetch` first (said in the error). Rejected: a fresh `git fetch` before checking (network in a writer, slow/hangs offline).

#### 2026-11-18T14:47:42Z

*commits-cli*

Decision: "which remote" is the existing rule `gitrepo::remote_of(branch)` (branch.<b>.remote, else origin) via a new helper `gitrepo::pushed` -> `Pushed::{Yes,No,NoRemote}` placed after `remote_has`. Rejected: checking every remote (a checkpoint on a fork nobody reads is not safe); reusing `hosting_repo` (it also errors on unrecognised hosts, which is irrelevant to "is it pushed").

#### 2026-11-18T14:47:43Z

*commits-cli*

Decision: `--commit` is `num_args=0..=1` with default `HEAD`, so `update <id> --commit` works; side effect: `update --commit <id>` would read the id as the rev, so the help and docs say to put the id first. Rejected: `--commit` as a flag plus a separate `--rev`/`--commit-rev` (two flags for one intent). Conflicts with `--meta commit=` are refused in `resolve_commit_arg` (not clap `conflicts_with`, since `--meta` values are key=value strings clap cannot inspect). `--commit` applies to each id given to update (same note on all), consistent with `--meta`.

#### 2026-11-18T14:47:43Z

*commits-cli*

Decision: `yaks land` body is `landed: [<short>](<link>) <subject>`; link = host commit URL when `gitrepo::hosting_repo(top, branch)` resolves (any recognised forge, incl. GitLab/Gitea via `commit_url`), else `commit:<full>`. Any hosting error (no remote, unknown host) falls back silently to `commit:`, because `land` never fails on recording. The pushed-ness of the landed commit is not checked in `land` (it just landed locally; `yaks url` already notes unpushed links). Meta fields commit/source/branch unchanged. Old notes keep their plain-text body.

#### 2026-11-18T14:47:44Z

*commits-cli*

Progress: implemented (src/gitrepo.rs `pushed`, src/land.rs body link, src/main.rs `--commit` + `resolve_commit_arg`), tests added in tests/commits.rs (land with host remote; update --commit full cycle), docs/cli.md (update, Note metadata, commits, land), README row, skills yaks/yaks-working/yaks-coordinating, brief.rs. Real runs in /tmp/f009-scratch (origin = git@github.com:o/r.git with pushurl to a bare repo):

- unpushed: `Error: --commit: a2f64f4 is on no branch of origin as last fetched; push it first (`git push origin main`; `git fetch origin` if it was pushed from elsewhere), then record it` (exit 1)
- bad rev: `Error: --commit: "nope" is not a commit in /private/tmp/f009-scratch/repo` (exit 1); no note: `--commit needs a note to attach to ...`; with `--meta commit=`: `--commit and `--meta commit=...` both set `commit`; use one`
- after push: `Updated t-3d2a: note /t/yaks/t-3d2a.md#2026-11-18t144524z`, header `*commits-cli · commit: a2f64f4630680dad4a217cea83ef1ce1d29c00f8*`
- no remote: `warning: a2f64f4 cannot be on a remote: .../repo has none; nobody on another machine can fetch it`, then recorded.
- land, no remote: `landed: [512b1bf](commit:512b1bff13e8ad7f9ae677c72f2aff99512f8e86) t-3d2a: w1`
- land, github remote: `landed: [fa183a8](https://github.com/o/r/commit/fa183a88e221eb157a0d8679ea724ae20261bcb5) t-3d2a: w2`; `yaks peek <that URL>` reads back as `commit:fa183a8` with the message.

#### 2026-11-18T14:51:02Z

*commits-cli · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-18T14:53:46Z

*commits-cli*

Summary: `yaks land` writes `landed: [short](host commit URL | commit:<full>) subject`; `yaks update <id> --commit [<rev>]` records the full sha, refuses unresolvable revs and (with a remote) unpushed commits, warns with no remote. Docs/skills/brief/README updated. Gate: `cargo fmt --check` clean; `cargo test -p yaks --no-fail-fast` all green (1263 unit + integration, incl. 2 new tests in tests/commits.rs); `yaks verify yaks-f009` PASS.

#### 2026-11-18T14:53:46Z

*commits-cli · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAG5xW5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-18T14:56:45Z

*sources-triage · kind: landed · commit: 65fa57e4ab9db0590d17167ea13e8db902d5bca4 · source: a82a78a14f08c244967d4ace8e85cadf4c7ec776 · branch: main*

landed: 65fa57e4 yaks-f009: land writes host commit links; update --commit records a pushed commit (also yaks-ab81)

#### 2026-11-18T15:00:28Z

*sources-triage · commit: d6ffc1ad7ac18d7de24cc41180db41c4d00d7a48*

Coordinator check: update --commit accepted HEAD after push.
