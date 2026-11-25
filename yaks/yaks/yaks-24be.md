---
id: yaks-24be
title: 'Feedback acknowledgement: record every triaged feedback-to-yaks entry once, whatever its outcome'
status: shaving
type: task
priority: 2
created: '2026-11-18T14:44:16Z'
updated: '2026-11-25T04:24:12Z'
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

#### 2026-11-18T14:44:37Z

*feedback-triage · kind: ask · needs: human*

asked: needs human

Acknowledgement: A (write back to the source entry), B (a ledger table dok here, my lean), or C (a yak for every entry)? The description has the tradeoffs. Answer B and I'll write the ledger with this round's 8 rows and update yaks-feaf and the skill.

#### 2026-11-19T20:31:29Z

*sched-1 · kind: answer · needs: agent*

Agreed, I think (B) will be cleaner overall. We need just barely enough structure to ensure we don't re-triage every time we scan another farm's feedback.

#### 2026-11-25T04:17:30Z

*feedback-triage · kind: pickup*

picked up

#### 2026-11-25T04:17:31Z

*feedback-triage · kind: move · from: hairy · to: shaving · thread: ksQQpRfd3SiPQbKzhUOE_ITXyJLOAG5JQpIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

[feedback-pending](yaks-24be/feedback-pending.sh)

#### 2026-11-25T04:24:12Z

*feedback-triage*

Done as answered (B), tested on the 2026-11-25 round.

- **Ledger:** [/doks/feedback-triage.md](/doks/feedback-triage.md), one row per triaged entry keyed `<owner/repo>#<heading>`; outcomes new / linked / not-a-bug / duplicate / done / wontfix. Backfilled with the 11-18 round's 8 rows. Written with `dok row set`, read with `dok query --rows` (checked first on a scratch farm: keys with `#`, backticks and `|` round-trip).
- **Re-triage rule:** an entry gets another look only when it has an `**Update YYYY-MM-DD:**` bullet later than its row's `triaged`. The `yaks-upkeep` skill now tells reporters to append such a bullet rather than edit (branch `yaks-24be-feedback-ledger`, not landed).
- **Collector:** yaks-feaf's description rewritten around the ledger (both farm layouts, repo keys, the update rule, a second pass that must be empty).
- **Test:** the attached `feedback-pending.sh` (reads every farm's dok, prints entries with no row or a newer Update). First pass: 5 NEW (1 cowpaths/joel, 4 fullstorydev/surveyor) and 1 UPDATED (cowpaths/joel 11-18), with the 7 other rows skipped. Triaged: yaks-6a2c, yaks-e0cd, yaks-b2e9, yaks-dc7c, yaks-3c55 new; yaks-837d linked again. Second pass: nothing pending.
- Left for later: the script is a stopgap. If the collector proves useful, `yaks feedback pending` could mechanize steps 1-3 (yaks-20fe's "no new command yet").
