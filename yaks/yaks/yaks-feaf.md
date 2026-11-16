---
id: yaks-feaf
title: Collect yaks feedback from other farms
status: hairy
type: task
priority: 3
created: '2026-11-10T18:45:04Z'
updated: '2026-11-10T18:45:04Z'
tags:
- meta
- skills
due: 2026-11-17
every: P1W
wake: agent
---

Collect other farms' friction with yaks (design: yaks-20fe; the convention: the `yaks-upkeep` skill, "Yaks feedback").

1. For each farm on this machine except this one (`ls ~/.local/share/yaks/farms/*/.yaks/doks/feedback-to-yaks.md`; skip `tmp.*` scratch farms), read it. **Read only: never write to another farm.**
2. For each entry (a `## <date>: <title>` heading), key it `feedback-to-yaks:<farm repo or dir>#<heading>` and `yaks create "<title>" --key <that key> ...`. An existing yak for that key comes back instead of a new one: skip it. If the entry is friction already tracked here under another yak, link it there with a note instead, and still create nothing twice.
3. Read the entry's **Shows it** links in that farm when the entry is unclear (`yaks -C <farm checkout> peek <link>` or read the file).
4. Shear with the account: per farm, entries seen, yaks created, entries linked to existing yaks.
