---
id: yaks-f009
title: 'Commits phase 4: writers emit host commit links; yaks update --commit'
status: shaving
type: feature
priority: 2
created: '2026-11-18T03:03:10Z'
updated: '2026-11-18T14:47:41Z'
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
