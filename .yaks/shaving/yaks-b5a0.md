---
id: yaks-b5a0
title: 'Parallel development: finish and harden coordinator/worker skills + tooling (git worktrees, Delta)'
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-06T21:37:02Z'
labels:
- agent
- skills
---

The coordination layer (the yaks-coordinating / yaks-working dev skills, public vs private farm modes, claim -> fan out -> merge) is nascent, hard to follow, and lightly tested. Goal: parallel agent work on a yaks farm that is reliable and legible in each environment we actually use: plain git worktrees today, Delta threads and subagents next. Test aggressively against real runs and refactor the skills and CLI freely as we learn.

Principles: prove it with real runs, not prose; one followable skill per environment instead of one skill full of conditionals; install/init make the right setup explicit rather than leaving agents to infer it.

Related, not reparented (decide whether to fold in): yaks-10fb (post-3901 follow-ups: 5d5c, 213b, eb5f, 70e5, 8265), yaks-5f12 (Model 3 vs git-store), yaks-c061 (answered-question pickup), yaks-0a87 (checkpoint footgun), yaks-75cc (drift as signal), yaks-77a5 (lightning-round case study).

---
▸ 2026-10-03T20:27:55Z [delta-agent]
Direction (Joel, 2026-10-03): Delta is NOT a fourth farm mode. Farm mode (public/team, private/local, out-of-tree) and environment (git worktrees, Delta) are orthogonal. Plan: (1) split the coordinator/worker skills by FARM MODE, so each is followable without the mixed public/private conditionals (yaks-5c9f); (2) a separate Delta skill that layers on top of whichever mode skill applies (yaks-2f9b); (3) skills install + init consolidate into the in-repo .agents/ by default, with init choosing the mode (yaks-3859, yaks-814a).

---
▸ 2026-10-03T20:39:31Z [delta-lead]
Progress (delta-lead): resynced from local/main (61ec995); yaks-80d5 shorn with the environment x farm-mode matrix (artifact skills-matrix.md); observation log yaks-df61 open with O1-O13; new bug yaks-b4dc (discover walk-up hazard) filed. Headline findings: private/out-of-tree + Delta shares one live farm (reproduced); skill tool did not load project skills after the reset (O1); my own first brief and acceptance were the main failures (O5, O6). Next: yaks-fe33 trial 1, which needs a decision on the worker model. Proposing to fold into this family: yaks-0a87 and yaks-75cc (feed 5c9f), yaks-c061 (feeds e87e); leaving 10fb, 5f12 and 77a5 related, since they are about archaeology and the store model, not skills.

---
▸ 2026-10-04T20:53:04Z [delta-lead]
Spawned (base ca6d595, model anthropic/claude-sonnet-5-5 for both, read from the spawn confirmations): lanes-1 (agent cf3d8487f2a446d5) on yaks-e545; attr-1 (agent d5432b27565a40aa) on yaks-7149. Decisions behind them: yaks-99ec (git worktrees plus a sibling-directory scan, no Delta database; skills stop assuming 'local' is a checkout) and yaks-7149 (option A). Per yaks-df61 O52 the coordinator does not edit a worker's yak after spawn.

---
▸ 2026-10-04T21:01:08Z [delta-lead]
Landed: yaks-7149 part 2 (2c28d23, plus follow-up 649b48a: strict doctor must not count moved: entries as evidence) and yaks-e545 'yaks lanes' (ddb5a83); yaks-8265 shorn as covered. Spawned lanes-2 (agent 7d0c5122bd6940af, anthropic/claude-sonnet-5-5, base 04d7114) on yaks-d738 (lanes: a lane's own changes vs merge-base, behind count). Nothing pushed to local/origin yet; the bundle goes to local main after lanes-2 and a decision on the preflight 'local is a checkout' check.

---
▸ 2026-10-04T21:06:09Z [delta-lead]
Pushed to local main (dab94a1; archive local delta/trial7): yaks-7149 part 2, yaks lanes (e545, d738), cross-machine probe and Delta skill fixes. Not on origin. Next candidates: preflight check that local is a checkout (new child of yaks-99ec), yaks init --mode (yaks-814a) with project-local skill install (yaks-3859), the TUI lanes view (yaks-38dc remainder), yaks brief (yaks-2c22).

---
▸ 2026-10-05T19:49:38Z [delta-lead]
Spawned (base 5dd2f4d, anthropic/claude-sonnet-5-5 both): pre-1 (agent 9b42bab0108849da) on yaks-d1bd, lbl-1 (agent e8c8fba428b440c4) on yaks-c29a. Spawn titles now start with the worker name (Delta skill). Joel's primary checkout (laptop) was a shallow repo; fixed with git fetch --unshallow at his request; one pinned commit (2c28d23) is still a shallow boundary, harmless.

---
▸ 2026-10-05T19:57:29Z [delta-lead]
Landed: yaks-c29a lane labels (88b1ab1), yaks-d1bd 'yaks preflight --push-main' (15c5cbc; Delta skill now points at it, 5f42ee2). Spawned unk-1 (agent 0a69b695de03476d, anthropic/claude-sonnet-5-5, base 83abad0) on yaks-1b39. Asked Joel the init/skills-install forks on yaks-814a. Not yet pushed to local main (after unk-1).

---
▸ 2026-10-05T20:57:26Z [delta-lead]
Spawned ins-1 (agent 48d1966b1f5e4947, anthropic/claude-sonnet-5-5, base ea99697) on yaks-3859 (project-local skills install, opt-in coordination group). Joel's answers on yaks-814a (1 yes, 2 yes optional, 3 'should init/skills roll together?') are uncommitted in his checkout; reply to him in chat first. O58 records the Delta shallow-file investigation.

---
▸ 2026-10-05T21:08:55Z [delta-lead]
Landed yaks-3859 (c568e09; reviewed: gate 414 + 8 + 28 + 3, install scenario reproduced in a temp repo with a temp HOME, six coordination skill descriptions load through the real skill tool). Spawned ini-1 (agent f3fb705cf6254a56, anthropic/claude-sonnet-5-5, base 629cc70) on yaks-814a. Asked Joel the auto-update fork on yaks-36b6. Not pushed to local main since 5a07d6a (Joel's checkout is at f441b3b; my clone merged it).

---
▸ 2026-10-05T22:40:25Z [delta-lead]
Joel's batch (2026-10-05): decision C on yaks-36b6, plus yaks-382b, 7acf, 75cc, 0a87, c061, f313 to look at while the coordination mechanics are finished before the lanes UI design (7204).

Spawned (base ca50b88, anthropic/claude-sonnet-5-5 both):
- ntf-1 (agent 483e2795e9184806) on yaks-5c83: notify once per release when project-local skills are stale (child of 36b6).
- inp-1 (agent b2485e98bc95473b) on yaks-c53a: stdin and file input for notes/descriptions (the CLI half of 382b).

Coordinator, in this clone: the skills pass (a398, Decision: notes, lanes/init pointers, 7acf yakherd, the 382b markdown guidance once inp-1 lands, cold-read) under yaks-5c9f.

Joel's checkout holds many uncommitted yak edits (36b6, 7204, c061, 382b...): I do not write into those files until he commits; new work on them goes into child yaks. Pushing to his main is blocked until then (preflight --push-main reports it).

---
▸ 2026-10-06T03:58:55Z [delta-lead]
Spawned (base 0725169, anthropic/claude-sonnet-5-5 both):
- agt-1 (agent 017210ed9be846ae) on yaks-c061: `needs: agent` pickup state, `yaks pickup`, inbox sections, `replied` flag (Joel's decision recorded on c061).
- brf-1 (agent c515ac6c7ea04cd2) on yaks-2c22: `yaks brief` (the binary owns the yak-derived core; Joel: "putting these into the tools now").

Pushed to Joel's repo as pr/yakherd-input-notice-2 (6 commits: four squashes, a merge of his 2e0aae6, the 2c22 answer); main waits for his checkout to be clean (yaks-ea13 uncommitted). Archive: delta/trial10.

Incident for the yaks-5f12 trigger: merging Joel's 'Yak herding.' conflicted in yaks-2c22 (my ask and his answer both appended at the end of the file). Resolved by keeping both. Count: 1 hand-resolved conflict inside .yaks/ so far.

---
▸ 2026-10-06T19:13:17Z [delta-lead]
brf-1 (agent c515ac6c7ea04cd2, yaks-2c22) went silent: its lane xw51kfjdpqy5 shows src/brief.rs (354 lines, untracked), an edit to src/main.rs and one note on yaks-2c22, last file write 04:02Z on 2026-10-06; no final message by 19:15Z (about 15 hours). The threads tool cannot read it ("history ... was reset while its state was awaited", the same offline/unreachable-history condition Delta logs). Nothing was committed.

Action taken: sent brf-1 a status request through send_agent_message (finish if alive, else say what happened; do not start over). If it does not answer, spawn brf-2 seeded from brf-1's src/brief.rs (readable in its lane) rather than from scratch.

Lesson for the Delta skill: a worker can die without a final message and the coordinator is never told. `yaks lanes` is what showed it (dirty files, an old FARM ACTIVE time, a `who` label); check lanes when a worker is overdue.

---
▸ 2026-10-06T19:51:48Z [delta-lead]
Published: origin/main is now d42cf08 (a fast-forward from 2e0aae6, pushed on Joel's explicit "push origin!"); local main is still waiting for Joel's checkout to be clean (yaks-ea13 uncommitted); the same commits are on his repo as pr/yakherd-input-notice-2, archive delta/trial11.

yaks brief is NOT in the thread worktree (no src/brief.rs); only yaks-c061 landed. brf-1's lane still holds the untracked draft and has not answered the status request (no change since 04:02Z).

Spawned brf-2 (agent 0c8cefa7b32f4c7b, anthropic/claude-sonnet-5-5, base db7cc43) on yaks-2c22, seeded from brf-1's draft (copy and review, no writes in the other lane). Risk accepted: if brf-1 ever delivers, the coordinator lands only one of the two; the other is discarded.

---
▸ 2026-10-06T20:01:59Z [delta-lead]
yaks brief is done and sheared (yaks-2c22; evidence and the clause comparison are on that yak). It was finished by the coordinator after brf-1 and brf-2 died when the thread changed machines (O61).

Published state (2026-10-06, Linux machine): GitHub main is d42cf08 (pushed on Joel's go). This machine's local (the Linux ~/src/yaks) main is 991c0d2, one squash commit on top of d42cf08 (a fast-forward, preflight --push-main ok). The laptop's ~/src/yaks is a separate checkout, still on 2e0aae6 with Joel's yaks-ea13 edit uncommitted; the branch pr/yakherd-input-notice-2 is in its repo. 991c0d2 is NOT on GitHub: it waits for Joel's go.

No delta/trial12 archive: pushing the unsquashed history from this shallow clone is refused ("shallow update not allowed"); the commits remain in the thread and in Delta's refs/delta pins.

Coordination mechanics are finished (preflight --push-main, stdin/file input, the stale-skills notice, needs: agent with pickup, yaks brief, the cold-read gate). Next: the yaks-7204 lanes design discussion.

---
▸ 2026-10-06T21:05:37Z [delta-lead]
Published (2026-10-06, laptop): GitHub main and the laptop's ~/src/yaks main are deea536 (a squash commit on d42cf08: yaks brief, O61, the Delta skill pointer), both fast-forwards, gate green on this machine. The Linux machine's ~/src/yaks holds my earlier equivalent commit 991c0d2, which is not an ancestor of deea536 and has no unique work: on that machine `git fetch origin && git reset --hard origin/main` brings it level.

Why yaks-2c22 was shorn in the thread worktree but not in ~/src/yaks: the thread had moved to the Linux machine, where `local` is the Linux checkout, so the squash was pushed there; the laptop checkout is a separate repo and was not updated until now.

Filed from Joel's yaks lanes review: yaks-37fa (lane baseline defect) and the CLI-affordances design task under yaks-7204.

---
▸ 2026-10-06T21:32:29Z [delta-lead]
Spawned shd-1 (agent 85ba98dba47846fc, anthropic/claude-sonnet-5-5, base f0eef06) on yaks-dfca (the lane to shed rename); the first spawn whose brief was produced by `yaks brief` (plus the wrapper lines the Delta skill prescribes). Joel stays on one machine for the run.

Order: yaks-dfca (rename) then yaks-37fa (lane baseline), since both edit src/lanes.rs. Then yaks status (yaks-ee0a step 2).

---
▸ 2026-10-06T21:37:02Z [delta-lead]
Landed yaks-dfca (the shed rename, 7f49fe2): gate 452 + 8 + 11 + 28 + 19 + 5 + 3 + 3, `yaks lanes` is an unknown subcommand, `yaks sheds` prints what `lanes` printed (shd-1 compared old and new binaries on the same checkout, outputs identical), the only remaining hits for the old word outside history are the two glossary lines. Cold-read gate (cold-read-4, four skills): all loaded, every answer right, no place where "shed" reads oddly.

Spawned bas-1 (agent 637d6f1015f64d9f, anthropic/claude-sonnet-5-5, base b9e7b37) on yaks-37fa (shed baseline from the fork point). Joel stays on one machine. Brief by `yaks brief` plus the wrapper lines, second use.

Not yet published: 7f49fe2 and later are local to the thread; GitHub and the laptop checkout are at deea536. Hold the push until the baseline fix lands (one publish for the rename and the fix), unless Joel asks sooner.
