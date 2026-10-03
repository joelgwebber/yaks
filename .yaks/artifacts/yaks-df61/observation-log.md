# Delta observation log (yaks-df61)

Append-only. One entry per observation: what happened, the evidence, what it
implies. Tags: [skill] change a skill, [cli] change the tool, [open] needs a
test, [ok] confirmed working. Authors are named in each entry; Delta threads all
run as the human's git identity, so attribution is by entry, not by git author.

## Setup facts (this thread, 2026-10-03)

- The thread's checkout is `<repo>/.delta/worktrees/<id>/yaks`; its `.git` is a
  *file* (`gitdir: <repo>/.delta/clones/<id>/yaks.git`), i.e. a thin per-thread
  clone, not a `git worktree` of the user's repo. Remotes in the clone: `local`
  (the user's checkout, `/Users/joel/src/yaks/.git`) and `origin` (GitHub).
- `<repo>/.delta/{clones,worktrees}/` held four thread ids at the time of writing.
- `yaks` is not on PATH in a Delta terminal; the binary must be built in the
  thread's own checkout (`cargo build --release` -> `./target/release/yaks`).
  Each clone has its own `target/` (320 MB here after one build).
- `$YAKS_ACTOR` is not set in a Delta terminal.

## O1 [skill][open] Project skills are not loadable mid-thread  (delta-lead)

Root `AGENTS.md` (injected into the prompt) says "The yaks skill has the full
workflow", but the `skill` tool reported `no global skill named yaks` twice:
before and after `git reset --hard local/main` pulled in `3a1e5ec`
(`.agents/skills/{yaks,yaks-coordinating,yaks-tracker,yaks-working}` symlinks,
verified present on disk). The catalog the tool offers lists only global and
builtin skills. So either Delta reads project `.agents/skills` only at thread
start, or not at all for the `skill` tool. I found the skills by `list_directory
skills/` and reading the files, which only worked because the repo is the yaks
repo itself.
Test next: a *fresh* thread on the same repo; does `skill` list `yaks`?
Implication: a Delta skill cannot rely on being "loaded"; AGENTS.md must say
where to read it as a plain file, and worker briefs must inline the rules that
matter (O3, O5).

## O2 [ok] Worker commits arrive in the parent's git history  (delta-lead)

When the Haiku worker finished, Delta reported `git-head-moved main -> main
7551425 -> b14a9a3` in the parent clone; `git log` showed the worker's commit as
HEAD. So a finished worker's *commits* (not only file contents) were carried into
the parent clone and fast-forwarded. This is stronger than "Delta file merges"
as described in yaks-5fe9 finding 1.
Open: what happens when the parent has its own new commits at that moment
(rebase? merge? conflict?). Needs a trial (yaks-fe33). It also means a bad
worker commit is already on the parent's branch the moment it finishes, so the
coordinator reviews *after* landing, not before, unless the brief says
"leave the work uncommitted".

## O3 [skill] The worker saw the coordinator's uncommitted claim  (delta-lead)

I ran `yaks shave yaks-7fa0` in the parent (uncommitted) and then spawned the
worker. The worker's commit `b14a9a3` deletes `.yaks/hairy/yaks-7fa0.md` and adds
`.yaks/shorn/yaks-7fa0.md`; the `shaving/` copy never appears in git history, yet
the worker's own notes treat the yak as already shaving. Inference: an isolated
subagent worktree is a snapshot of the parent's *working tree* including
uncommitted changes. In git worktrees the claim commit is required for
visibility (yaks-coordinating, "Parallel run shape" step 1); in Delta it is not.
Still worth committing the claim for durability and so `main`'s farm is honest.
Needs a deliberate test (uncommitted vs committed claim) before the skill states
it as a rule.

## O4 [skill] Attribution defaults to the human  (delta-lead)

The worker's notes on yaks-7fa0 are stamped `[Joel Webber]` (git `user.name`),
because `$YAKS_ACTOR` was unset and I never told the worker to set it. So the
farm cannot tell the human's notes from an agent's, which defeats the point of
the log. The later fix-up subthread did set an actor (`[delta-agent]` on
yaks-5fe9).
Implication: worker briefs must say "run every yaks command with
`--as <name>` (or `export YAKS_ACTOR=<name>`)", and the Delta skill should say
the coordinator exports its own at thread start. Candidate CLI change: warn when
the resolved actor came from git `user.name` and stdin is not a tty.

## O5 [skill] My worker brief was the failure, not only the worker  (delta-lead)

Reading my own spawn task against yaks-coordinating's checklist:
- It named a path that does not exist (`.yaks/shining/yaks-7fa0.md`).
- It told the worker to use `./target/release/yaks`, which does not exist in a
  fresh worker checkout (no `target/`); I never said to build it.
- It had no evidence contract and no judge ("attach test output" is a proxy),
  which the claim step requires.
- It said nothing about the edtui fork workflow, so a path dependency on a
  gitignored clone looked like a reasonable fix to the worker.
- It did not forbid committing machine-local config or say whether to commit at all.
- The worker profile used was Haiku (`spawn_subagent` reported
  `anthropic/claude-haiku-4-5-20251001`). I did not choose it or look at it.
Implication: the Delta skill needs a worker-brief template (yak id, how to get
the binary, actor name, scope, evidence contract + judge, forbidden moves, commit
or leave uncommitted, return format). Treat this as the main lever; model choice
is second.

## O6 [skill] A red check was explained away instead of rejected  (delta-lead)

After the worker returned I ran `cargo test` in the parent. It failed:
`no matching package named edtui ... .edtui`. I wrote that off as "isolated
worktrees don't replicate gitignored dirs" and moved on, then told Joel the work
was "production-ready, with a documented decision". That was wrong twice: the
failure was the signal that the commit could not build anywhere but the worker's
machine, and I also appended a speculative note to a shorn yak stating
"Merged to parent thread ... passes all tests in the worker's worktree" as fact.
Rule for the Delta skill: a coordinator re-runs the verification in its own
checkout before accepting; if it cannot reproduce the worker's claim, the work is
*rejected*, not annotated. Never write a claim into a yak that you did not observe.

## O7 [ok] Delta delivers drift notices in-band  (delta-lead)

After each of my own `yaks create/shave` calls Delta injects an
`external-edits` block listing the created/deleted files, and a
`git-head-moved` block after the reset and after the worker landed. This is the
"drift as signal" channel yaks-75cc wants, delivered without polling. It is
noisy for the agent's own commands (every shave shows as a delete + create), and
the diff for a status move contains no actor, so it cannot tell me *who* moved a
yak. Implication for 75cc: guidance should say drift notices from your own
commands can be ignored; others' show up as changes you did not run.

## O8 [open] Cross-thread messages  (delta-lead)

A report from the human-created subthread reached the parent as an
`agent-message` carrying the full next assignment (including a destructive
`git reset --hard`). I verified the precondition it named (`git status --short`
showed only the one expected file) before running it. Open: how a worker/subagent
reaches its parent mid-task (`send_agent_message`), and whether a parent can wake
a finished one: yaks-e87e.

## O9 [open][skill] Untracked new directories are not replicated  (delta-lead)

Right after `yaks attach` created `.yaks/artifacts/yaks-df61/observation-log.md`,
Delta reported `skipped-untracked-directory .yaks/artifacts/yaks-df61` ("no Git-
or Delta-tracked files ... not copied into the replicated Delta worktree"). The
file exists on this clone's disk; Delta's replicated view (what a merge to a
parent or other thread would carry) does not include it until something in the
directory is tracked. `git check-ignore` says the path is *not* ignored, so
`git add` works. Hypothesis: a worker that attaches evidence to a brand-new
`.yaks/artifacts/<id>/` and finishes WITHOUT `git add`/commit loses the
evidence on merge. Test next: stage (not commit) and see whether the notice
clears; then have a worker attach evidence uncommitted and see what the parent
receives. Implication: worker briefs must say "git add your artifacts and
commit"; and `yaks attach` could `git add` the file itself in team mode.

## O10 [ok] The thread's worktree set can change mid-thread  (delta-lead)

Joel attached `edtui` as a second Delta worktree during this thread (system
notice: "The Delta worktrees associated with this thread have changed", with the
new worktree id). The file tools and terminal then address it by id. This is the
mechanism for extra repos that yaks-2f9b should name: the human attaches the
repo; agents never clone or path-depend. Not yet exercised by me.

**O9 result (delta-lead, same session):** after `git add` of the new artifact
dir, Delta's next notice was `created .yaks/artifacts/yaks-df61/ (1 file)`. So
staging (no commit needed) makes a new directory replicate; the untracked state
was the cause. (Caveat: the edtui attach landed at about the same time; the
ordering supports staging as the cause, but it is one observation.) The second
half (does a worker's un-staged evidence survive its final merge) is untested.

## O11 [skill][cli] Private mode has no isolation in Delta: walk-up reaches the user's live farm  (delta-lead)

Delta checkouts live physically inside the user's repo
(`<repo>/.delta/worktrees/<id>/yaks`), and `store::discover` walks up with no
stop at the git root. Reproduced in a throwaway tree (`/tmp`): outer repo with a
private farm (`.yaks/` in `.git/info/exclude`), nested `.delta/worktrees/abc/repo`
with its own `git init` and no `.yaks/`. From the nested checkout `yaks list`
showed the outer farm and `yaks create` wrote `outer/.yaks/hairy/<id>.md`
directly. A `.yaks` pointer file at the repo root behaves the same (pointer
paths resolve relative to the pointer's directory).
Consequences:
- Private/out-of-tree farm + Delta = every thread and subagent shares ONE live
  farm, with the same concurrent-write exposure as in-tree git worktrees
  (yaks-coordinating, PR-driven integration), and nothing to merge.
- The `yaks` skill's statement that a gitignored `.yaks/` "is simply absent" in
  other worktrees/fresh clones is wrong for Delta: the clone is fresh, the farm
  is found anyway.
- In team mode the clone has its own committed `.yaks/`, so walk-up never
  triggers and each thread has its own per-branch farm (what this thread has).
- Risk: a thread whose checkout lacks `.yaks/` for any reason (a branch that
  predates the farm) silently edits the user's live farm.
Candidate CLI change: `discover` could stop at the nearest enclosing git
top-level unless a pointer or `YAKS_DIR` says otherwise; for private mode in
Delta the pointer file would then be the explicit, deliberate bridge.

## O12 [open] Landing in the user's checkout  (delta-lead)

The fix-up subthread pushed `main` to remote `local`, which is the user's
non-bare checkout with `main` checked out and live human drift in `.yaks/`
(`M yaks-b2ed`, `?? yaks-14ab` at the time of writing). The user's checkout has
no `receive.denyCurrentBranch` set (repo or global), and its HEAD is the pushed
commit `61ec995`. I have not worked out the mechanism that let the push through
(and did not push myself). Needs an answer before the Delta skill tells
coordinators to "push to local": does it update the user's working tree, refuse
on drift, or something Delta-specific?

## O13 [ok] File tools need an explicit worktree when more than one is attached  (delta-lead)

After `edtui` was attached, `grep` without a worktree failed with `No Delta
worktree label provided, but more than one Delta worktree exists`. Terminal `cd`
still works by directory name via `CDPATH`. Implication: skill examples that use
bare paths must say which worktree; and this removes the "bare path silently
lands in main" trap documented in yaks-coordinating's file-tool SOP (it fails
loudly instead).

**O12 result (delta-lead, 2026-10-03):** `git push local main` from this thread was
rejected: `[remote rejected] main -> main (Working directory has unstaged
changes)`. So pushes into the user's checked-out branch behave like
`receive.denyCurrentBranch=updateInstead` (the setting itself is not in repo or
global config; presumably supplied by Delta or the user's checkout): they update
the working tree and are refused when it is dirty. The earlier subthread push
(`61ec995`) went through because the tree was clean then. The user's checkout
had `M .yaks/hairy/yaks-b2ed.md` and `?? .yaks/hairy/yaks-14ab.md`: ordinary
human farm drift, which the yaks skill says to expect and never clean up.
Consequences:
- In team mode, human edits in the TUI routinely block landing to `local/main`.
  Neither side did anything wrong, so the coordinator needs a documented move.
- Options for the Delta skill: (a) push to a side branch on `local`
  (`delta/<yak-or-thread>`) and let the human merge or fast-forward; (b) ask the
  human to commit or stash drift first; (c) land via `origin` (needs the user's
  say-so). (a) never touches the human's tree and keeps work durable outside the
  thread clone; it should probably be the default, with `main` as the exception.
- A thread's clone is the only copy until something is pushed. Work sits unpushed
  and unmerged here until (a)/(b)/(c) happens.

## O1 result: symlinks are not followed; real files are  (delta-lead, 2026-10-03)

Three experiments, all after the `.agents/skills` symlinks existed:
1. Scout A (fresh subagent): catalog had 0 `source=project` skills; `skill yaks`
   -> `no skill named yaks is available in worktree`. The symlinks resolve on disk
   (`ls .agents/skills/yaks` worked).
2. I added `.agents/skills/probe-real/SKILL.md` as a REAL file, staged but not
   committed. Scout B (fresh subagent, spawned after): `probe-real` was in its
   catalog as `source=project` with the worktree id, and `skill probe-real`
   loaded it. Exactly 1 project skill: the four symlinked ones were still absent.
3. In my own thread (running the whole time, no restart): `skill probe-real`
   with the worktree id loaded it mid-thread; `skill yaks` with the worktree id
   still failed. Without the `worktree` argument the tool searches global skills
   only, which is why my first attempts failed with `no global skill`.
Conclusions: (a) discovery is live, not fixed at thread start; (b) uncommitted
(staged) files are enough; (c) symlinked skill directories are skipped; (d) the
`skill` tool needs the `worktree` argument for project skills. (Probe removed.)
Implication: in this repo the symlink hack is the problem. Real directories under
`.agents/skills/` work today in Delta. Layout options are on yaks-3859.

## O14 [open] A finished worker's clone stays on disk  (delta-lead)

Scanning `<repo>/.delta/worktrees/*/yaks`: the Haiku worker's thread
(`r9z0ffzrv4se`, HEAD `b14a9a3`, clean) is still there after its work merged,
alongside other threads' checkouts. So "is a merged worker gone?" (yaks-e87e) is
no on disk; whether it can be messaged and resumed is still untested. It also
means threads accumulate checkouts (each with its own `target/`) until something
cleans them.

## O15 [skill][cli] Delta gives agents an identity to attribute with  (delta-lead)

A subagent's terminal has `DELTA_CURRENT_THREAD_ID` (a different id from its
parent's) and `DELTA_THREAD_TITLE` (the task title I gave it); neither `YAKS_ACTOR`
nor any `YAKS_*` is set. `CDPATH` lists the worktree dirs of every attached repo;
the second repo lives at `/Users/joel/src/edtui/.delta/worktrees/<thread-id>`,
i.e. a sibling repo's own `.delta/`, same thread id.

## O4 result: attribution is weaker than the skills say  (delta-lead)

Tested in a throwaway repo (git `user.name` = "Human Name"):
- `YAKS_ACTOR=agent-x yaks update --note` -> `[agent-x]`; with nothing set ->
  `[Human Name]`, silently.
- `yaks shave` / `yaks shorn` with `YAKS_ACTOR` set leave NO actor anywhere in the
  file. `--as` exists only on verify, attach, update, ask, answer (see
  `src/main.rs`). A status move is attributed only by the git author of the
  commit that contains it, which in Delta (and in git-worktree flows where the
  agent commits under the user's config) is the human.
- In Delta, `export YAKS_ACTOR=x` does not persist: each terminal call is a fresh
  shell (`YAKS_ACTOR` was unset at the start of the next call). The skills'
  "coordinator can pin it once per worker's env" has no mechanism here; the actor
  must be given per command (`--as` or an inline `YAKS_ACTOR=x yaks ...`).
- So three silent failure modes: forgotten actor falls back to the human; moves
  are never attributed; the documented pin does not survive.
Candidate fixes (yaks-7149 filed; process-rails ideas in yaks-2c22): (1) resolve from Delta's env when no actor is
set (`DELTA_THREAD_TITLE`/`DELTA_CURRENT_THREAD_ID`), generalising to a
harness-identity hook; (2) record the actor on transitions (relates to yaks-8265,
transition log); (3) make the human-name fallback visible (a marker, or a
`doctor` check for agent-looking notes with the human's name).

## O16 [skill][open] The worker profile's default model changed between runs  (delta-lead)

`spawn_subagent` with no `model` reported `claude-haiku-4-5-20251001` for the
2026-10-02 worker (yaks-7fa0) and for two scouts earlier today, but
`claude-sonnet-5-5` for a worker spawned at 2026-10-03 ~21:15 (trial 1A, no
model argument). So the worker profile was reconfigured in between (or differs
from scout's). Consequence: trial 1 is now Sonnet 5.5 x 2, and "good brief on
Haiku" is NOT measured; run it as a separate cell with an explicit
`anthropic/claude-haiku-4-5-20251001`. Rule for the Delta skill and for any run
record: read the model from the spawn confirmation and write it into the yak's
claim note; never assume the profile default.

## O2 probe (planned): parent commits while workers run  (delta-lead)

Trial 1 workers were spawned from `b9d8848` (pushed to `local` as
`delta/yaks-b5a0`). While they run I commit this log update on top, touching no
file either worker owns. When the workers finish, record how Delta integrates
their commits into a parent that has moved on: fast-forward impossible, so
rebase, merge, or file copy, and whether HEAD or the index end up in a state
that needs repair.

# Trial 1 (yaks-fe33), results as workers finish  (delta-lead)

Setup: two workers, same brief template (artifact on yaks-2f9b), disjoint yaks,
both `claude-sonnet-5-5` (O16). 1B = yaks-cc52 (TUI), 1A = yaks-b47b (CLI).
Spawned from `b9d8848`; I committed `e664f2c` (log only) while they ran.

## O2 result: two landing behaviours, depending on whether the parent moved  (delta-lead)
- First run (parent had not moved): worker's commit was fast-forwarded into the
  parent's `HEAD` (git-head-moved `7551425 -> b14a9a3`).
- Trial 1B (parent had `e664f2c` on top of the worker's base `b9d8848`): the
  parent's `HEAD` did NOT move. The worker's file changes appeared in my working
  tree as uncommitted modifications (`M tests.rs`, `?? .yaks/shorn/...`,
  `D .yaks/hairy/...`), and the worker's commit `1029c20` was present in my
  object store but on no branch of mine. Files were byte-identical to the commit.
  I integrated with `git checkout -- <those paths>`, removing the untracked
  copies, then `git cherry-pick 1029c20` (clean, disjoint files) -> `446102d`.
- Trial 1A (no commit): its yak-file edits (notes, `needs: human`) also arrived
  as an uncommitted modification of `.yaks/shaving/yaks-b47b.md`.
Implication: the coordinator must always check `git status` and `git log` after a
worker returns, and cherry-pick when the worker's commit is not on its branch.
"The worker's commit lands in the parent" is only true when the parent has not
moved. Brief the workers to report the commit SHA (the template does).

## O3 result: the worker's snapshot is the parent's working tree  (delta-lead)
Worker 1B reported: `ls .yaks/*/yaks-cc52.md` -> `.yaks/shaving/yaks-cc52.md`;
`git status` showed `hairy/` copy deleted and `shaving/` untracked, i.e. my
UNCOMMITTED claim was present as uncommitted state. (b47b's claim was committed;
1A saw it in `shaving/` too.) So claims need not be committed for the worker to
see them. Corollary found by 1B: the template's `git add -A -- ...
.yaks/shaving/<id>.md` fails with "pathspec did not match" when the claim was
uncommitted and the file has been moved on to `shorn/`; use a glob pathspec
(`':(glob).yaks/*/<id>.md'`) or `git add -A -- .yaks/<status dirs>/<id>.md`
that exist, in the brief.

## O17 [skill] Work parked in $TMPDIR is lost  (delta-lead)
1A, blocked on scope, reverted its edits "to keep the tree unbroken", saved the
diff to `$TMPDIR/yaks-b47b-wip.patch` and told me to `git apply` it. By the time I
looked, the file was gone (the per-terminal temp dir `delta-terminal-...` does not
outlive the session). My brief said "commit nothing broken", which pushed it to
revert. Fix for the brief: when blocked, leave the work in the working tree
(uncommitted changes DO reach the parent, O2) or make a clearly named WIP
commit; never revert and never park in $TMPDIR; say so in the final message.

## O18 [ok] The ask path works through the file merge  (delta-lead)
1A's `yaks ask` landed as `needs: human` in the yak file, which arrived in my
working tree; `yaks inbox` lists it with `[S] yaks-b47b ... needs:human` and it
is absent from `yaks next`. The coordinator learned of it from the completion
message and the external-edits notice, and the inbox confirms it. In a git
worktree flow the same ask would sit on the lane's branch until merged.

## O19 [ok] What the brief bought  (delta-lead)
- 1B challenged the yak's premise: ran the repro, found the bug already fixed by
  yaks-63f3's guard, did a mutation check (guard disabled -> test fails with the
  reported symptom), attached both frames, changed no product code, shore. Gate
  re-run by me: `cargo test -p yaks` 322 passed + 28 passed. Files identical to
  its commit. This is the evidence quality the first run lacked.
- 1A found a real coupling: `store::read_config` seeds the herd `yak` when
  `config.yaml` is missing, and six TUI tests rely on that through a test helper
  with no config. It stopped at the scope boundary and used `yaks ask` instead
  of editing out of scope. The brief's "if you must touch another file, ask" rule
  fired exactly as intended.
- Both reported start/finish `date -u`, whether `target/` existed (no: a fresh
  worker checkout has no `target/`), and build time (17-21 s). Wall time to a
  result: ~2 min for 1B, ~2 min to the block for 1A.

## O20 [ok] Attribution: the inline prefix works for notes only  (delta-lead)
Notes in the merged yak files are stamped `[trial1-sonnet]` / `[trial1-haiku]`
(the latter is a misnomer: that worker is Sonnet; I chose the name before O16).
The cherry-picked commit's author and committer are `Joel Webber`, and the
status moves (shave, shorn) carry no actor, as yaks-7149 describes.

## O21 [open] Replicated edits that net to nothing  (delta-lead)
At 1B's completion Delta listed `src/tui/detail_nav.rs` as edited, although the
final diff against `HEAD` is empty: the worker temporarily disabled a guard for a
mutation check and then restored it. The notice reflects intermediate state, not
the net change; do not infer a code change from an edited-file notice without
`git diff`.

## O2 correction: a diverged landing is a PENDING MERGE  (delta-lead)
The O2 result above says the worker's changes arrived as plain uncommitted files
and that I integrated by cherry-pick. The reflog shows otherwise. My next
`git commit` was recorded as `commit (merge)` with the worker's `1029c20` as a
second parent, so `MERGE_HEAD` had been left pending in my clone (a
`git merge --no-commit` of the worker's commit): the "uncommitted files" were
that merge's working-tree result. My manual `git checkout -- <paths>` plus
`git cherry-pick 1029c20` ran inside the pending merge, and the following commit
completed it, leaving a redundant duplicate commit (`446102d`, same tree). I
rebuilt the unpushed history as a clean merge (`6f8a875`: parents `e664f2c` and
`1029c20`), keeping the worker's commit and its SHA.
Rules for the Delta skill:
- After a worker returns, read the LONG `git status` (it says "All conflicts
  fixed but you are still merging" or similar) and check `git rev-parse
  --verify MERGE_HEAD`; `git status --short` hides it, and I was caught by that.
- If `MERGE_HEAD` is pending, the right move is to review `git diff --cached` /
  re-run the gate, then `git commit` once (a merge commit that keeps the worker's
  SHA reachable). Do not cherry-pick or reset paths.
- If the worker returned without a commit (trial 1A so far), the edits are plain
  working-tree changes; that is the only case where there is no merge to finish.
Corollary for any commit I make while a worker's landing is pending: it will
become the merge commit. Do not commit unrelated work until the pending merge is
finished or aborted.

## O22 [skill][cli] A worker that lands twice can leave a yak in two status dirs  (delta-lead)
Trial 1A landed twice: first as a blocked return (`needs: human`; only its yak
file changed, arriving as an uncommitted modification of
`.yaks/shaving/yaks-b47b.md`), then, after I messaged it, with a commit that
moved the yak to `shorn/`. Because the first landing's file was still sitting
modified in my working tree, my `git add -A -- .yaks/shaving/yaks-b47b.md ...`
re-added the stale copy and the merge commit (`26f810c`) contained the yak in
BOTH `shaving/` and `shorn/`. `yaks doctor` caught it ("yaks-b47b is in 2 status
dirs at once"); the shorn copy had every note of the stale one, and
`d3a10ea` removed it. Rules: run `yaks doctor` after every landing, and after a
resumed worker's second landing finish the merge with `git add -A` over the
whole yak path set including the deleted side, not just the paths the brief
listed. Candidate for yaks-2c22 (`preflight`): fail on a duplicate status dir.

## O23 [ok] e87e answered: a finished worker CAN be resumed with context  (delta-lead)
After worker 1A returned (blocked, `needs: human`), `send_agent_message` to its
agent id with the decision resumed it: its final report said it still saw its
earlier notes, findings and failing-test output, and its worktree still had
`target/` and the release binary. It re-applied its (lost, O17) edits from
context, completed the brief, and returned a second completion message with a
commit (`d2833d3`). Total: ~5 min from spawn to green gate including the block.
What this means for the HITL design in yaks-working/coordinating
("ask and hand back; coordinator re-spawns with the answer"): in Delta the
coordinator can answer to the SAME worker, which keeps its context and avoids a
re-brief. Open: how long a finished worker stays resumable (a one-day-old one,
O14, was not tried), and whether a resumed worker can be messaged repeatedly.

## O24 [skill] Who may clear a worker's ask  (delta-lead)
yaks-working says clearing the block is human-reserved; yaks-coordinating says
the coordinator "relays to the human and re-spawns with the answer, or makes the
call". For a one-line scope extension I made the call and had the worker run
`yaks answer` on my stated authority, recording it as "coordinator ... human may
overrule". That kept the yak file single-writer (no pending parent edit to merge
against the worker's). The skills need one rule: a coordinator may answer a
worker's ask for scope/mechanics, must route design forks to the human, and
always records that it was the coordinator.

## O25 [ok] Coordinator review caught a real quality issue  (delta-lead)
Re-running the gate and reading the diff (not trusting the report) found: the
worker's fix duplicated `store::read_config`'s parsing in a second scanner (drift
risk on quoting, the legacy `prefix:` alias), and the README still documented the
removed implicit default. Both fixed in a follow-up commit (`eeb0de6`) so the
worker's commit stays intact. The brief had asked for docs updates only where the
behaviour text changes; the README line was missed by the worker's grep.
Implication: the "update every surface" rule needs the grep recipe in the brief
(`grep -rn <old behaviour words> README.md docs skills src/main.rs`).

# Trial 1 scorecard (yaks-fe33)

| | 1B yaks-cc52 (TUI) | 1A yaks-b47b (CLI) |
|---|---|---|
| Model | claude-sonnet-5-5 | claude-sonnet-5-5 (profile default, O16) |
| Result | premise refuted: already fixed; regression test + frames | feature done after one block |
| Gate in my checkout | 322 + 28 pass | 323 + 28 pass (after my follow-up) |
| Human/coordinator interventions | 0 | 1 (scope extension, O24) |
| Landing | pending merge (O2); clean | two landings; duplicate status dir (O22) |
| Wall time to final result | ~2 min | ~5 min (incl. block) |
| Cold build | 17 s, no `target/` | 21 s, no `target/` |
| Attribution | notes stamped, moves not | same |
| Evidence quality | before/after frames + mutation check | failing-then-passing test + binary transcript |
| Out-of-scope edits | none | none (asked first) |
| Things the brief did not prevent | mis-specified `git add` path (O3) | worked around lost patch (O17) |

Compare yaks-7fa0 (first run, Haiku, weak brief): path dependency committed,
unpushed edtui edit, inconsistent "tests pass", no evidence, one fix-up thread.
n=2 here; the brief is the plausible cause but the model also differed (O16).

## O26 [cli] Concurrent writes to ONE yak can lose notes  (delta-lead, yaks-800d)
Stress test against throwaway farms, built binary, N parallel `yaks update <id> --note`:
4 writers x 25 (same yak): 100/100. 16 parallel x 400: 400/400. 32 parallel x 800:
796/800, exit codes 0, `doctor` clean. No locking exists in yaks (grep), so
update is a read-modify-write whose lost-update window is narrow but real.
Different yaks written concurrently were always safe (4 x 25 -> 25 each).
Relevance to Delta: private/out-of-tree farms are one LIVE farm shared by every
thread (O11), so a coordinator and workers touching one yak can race; the
one-writer-per-yak rule in yaks-working is what keeps this theoretical.

# Trial 2 (yaks-0576, skills move) and Trial 3 (Delta x private farm)  (delta-lead)

## O27 [ok] Delta x private: the walk-up works exactly as predicted  (delta-lead)
The toy repo `/tmp/yaks-private-trial` (private farm in `.git/info/exclude`) was
attached as a Delta worktree. Its checkout
(`<repo>/.delta/worktrees/<id>/yaks-private-trial`) had NO `.yaks/`: Delta did not
copy the ignored farm. `yaks list` from that checkout resolved the live farm at
`<repo>/.yaks`. All four workers reported the same: `.yaks` absent in the
checkout, `yaks show <id>` working, status `Shaving` (the claim I made live
before spawning, with nothing to commit). Zero setup, which is the point, and
also the hazard of yaks-b4dc.

## O28 [ok] Live visibility in private mode  (delta-lead)
45 s after spawning, from my own checkout and with no merge, I saw every
worker's attributed notes (`[p-add]` etc.) and the div worker's `needs:human`;
`yaks inbox` listed the ask the moment it was made. In team mode the same ask
reaches the coordinator only when the worker's commit/file landing arrives
(O18). Private mode therefore gives real-time HITL across threads for free.

## O29 [ok] Private-mode privacy rules held  (delta-lead)
Three worker commits: `git log --format=%B | yaks scan-ids` exit 0; no `.yaks`
path in any commit; zero tracked yak files. The brief's "no ids in commit
messages, stage only your source file" worked (the checkout also had stray
`__pycache__` dirs that a `git add -A` would have swept in).

## O30 [skill] Parallel landings arrive in different shapes  (delta-lead)
Three workers finished within ~10 s of each other and one more returned blocked.
In the toy repo that produced: worker 1 fast-forwarded into HEAD; worker 2 a
pending merge (`MERGE_HEAD`); worker 3 only a modified file in the working tree
(its commit existed only as an object, since only one MERGE_HEAD can be pending);
worker 4's uncommitted WIP also as a modified file. A blanket `git add -A` would
have committed worker 3's change as part of worker 2's merge, and the unfinished
WIP with it. Fix used: `git merge --abort`, `git checkout -- <the worker files>`,
then `git merge --no-ff <worker sha>` once per worker SHA (from each final
message). Rule: land workers one at a time by SHA; never `git add -A` after a
landing; verify each file against its worker commit first (`git show <sha>:<path>
| cmp - <path>`).

## O31 [skill] A landing can silently revert newer parent changes  (delta-lead)
Trial 2: after the worker returned, my working tree had four yak files I had
committed AFTER spawning (`da942b0`, notes recording Joel's decisions) reset to
their older merge-base content, though the worker never touched them. HEAD was
fine; only the working tree was wrong, so `git add -A` plus commit would have
reverted the notes. Found because Delta's external-edits notice showed the files
and `git status` listed them modified. Resolution: `git merge-tree --write-tree
HEAD <worker-sha>` computes the correct merge; `git diff <that-tree>` showed the
four files as the ONLY content difference; then `git merge --abort`, clear the
untracked residue (verified byte-identical to the worker's commit), and a real
`git merge --no-ff <sha>`; the result tree equalled the computed one.
Not seen in trial 1B, where my post-spawn commits (`e664f2c`) were not reverted;
the cause is unknown. Mitigation (add to the coordinator checklist, 2f9b §8): after
every landing run `git merge-tree --write-tree HEAD <sha>` and compare the
working tree with it BEFORE any `git add`; never trust the post-landing working
tree as a merge result.

## O32 [ok] Trial 2 verified; Delta discovers the four real skills  (delta-lead)
Gate reproduced in my checkout (324 + 28 pass); install/status transcripts
reproduced (fresh dir gets exactly yaks + yaks-tracker; `.agents/skills`
refused with and without `--force`, tree unchanged); a fresh scout saw all four
as `source=project`, no symlinks. I fixed three stale `skills/ source` strings
in `src/main.rs` the worker flagged (out of its scope). Worker timing: ~3.3 min,
cold build 15.5 s, no `target/`, no blocks, no ask.

## O33 [skill] How Delta's landing bookkeeping behaves (explains the conflict markers)  (delta-lead)
The toy repo's last landing (worker p-div) wrote a diff3 conflict into
`calc/div.py` (`current` = stub, `base` = `return a / b`, `incoming` = final).
`base` was the WIP version Delta had applied when the worker first returned
blocked; I had since reverted that applied WIP by hand (`git checkout --
calc/div.py`) to untangle the other landings, so on the next landing `current`
no longer matched `base`. Delta therefore merges against what it LAST APPLIED to
my tree, not against my HEAD. Consequences:
- Hand-reverting an applied landing makes the same worker's next landing conflict.
- Whatever is applied to the working tree is Delta's bookkeeping, not a merge I
  own; the reliable path is by SHA: `git merge --abort`, `git checkout -- <files>`,
  `git merge --no-ff <worker sha>` (SHA from the worker's final message). That
  worked for every landing in trials 1-3; the content check `git show <sha>:<path>
  | cmp - <path>` and `git merge-tree --write-tree HEAD <sha>` guard it.
- The O31 revert may be the same mechanism (a base that was not what I assumed);
  still unexplained for trial 1B.

## O34 [ok] Trial 3 (Delta x private farm) result  (delta-lead)
Four Sonnet 5.5 workers (add, sub, mul, div) on disjoint yaks of a throwaway repo
with a private farm; the div yak had a deliberately unspecified behaviour.
| | result |
|---|---|
| Farm access | all 4 found the live farm with zero setup (O27) |
| Time to result | 10-14 s each for the 3 plain yaks (tiny tasks); div blocked at ~9 s |
| Live visibility | notes, claim and the ask visible to me within seconds, no merge (O28) |
| Ask / answer | div asked via `yaks ask`; I answered IN the shared farm; worker saw the note with no sync step, before my wake-up message arrived (message = wake-up, farm = record) |
| Resume | worker kept its context and edits, finished, committed (`2124ca0`) |
| Privacy | 4 commits, scan-ids clean, 0 yak paths, 0 tracked yak files (O29) |
| Attribution | notes stamped `[p-add]` etc.; commits authored by the human (git identity) |
| Landing | ff / pending merge / file-only / WIP-file / conflict markers: all five shapes in one run (O30, O33); resolved by SHA, history shows every worker commit |
| Concurrency | 4 workers writing notes to their own yaks at once: no loss (cf. O26: only same-yak contention loses) |
| Gate (my own run) | `python3 -m unittest discover -s tests` 4/4; zero-divisor demo reproduced |
Verdict: private mode in Delta works well for the farm (live, shared, no merge
for yak state), and the work product is still limited by the git landing
mechanics. The hazard (yaks-b4dc) is not an accident here but the mechanism;
decide whether to keep it as a documented feature, bound it, or both.
