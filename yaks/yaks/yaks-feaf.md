---
id: yaks-feaf
title: Collect yaks feedback from other farms
status: hairy
type: task
priority: 3
created: '2026-11-10T18:45:04Z'
updated: '2026-11-25T04:24:21Z'
tags:
- meta
- skills
due: 2026-11-17
every: P1W
wake: agent
---

Collect other farms' friction with yaks (design: yaks-20fe; acknowledgement: yaks-24be; the convention: the `yaks-upkeep` skill, "Yaks feedback"). **Read only: never write to another farm.** Every entry triaged is recorded once, whatever came of it, in this farm's ledger [/doks/feedback-triage.md](/doks/feedback-triage.md) (its body says the columns and outcomes).

1. **Find the docs.** Every farm on this machine except this one: `ls ~/.local/share/yaks/farms/*/doks/feedback-to-yaks.md` (schema 9 and later) and `.../farms/*/.yaks/doks/feedback-to-yaks.md` (older). Skip `tmp.*` and other scratch farms.
2. **Key each entry** `<owner/repo>#<heading>`: `owner/repo` from that farm's `git -C <farm> config remote.origin.url` (not the farm dir name, which changes per clone), the heading after `## ` exactly.
3. **Skip what is acknowledged.** An entry with a ledger row (`yaks dok query --rows --where 'entry=~<owner/repo>#'`) is done, unless it has an `**Update YYYY-MM-DD:**` bullet later than the row's `triaged`: then look again and update the row.
4. **Triage the rest**, checking each against the current code or that farm's history (read its **Shows it** links: `yaks -C <farm checkout> peek <link>`, or the file), and reproduce what you can on a scratch farm:
   - new work: `yaks create "<title>" --key feedback-to-yaks:<key> ...`; outcome `new`;
   - already tracked: a note on that yak naming the entry; outcome `linked`;
   - not a yaks problem, already fixed, a duplicate of another entry, or won't fix: outcome `not-a-bug` / `done` / `duplicate` / `wontfix`, with the reason in `note`.
5. **Write the row**: `yaks dok row set 'doks/feedback-triage.md#<key>' --set outcome=... --set yak=... --set triaged=<today> --set 'note=...'`. Then a second pass (steps 1-3) must find nothing pending.
6. Shear with the account: per farm, entries seen, new, linked, other outcomes (with the yaks).

#### 2026-11-25T04:24:21Z

*feedback-triage*

Round 2026-11-25 (with yaks-24be's ledger): cowpaths/joel 9 entries: 1 new (yaks-6a2c), 1 re-triaged for its Update (yaks-837d, linked), 7 skipped as acknowledged. fullstorydev/surveyor 4 entries, all new: yaks-e0cd, yaks-b2e9, yaks-dc7c, yaks-3c55. Ledger: /doks/feedback-triage.md, 14 rows. Second pass: nothing pending.
