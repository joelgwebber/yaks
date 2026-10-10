---
id: yaks-ad6d
title: 'yaks sheds: list the current shed too (marked), and let changes/--shed/answer name it'
type: feature
priority: 2
created: '2026-10-10T17:21:10Z'
updated: '2026-10-10T17:21:10Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

Joel (2026-10-10): `yaks sheds` omits the shed you are in, so you cannot see where you are, and `changes <name>`, `--shed <name>` and `answer <id>@<name>` cannot name your own shed (from inside this clone `yaks changes <its own name>` fails; from the primary `main` was mis-resolved, yaks-3344). Direction agreed (A): list the current shed as a row, marked, and let every selector accept it. The semantics of what "changes" are measured against, per shed, are the open design discussion (see the notes) and decide the code: do not start before that is settled.
