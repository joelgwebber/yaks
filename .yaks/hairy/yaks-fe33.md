---
id: yaks-fe33
title: Repeatable parallel-run trials, one per environment
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T21:22:16Z'
parent: yaks-b5a0
labels:
- agent
---

A small scripted scenario we can rerun in each environment after any skill change: 3-4 disjoint toy yaks, one deliberate same-file conflict, one human question (ask). Record per run: claims, merges and conflicts, which notes were visible when, evidence quality, human interventions, wall time. Score like the 3901 runs and attach results. Possible vehicle: yaks-fe5a (blinded skill/prompt eval).

---
▸ 2026-10-03T20:39:31Z [delta-lead]
Trial list from the Delta matrix (yaks-80d5), highest value first: (1) Delta x team repeat of the 7fa0 run with a full worker brief, and a stronger worker model, to separate brief quality from model; (2) Delta x private (O11 risk, in a throwaway repo, not this one); (3) direct tests of O3 (uncommitted vs committed claim) and O9b (worker leaves artifacts unstaged); (4) parent commits during a worker's run, then the worker lands (O2); (5) yaks-e87e resume/message behavior.

---
▸ 2026-10-03T21:14:11Z [delta-lead]
Trial 1 launched: two workers in parallel on disjoint yaks (b47b = Haiku default, CLI; cc52 = Sonnet 5.5, TUI) with the draft brief template (artifact on yaks-2f9b). Deliberate probes: O3 (b47b claim committed, cc52 claim left uncommitted), attribution via inline YAKS_ACTOR, un-staged evidence is NOT sabotaged (workers told to git add). Confounds acknowledged: different yaks and models, n=1 each, so this shows feasibility and failure modes, not a model comparison.

---
▸ 2026-10-03T21:14:55Z [delta-lead]
Correction: both trial workers report claude-sonnet-5-5 (the worker profile default changed since the 7fa0 run, log O16). Trial 1 therefore does not vary the model; run a Haiku cell separately with an explicit model. The O3/O2/attribution probes stand.

---
▸ 2026-10-03T21:22:16Z [delta-lead]
Trial 1 done (scorecard at the end of the observation log, yaks-df61): 2 workers, both Sonnet 5.5; cc52 premise refuted with evidence, b47b done after one scope block, both gates reproduced by me; landing quirks O2/O22; brief v2 + coordinator checklist on yaks-2f9b. Not yet run: Haiku with the same brief, Delta x private (O11), same-file conflict, repeat to separate noise.
