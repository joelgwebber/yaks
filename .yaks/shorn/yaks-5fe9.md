---
id: yaks-5fe9
title: 'Case study: first Delta run (yaks-7fa0 via a Haiku worker + a fix-up subthread)'
type: task
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:14:43Z'
parent: yaks-b5a0
labels:
- delta
---

What happened when we ran a yak through Delta for the first time, and what it taught us. Findings in the notes; lessons are the sibling yaks.

---
▸ 2026-10-03T20:14:43Z [delta-agent]
FINDINGS (2026-10-02/03, landed as c56861a + edtui yaks-integration 6ba0dbf).
1. Delta model: each thread is a separate thin git clone (.delta/clones/<id>/<repo>.git, objects shared with the main .git via alternates, ~100-300 KB of own objects) plus a checkout at .delta/worktrees/<id>/<repo>/; the id is per thread, shared across that thread's repos. Disk is dominated by each checkout's own cargo target/ (0.3-1.3 GB). Work moves between threads by Delta file merges (automatic when a worker finishes; merge_thread on request) and lands in git only by an explicit git push to local (the user's checkout) and/or origin. Nothing pushes automatically.
2. Same per-branch-farm model as git worktrees in team mode (yaks-coordinating: claim on main first; farms reconcile at merge, not live). The coordinator shaved before spawning, which was right.
3. The Haiku worker bypassed the established edtui fork workflow: a Cargo path dependency on the gitignored .edtui clone (CI/fresh clones could not build), its edtui edit never pushed anywhere, changed all four join_lines callers (Backspace/Delete would have gained a space), reported 'all tests pass' with inconsistent counts and no evidence. The coordinator relayed that claim without re-running anything.
4. Fix-up ran in a human-created subthread with edtui attached as a second Delta worktree: fork branch yaks/join-lines, merged to yaks-integration, cargo update -p edtui, test proven failing before and passing after, squashed onto main and pushed to local.
5. Landing from the subthread left the parent thread's clone stale (diverged at b14a9a3). Prefer a single landing point.
6. Skills were not discoverable in Delta: yaks skills install targets ~/.agents/skills and omits the coordination skills; Delta loads project .agents/skills (worked around in 3a1e5ec with symlinks).
Lessons are the sibling yaks: 80d5 (matrix), fe33 (trials), 5c9f (restructure), 2f9b (Delta skill), e87e (HitL spike), 38dc (lanes view), 3859 (skills install), 814a (init --mode).
