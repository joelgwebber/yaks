---
id: yaks-feaf
title: Collect yaks feedback from other farms
status: hairy
type: task
priority: 3
created: '2026-11-10T18:45:04Z'
updated: '2026-11-18T14:44:38Z'
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

#### 2026-11-18T14:44:38Z

*feedback-triage*

First round run by hand, 2026-11-18 (asked by Joel; not sheared, since the acknowledgement question in yaks-24be changes this yak's own steps). One farm on this machine has the dok: cowpaths/joel (`farms/joel-215beeeb/doks/feedback-to-yaks.md`, schema 9, so not at this description's `.yaks/doks/` glob). 8 entries, all links resolve, all in the expected shape (one is missing a blank line before its heading; harmless).

- New yaks (keyed `feedback-to-yaks:cowpaths/joel#<heading>`): yaks-5250 (`whoami --set` doesn't reach other checkouts), yaks-9dc2 (email autolink as broken link), yaks-329a (`.mdx` gets no text), yaks-8a49 (`check` and `.yaks/.activity/`).
- Linked to existing yaks with a note: yaks-76a3 (run from outside a checkout; p4 to p3), yaks-d11e (`touched` missed harness-actor writes), yaks-837d (regrow right after shear; the entry's "due was already tomorrow" is not what the commits show).
- Not a yaks bug: `dok row set` stamps `as_of`. 0.0.38 leaves other cells alone (checked); the originating thread's helper passed `--set as_of=2026-11-13` on every call.

Each one reproduced on 0.0.38 or checked against the farm's history.
