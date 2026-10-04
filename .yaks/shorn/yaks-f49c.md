---
id: yaks-f49c
title: 'Probe: a worker''s clone, remotes and sibling visibility on a shared-thread machine'
type: task
priority: 3
created: '2026-10-04T19:28:09Z'
updated: '2026-10-04T19:29:39Z'
parent: yaks-99ec
labels:
- delta
- agent
---

Read-only probe for yaks-99ec / yaks-df61 O50. Record, as an attachment plus a short note, what a Delta worker sees on this Linux machine (managed-bare 'local' fallback, repointed 'local' in the coordinator): its checkout and git dir layout, objects/info/alternates, remotes, refs/delta pins, git worktree list, what lives next to its checkout, env names, whether it can read the coordinator's checkout and binary, what 'git fetch local' and 'git push --dry-run local' do, and what yaks path/preflight say. Evidence: the attachment. Judge: coordinator (me).

---
▸ 2026-10-04T19:28:22Z [delta-lead]
Claim (coordinator, delta-lead). Owner: worker probe-1. Scope: read-only; no code changes. Evidence contract: an attachment (probe-1.md) with each command and its real output, plus a short note of surprises; judge: coordinator re-reads and re-runs the cheap commands. Context from my own checks: the coordinator clone git dir is <root>/<dir>/yaks.git with .git a gitdir: file; objects/info/alternates names the managed bare repo; pins refs/delta/<dir>/yaks/<sha> live in the managed repo; laptop layout is <repo>/.delta/{clones,worktrees}/<dir>.

---
▸ 2026-10-04T19:28:39Z [delta-lead]
Worker probe-1 spawned (agent e4f18a94cefb4330), model anthropic/claude-sonnet-5-5 (read from the spawn confirmation); base commit e4c2bf5.

![probe-1](artifacts/yaks-f49c/probe-1.md)

---
▸ 2026-10-04T19:29:35Z [probe-1]
Probe done; evidence .yaks/artifacts/yaks-f49c/probe-1.md (started 19:28:38Z). Key findings: checkout and clone are siblings (<root>/<dir>/yaks and yaks.git, no .delta/); 'local' is a Delta managed BARE repo (also the alternates target) with no refs/heads, only refs/delta pins + tags + origin copy, so 'git fetch local' is a no-op and local/main does not exist; push --dry-run local HEAD:refs/heads/probe-1-dryrun is accepted; coordinator checkout and its target/release/yaks are readable from the worker; preflight ok, doctor clear.
