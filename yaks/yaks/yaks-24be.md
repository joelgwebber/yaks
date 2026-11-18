---
id: yaks-24be
title: 'Feedback acknowledgement: record every triaged feedback-to-yaks entry once, whatever its outcome'
status: hairy
type: task
priority: 2
created: '2026-11-18T14:44:16Z'
updated: '2026-11-18T14:44:16Z'
parent: yaks-1c89
tags:
- skills
- design
- meta
---

Joel, 2026-11-18: "I also don't see an explicit pattern for acknowledging feedback and ensuring we don't waste time revisiting already-triaged feedback ... just editing the upstream feedback notes with a yak reference? Or just creating a simple local 'database' in a dok / doks that keeps the acknowledgement in this repo, in the event we don't have write access to the source."

Follows yaks-20fe (the cross-farm feedback design) and yaks-feaf (the collector).

## The gap (found in the first real round, 2026-11-18, on cowpaths/joel)

yaks-feaf dedupes by `yaks create --key feedback-to-yaks:<farm>#<heading>`: the key becomes the new yak's `source:`. That only acknowledges entries that **became a new yak**. Of the 8 entries in that round:

- 4 became new yaks: they carry the key (yaks-5250, yaks-9dc2, yaks-329a, yaks-8a49).
- 3 matched existing yaks and got a note (yaks-76a3, yaks-d11e, yaks-837d). A yak has one `source:`, so the key can't go there; next round re-triages them.
- 1 was not a yak bug (`dok row set` "stamps as_of": the agent's own helper passed `--set as_of=...` on every call). Nothing records that verdict, so next round re-investigates it.

So half the round has no acknowledgement. Every outcome needs one, in one place.

## Options

**A. Write back to the source entry** (`- **Triaged:** yaks-5250 (new) 2026-11-18`).
- Plus: the reporting farm sees the outcome; one place to look.
- Minus: writes to another farm, which yaks-20fe ruled out for the collector; needs write access (fails for others' farms, the later case); a `yaks-5250` there links nowhere; edits another farm's dok, which the skill says only to append.

**B. A ledger table dok in this farm**, `/doks/feedback-triage.md` (`type: topic` with a table, or its own kind), one row per entry, key in the first column:

| entry | outcome | yak | date | why |
| --- | --- | --- | --- | --- |
| cowpaths/joel#2026-11-13: \`dok row set\` stamps ... | not-a-bug | | 2026-11-18 | the agent passed --set as_of itself |

- Plus: every outcome (new, linked, not-a-bug, duplicate, wontfix) fits; no write to another farm; `yaks dok row set 'doks/feedback-triage.md#<key>' --set ...` writes it and `yaks dok query --rows` reads it, so the collector's "skip what's acknowledged" is one lookup; it works the same for farms we can't write to.
- Minus: the reporting farm never learns the outcome (fine while it's Joel's farms; the "send feedback upstream" step is later, per yaks-20fe).

**C. Keep `--key` only, and file a yak for every entry**, closing it at once with a `Duplicate of yaks-x` / `not a bug` note.
- Plus: no new structure.
- Minus: a dead yak per non-bug clutters the backlog and `yaks list --all`; a linked entry gets a stub yak beside the real one.

## Lean

**B**, with the `--key` on new yaks kept as it is (so a yak still says where it came from). The collector becomes: read the entries, skip those with a ledger row, triage the rest, write one row each. Writing back (A) is the later, opt-in "tell the reporter" step, for when other people's farms send feedback.

Also, found in this round, for the same change to yaks-feaf's description:

- Its glob is `farms/*/.yaks/doks/feedback-to-yaks.md`; schema 9 farms (cowpaths/joel) keep doks at the root: `farms/*/doks/...`. Read both.
- Key by the farm's repo (`cowpaths/joel`, from `remote.origin.url`), not its farm dir name (`joel-215beeeb`): that name changes with each clone.
- The skill's entry format could ask for one line on what was run (the command line itself): the as_of report would have been caught by its author.

Evidence when done: yaks-feaf's description and the `yaks-upkeep` "Yaks feedback" section say the ledger rule; the ledger holds the 2026-11-18 round's eight rows; a second collector pass over cowpaths/joel triages nothing.
