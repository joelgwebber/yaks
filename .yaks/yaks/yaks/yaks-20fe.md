---
id: yaks-20fe
title: 'Cross-farm feedback: other farms'' upkeep friction flows back to this one'
status: hairy
type: task
priority: 2
created: '2026-11-10T17:59:19Z'
updated: '2026-11-10T17:59:27Z'
parent: yaks-1c89
tags:
- skills
- design
needs: human
---

Joel, 2026-11-10 (on yaks-2b40): "let's also discuss ideas around a process for getting feedback from other farms' experience with these memory consolidation processes. Maybe something as simple as having them keep a running log of friction points that we can copy over to this farm as feedback?"

## What already happens

Every farm's upkeep already produces the raw material: session notes carry `## Lessons`, and reflection files "friction with yaks or its skills" as a yak "where the person tracks yaks' own work". In another farm that phrase has no answer, so today the friction either stays in that farm's yaks or Joel relays it by hand (yaks-3f35 from schism, yaks-d426 from cowpaths/joel). On this machine 14 farms exist; four have session notes.

## Proposal

1. **A friction dok per farm, written by the existing steps.** `/doks/yaks-feedback.md` (`type: topic`), one dated entry per item: what happened, the command or skill section, the yak or session that shows it, and a `status:` (`new` / `relayed`). The `yaks-upkeep` skill's review step 4 and reflection step 7 say: in a farm that is not where yaks itself is tracked, append the friction there (and file a local yak only if that farm wants to fix it locally).
2. **Collecting it here is a recurring yak in this farm** (weekly, beside the reflection): for each farm on this machine (`~/.local/share/yaks/farms/*`, read-only), read the `new` entries of its feedback dok, file or extend yaks here (`--source` naming the farm and entry, so a re-run finds it with `--key` and never duplicates), and record the farm and entry in the round's account. Marking entries `relayed` in the other farm is a write to it, so the collecting agent asks the person first, or leaves it and dedupes by `--key`.
3. **No new command yet.** A dok and a recurring yak cover it with today's tools; `yaks find --kind doks` across farms is a shell loop. If the loop proves useful, a later `yaks feedback collect` can mechanize step 2.

## Alternatives

- **Each farm files yaks straight into this farm** (`yaks -C ~/src/yaks create ...`): no relay step, but it writes across repos from an unrelated session, needs this checkout present, and mixes another project's context into yaks' backlog unreviewed.
- **Only session notes, no dok:** the collector greps every farm's `## Lessons`. Zero new convention, but lessons mix project friction with yaks friction, and nothing marks what was already relayed.
- **A shared upstream channel** (GitHub issues on yaks): right for other people's farms eventually; heavy for Joel's own, and it puts farm content outside the farm.

Lean: the dok plus the recurring collector, with entries deduped by `--key` so the collector never has to write to another farm.

---
▸ 2026-11-10T17:59:27Z [herd-unlock]
kind: ask
needs: human
---

asked: needs human

Three choices for the cross-farm feedback loop (the description has the detail and the alternatives). My lean on each:

1. **Where other farms put yaks friction:** a `/doks/yaks-feedback.md` dok in each farm, appended by review and reflection, one dated entry each. Alternative: no dok; the collector greps every farm's session `## Lessons`. Lean: the dok, since lessons mix project friction with yaks friction.
2. **Who carries it here:** a weekly recurring yak in this farm reads every farm on this machine (read-only) and files or extends yaks with `--key <farm>#<entry>`, so it never duplicates and never writes to another farm. Alternative: other farms create yaks here directly with `yaks -C`. Lean: the collector.
3. **Scope now:** skill text plus the recurring yak, no new command until two rounds show it pays. Lean: yes.
