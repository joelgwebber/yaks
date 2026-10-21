# Walkthrough: a yakherd fans work out to Sonnet workers across Delta sheds (yaks-a3d2, wave 3)

Real commands and their real output, recorded on Joel's Mac on 2026-10-09 while wave 3 ran.
The yakherd runs from its own Delta clone; "the human's checkout" is `~/src/yaks`. Paths
are shortened to `~`; long lines are cut at 160 characters. `notes-1`'s ask in step 5 was
scripted to demonstrate the ask/answer loop; everything else is the work as it happened.

Commands used, in order: `yaks list`, `yaks show`, `yaks brief`, `yaks sheds`,
`yaks inbox --sheds`, `yaks changes <shed>`, `yaks answer <id>@<shed>`, `yaks show <id> --sheds`
(by the workers' tests), `yaks --shed <name>`, plus `git` and the Delta skill's `land.sh`.

### 1. Claim. The yakherd files wave 3 under yaks-a3d2, shaves two yaks, and writes each claim note (the spec).
```
$ yaks list --parent-of yaks-a3d2 --status shaving
  [S] yaks-85d9  p3 task     yaks sheds counts a created yak's notes too (matches yaks changes) [cli]
  [S] yaks-886a  p3 feature  Shed names from Delta thread titles, slugged like a git worktree name [cli,delta]
```

###    One claim note, as the worker will read it
```
$ yaks show yaks-85d9 | sed -n '/Claim/,/WALKTHROUGH/p'
Claim (coordinator). Owner: notes-1. Decided by Joel (2026-10-09, "SGTM" on dedupe-1's recommendation in yaks-5138): `yaks sheds` counts a created yak's notes a
Change: in `sheds::compare_farms`, drop the `else` that skips a created yak's notes (count `c.notes.len()` for added yaks too), update the doc comment, update e
Scope: src/sheds.rs (compare_farms and tests only), tests/changes.rs if it asserts the old count, docs. Out of scope: everything else; another worker is editing
Evidence: a unit test where a created yak with 2 notes counts as 2 new notes (fails before). Judge: coordinator.
WALKTHROUGH STEP (contrived on purpose, for a demo of the cross-shed ask/answer loop): before you change any code, run `yaks ask <this yak> --note "Mechanics ch
```

### 2. Commit the claims, so every worker's clone starts from them
```
$ git log --oneline -1
dba1e31 yaks-a3d2: Joel's answers picked up; claim wave 3 (yaks-886a titles, yaks-85d9 notes count); AGENTS.md: delegating to cheaper models
```

### 3. Brief. yaks brief prints every clause the yak, config and farm mode determine; the yakherd appends the task-specific lines.
```
$ yaks brief yaks-85d9 --as notes-1 | head -8
You are worker `notes-1`. You own exactly ONE yak: `yaks-85d9` - "yaks sheds counts a created yak's notes too (matches yaks changes)". It is already `shaving`.
Do not shave, shear, regrow or edit any other yak.

Commands
- Run every yaks command as `YAKS_ACTOR=notes-1 ~/src/yaks/.delta/worktrees/jnc4mb98btw6/yaks/target/release/yaks <args>`; each terminal call is a fresh shell, 
- Use that binary, or the one the coordinator names; never npx or an installer.

Start
```

### 4. Right after spawning: the new clones are already sheds (Delta pinned them on creation), named by dir id until a first note
```
$ yaks sheds | grep -E '^(KIND|delta|worktree)|name:|who:'
KIND      BRANCH  HEAD     +AHEAD -BEHIND  DIRTY  FARM ACTIVE           PATH
worktree  main    aa23381  +0 -1           0      2026-10-09T21:05:40Z  ~/src/yaks
  name: main
delta     main    dba1e31  +0 -0           0      2026-10-09T22:37:52Z  ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks
  name: 2wqa4188f15r
delta     main    8793594  +0 -21          0      2026-10-09T20:35:43Z  ~/src/yaks/.delta/worktrees/5a644x273r07/yaks
  name: 5a644x273r07
delta     main    dba1e31  +0 -0           0      2026-10-09T22:37:52Z  ~/src/yaks/.delta/worktrees/606y05kk15w7/yaks
  name: 606y05kk15w7
delta     main    b07aadf  +0 -11          0      2026-10-09T20:52:15Z  ~/src/yaks/.delta/worktrees/73gvtasv9rjr/yaks
  name: 73gvtasv9rjr
delta     main    360fbfb  +0 -21          0      2026-10-09T20:36:24Z  ~/src/yaks/.delta/worktrees/99yx2fj5eh45/yaks
  name: 99yx2fj5eh45
delta     main    2594cd6  +0 -24          0      2026-10-09T20:57:21Z  ~/src/yaks/.delta/worktrees/dv4fmmmdm74z/yaks
  name: dv4fmmmdm74z
delta     main    3d4dd1e  +0 -11          0      2026-10-09T20:52:42Z  ~/src/yaks/.delta/worktrees/h4skc0fpebk8/yaks
  name: h4skc0fpebk8
delta     main    ffd1714  +0 -21          0      2026-10-09T20:31:53Z  ~/src/yaks/.delta/worktrees/nw0v12c3ymte/yaks
  name: nw0v12c3ymte
delta     main    d237d16  +0 -11          0      2026-10-09T20:53:54Z  ~/src/yaks/.delta/worktrees/tv2dwd6ghd1a/yaks
  name: tv2dwd6ghd1a
```

### 5. notes-1 returned blocked. Delta also applied its ask into the yakherd's tree (the skill: leave it alone, land by SHA later)
```
$ git status --short
 M .yaks/shaving/yaks-85d9.md
```

### 6. Read the question from here, without landing anything
```
$ (cd ~/src/yaks && yaks inbox --sheds --for human | sed -n '/^Shed/,$p')
Shed notes-1 (~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks):
  [S] yaks-85d9  p3 task     yaks sheds counts a created yak's notes too (matches yaks changes) [cli] ⚠ needs:human
      Started: Fri Oct  9 22:38:33 UTC 2026
      yaks path: ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks/.yaks/shaving/yaks-85d9.md
Shed Joel Webber (~/src/yaks/.delta/worktrees/jnc4mb98btw6/yaks):
  [S] yaks-85d9  p3 task     yaks sheds counts a created yak's notes too (matches yaks changes) [cli] ⚠ needs:human
      Started: Fri Oct  9 22:38:33 UTC 2026
      yaks path: ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks/.yaks/shaving/yaks-85d9.md
```

### 7. Per-yak detail for that shed (named by dir id: notes-1 wrote its first note, so its actor name works too)
```
$ yaks changes notes-1
notes-1  ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks
  changes:
  yaks-85d9  yaks sheds counts a created yak's notes too (matches yaks changes)  [shaving]
    2026-10-09T22:38:36Z  notes-1  asked: needs human
    2026-10-09T22:38:41Z  notes-1  Started: Fri Oct  9 22:38:33 UTC 2026
    needs: human
```

### 8. Scope/mechanics, so the yakherd answers IN THE SHED: the reply lands in notes-1's copy, uncommitted there
```
$ YAKS_ACTOR=delta-lead yaks answer yaks-85d9@notes-1 --note 'Yes, as the yakherd: update that assertion; the JSON notes[] for a created yak now carries its count.'
Wrote the answer into shed notes-1 at ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks/.yaks/shaving/yaks-85d9.md (left uncommitted there; the shed's worker commit
Answered yaks-85d9: needs agent (awaiting pickup; see `yaks inbox`)
```

### 9. The worker's clone now holds the answer, uncommitted; the yakherd's own copy is unchanged by the answer
```
$ git -C ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks status --short; grep '^needs' ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks/.yaks/shaving/yaks-85d9.md
 M .yaks/shaving/yaks-85d9.md
needs: agent
```

### 10. And from the human's checkout it now reads as answered, awaiting the agent
```
$ (cd ~/src/yaks && yaks inbox --sheds --for agent | sed -n '/^Shed/,$p')
Shed notes-1 (~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks):
  [S] yaks-85d9  p3 task     yaks sheds counts a created yak's notes too (matches yaks changes) [cli] ✓ answered (needs:agent)
      Yes, as the yakherd: update that assertion; the JSON notes[] for a created yak now carries its count.
```

### 11. Wake the same worker (send_agent_message to its agent id). It resumes with yaks inbox --for agent and yaks pickup.

Observation: from `~/src/yaks`, `inbox --sheds` listed the ask twice: once from notes-1's own clone, and once from the yakherd's clone, which carries it only because Delta applied notes-1's state there (step 5). The yakherd's clone is named `Joel Webber` because Joel is the first actor on its notes (he answered yaks-a3d2 there). title-1's thread-title names (yaks-886a) fix that name. The duplicate goes away when the yakherd lands notes-1 by SHA.

### 12. notes-1 finished. Delta fast-forwarded the yakherd's HEAD to its commit (one of the landing shapes the Delta skill lists)
```
$ git log --oneline -2; git status --short
8ee705f yaks-85d9: yaks sheds counts a created yak's notes too (matches yaks changes)
dba1e31 yaks-a3d2: Joel's answers picked up; claim wave 3 (yaks-886a titles, yaks-85d9 notes count); AGENTS.md: delegating to cheaper models
```

### 13. The worker's commit carries the yakherd's answer, the pickup and the shear: one committed history for the yak
```
$ yaks show yaks-85d9 | grep -E '^status|▸' | tail -6
▸ 2026-10-09T22:39:51Z [delta-lead]
▸ 2026-10-09T22:40:18Z [notes-1]
▸ 2026-10-09T22:43:00Z [notes-1]
▸ 2026-10-09T22:44:27Z [notes-1]
▸ 2026-10-09T22:45:17Z [notes-1]
▸ 2026-10-09T22:45:17Z [notes-1]
```

### 14. Re-run the gate here (14 suites ok), then check the claim from the human's checkout: the yaks this clone CREATED now count their notes (yaks-440d +1, yaks-85d9 +10, yaks-886a +2)
```
$ (cd ~/src/yaks && yaks sheds) | sed -n '/jnc4mb98btw6/,/yaks-a3d2 +/p'
delta  main    8ee705f  +2 -0           0      2026-10-09T22:45:30Z  ~/src/yaks/.delta/worktrees/jnc4mb98btw6/yaks
  name: Joel Webber
  who: Joel Webber, delta-lead, delta:Delta :Yaks (cont'd), notes-1 · shaving: yaks-886a, yaks-a3d2
  farm: 3 new, 16 new notes
    + yaks-440d
    + yaks-85d9
    + yaks-886a
    yaks-440d +1 notes
    yaks-85d9 +10 notes
    yaks-886a +2 notes
    yaks-a3d2 +3 notes
```
### 15. The duplicate ask from step 6 is gone: notes-1's copy is answered and landed here
```
$ (cd ~/src/yaks && yaks inbox --sheds | grep -c yaks-85d9)
0
```


### 16. title-1 lands (by cherry-pick: both workers rewrote the same docs/cli.md row; resolved by hand, keeping both)
```
$ git log --oneline -1
49c2d5d yaks-886a: name Delta clones by the slug of their thread title
```

### 17. Any yaks command run in a Delta clone now records the thread title in that clone's OWN git config (never committed)
```
$ yaks list --status shaving >/dev/null; git config --get yaks.shed.title
Delta :Yaks (cont'd)
```

### 18. Sheds are now named by a slug of the thread title (workers that ran a new-enough binary: title-1 built its own)
```
$ (cd ~/src/yaks && yaks sheds | grep -E 'name:')
  name: 2wqa4188f15r
  name: 5a644x273r07
  name: title-1-shed-names-from · title: title-1: shed names from Delta thread titles
  name: 73gvtasv9rjr
  name: 99yx2fj5eh45
  name: dv4fmmmdm74z
  name: h4skc0fpebk8
  name: delta-yaks-cont-d · title: Delta :Yaks (cont'd)
  name: nw0v12c3ymte
  name: tv2dwd6ghd1a
```

### 19. shed-1 just spawned. Its first yaks command (run with the binary it built from the current code) recorded its spawn title, so `sheds` names it by that from the start; older finished clones keep their dir ids
```
$ (cd ~/src/yaks && yaks sheds | grep -E 'name: shed-1|name: [0-9a-z]{12}$' | head -12)
  name: 2wqa4188f15r
  name: 5a644x273r07
  name: 73gvtasv9rjr
  name: 99yx2fj5eh45
  name: dv4fmmmdm74z
  name: h4skc0fpebk8
  name: nw0v12c3ymte
  name: tv2dwd6ghd1a
  name: shed-1-yaks-shed-global · title: shed-1: yaks --shed global flag
```

### 20. shed-1 lands (another fast-forward); gate 16 suites ok here
```
$ git log --oneline -1
6f1f988 yaks-440d: global --shed <name> runs any command in another shed (-C by name)
```

### 21. --shed <name> runs any command in another shed: here, the human's checkout ('main') from inside the yakherd's clone
```
$ yaks --shed main show yaks-440d | head -3
id:       yaks-440d
title:    yaks --shed <name>: run any command in another shed (full access; -C by name)
status:   Shaving
```

### 22. ...and a worker's shed by its title slug: shed-1's clone still has yaks-440d shorn there
```
$ yaks --shed shed-1 list --status shorn | grep yaks-440d
  [N] yaks-440d  p3 feature  yaks --shed <name>: run any command in another shed (full access; -C by name) [cli]
```

### 23. A name that matches nothing lists the sheds and exits 1
```
$ yaks --shed nope list; echo exit=$?
error: no shed matches `nope`; the sheds are:
  main  ~/src/yaks
  2wqa4188f15r  ~/src/yaks/.delta/worktrees/2wqa4188f15r/yaks
  5a644x273r07  ~/src/yaks/.delta/worktrees/5a644x273r07/yaks
  title-1-shed-names-from  ~/src/yaks/.delta/worktrees/606y05kk15w7/yaks
  73gvtasv9rjr  ~/src/yaks/.delta/worktrees/73gvtasv9rjr/yaks
  99yx2fj5eh45  ~/src/yaks/.delta/worktrees/99yx2fj5eh45/yaks
  dv4fmmmdm74z  ~/src/yaks/.delta/worktrees/dv4fmmmdm74z/yaks
  h4skc0fpebk8  ~/src/yaks/.delta/worktrees/h4skc0fpebk8/yaks
  nw0v12c3ymte  ~/src/yaks/.delta/worktrees/nw0v12c3ymte/yaks
  tv2dwd6ghd1a  ~/src/yaks/.delta/worktrees/tv2dwd6ghd1a/yaks
  shed-1-yaks-shed-global  ~/src/yaks/.delta/worktrees/vks97kdt9jpt/yaks

exit=1
```

### 24. --shed does NOT stamp the caller's title into the target (the recording is skipped for -C and --shed)
```
$ git -C ~/src/yaks config --get yaks.shed.title || echo '(no title recorded in the main checkout)'
(no title recorded in the main checkout)
```

### 25. Push: preflight checks local is the human's checkout, then main moves (and GitHub with it)
```
$ yaks preflight --push-main | tail -1
preflight: ok
```
