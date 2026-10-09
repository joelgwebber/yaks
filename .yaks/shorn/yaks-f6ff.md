---
id: yaks-f6ff
title: 'Skills pass: wire sheds/changes/inbox --sheds/show --sheds/answer @shed into the coordinating and working skills'
type: task
priority: 3
created: '2026-10-09T20:59:45Z'
updated: '2026-10-09T21:05:39Z'
parent: yaks-a3d2
labels:
- skills
---

---
▸ 2026-10-09T20:59:45Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-09T20:59:45Z [delta:Delta :Yaks (cont'd)]
Edited (coordinator): core §3 (the cross-shed toolkit: sheds, changes, inbox --sheds, show --sheds, the <shed> selector, -C with care), core §8 (answer a worker's ask in its shed with answer <id>@<shed>; design forks: tell the human the shed and id), team 'Asks' (asks are visible via inbox --sheds before landing; answer in the shed), delta 'Asking and resuming' (answer @worker instead of having the worker run answer) and the overdue-worker hazard (changes <name>), working (an answer may arrive uncommitted in your copy; commit it with your yak file). Gate: cargo test skills (33 + 6 + 1). Cold read by a Sonnet scout next.

---
▸ 2026-10-09T21:02:29Z [delta:Delta :Yaks (cont'd)]
Cold read 3 (cold-read-3, claude-sonnet-5-5 scout, read only the skill files). Clear: Q1, Q2, Q3 (command and committer), Q7, Q8, Q9. Ambiguous: Q4 (no place to put the design-fork lean without editing the worker's yak), Q5 (the working skill looked like it let a worker answer its own ask), Q6 (no overdue threshold; FARM ACTIVE undefined; a worker with no notes has no name), Q10 ("the only command that writes outside your checkout" is false: -C and git push local).
Top 3: (1) the team skill said "not by watching each other" while core §3 says to watch; (2) the design-fork lean contradicted "never edit a worker's yak"; (3) the write-boundary claim was false, and `resolve` / FARM ACTIVE / a note-less worker's name were undefined. Also: a blocked worker's unfinished edits may land in the yakherd's tree, and no rule covered that.
Fixed (coordinator): the team skill says plain commands do not see shed state but the cross-shed commands do, and to watch for asks and stalls, never to merge by hand; the lean goes in the reply to the human and on the umbrella/parent yak; "only writer" is scoped to the cross-shed commands, and -C is a cd (writes included, only on a checkout you own); a note-less worker is named by its dir id or branch; FARM ACTIVE and an overdue default (30 min) are defined; `resolve` is replaced by the rule in README/docs/skill; the working skill's self-answer wording is fixed; Delta "Asking and resuming" says how to treat a blocked worker's applied edits. Second cold read next, on only the failed questions.

---
▸ 2026-10-09T21:05:39Z [delta:Delta :Yaks (cont'd)]
Cold read 4 (cold-read-4, claude-sonnet-5-5 scout; re-asked only the questions that failed before). Now clear: watching in team mode (Q1), the worker woken after an answer (Q3), the write boundary (Q6). Still ambiguous, all fixed in this commit:
- I had introduced a contradiction: "restore a blocked worker's applied edits with git checkout" against Delta step 6, "do not hand-revert Delta's applied state". Fixed: leave them alone and land by SHA later.
- "Do not edit a worker's yak at all" against `answer @shed`: now an explicit exception while the worker is idle after an ask.
- A note-less worker's dir id could not be mapped to its name: now run `yaks sheds` right after spawning and record the dir id on the spawn line.
- Overdue: FARM ACTIVE vs DIRTY explained; "slow vs dead" has a rule.
- Design fork: what to do when there is no umbrella or parent yak (the lean goes in the message only); stay idle until the human answers; `inbox --sheds` shows the answered yak as needs: agent, then wake the worker.
- "Delta dir id" and "unique substring" are defined where first used in core.
- The team skill's "commit answers" now says why when Delta copies the working tree; a design fork cannot be answered from a shared-only machine.
Not changed: the yaks-df61 O61/O62 references (provenance; the rules stand on their own) and the "older briefs" sentence in yaks-working (the cold read rated it clear).

---
▸ 2026-10-09T21:05:39Z [delta:Delta :Yaks (cont'd)]
Done: two cold reads (cold-read-3: 10 questions; cold-read-4: the 6 failed or new ones), every gap fixed, skills tests 33 + 6 + 1 pass.

---
▸ 2026-10-09T21:05:39Z [delta:Delta :Yaks (cont'd)]
moved: shaving -> shorn
