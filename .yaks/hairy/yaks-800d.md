---
id: yaks-800d
title: Concurrent updates to one yak can silently drop a note (no locking on read-modify-write)
type: bug
priority: 3
created: '2026-10-03T22:24:19Z'
updated: '2026-10-03T22:24:19Z'
parent: yaks-b5a0
labels:
- cli
- agent
---

Measured 2026-10-03 (log yaks-df61 O26): N parallel 'yaks update <same id> --note' processes against one yak. 4 writers x 25 notes = 100/100 preserved; 16 parallel x 400 = 400/400; 32 parallel x 800 = 796/800 (4 notes silently lost, exit codes all 0, yaks doctor clean). atomic_write (tmp + rename) prevents torn files but update is a read-modify-write with no lock (no flock/lock file anywhere in src/), so two processes that read the same version drop one's edit. Writers on DIFFERENT yaks were safe (4 x 25 each = 25 each). Matters most where one yak has several writers: a coordinator and workers on a shared parent yak, and live-shared farms (private mode in git worktrees or Delta, log O11). Options: (a) advisory lock per yak file or per farm around read-modify-write (flock via std File::lock, stable since 1.89, or a .lock file), (b) compare updated: before rename and retry, (c) document one writer per yak as the rule and leave it. Evidence when done: the same stress script as a test or doctor-style check showing N/N.
