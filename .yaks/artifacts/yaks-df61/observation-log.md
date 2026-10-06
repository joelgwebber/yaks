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

## O35 [open] What Delta's diff view compares against  (delta-lead, 2026-10-03)
Context: after trial 3 I merged a squashed PR branch to `local/main`; Joel then saw
the thread's diff vanish after changing the branch in Delta's project button, and
it did not come back.
Facts gathered (git only; I cannot see Delta's UI state):
- This clone's `origin/main` moved `3a1e5ec -> fb5cf3f` (reflog) without any push
  from me: GitHub's `main` is now `fb5cf3f` (`git ls-remote origin`), so the human
  pushed it and Delta refreshed `origin/main` here. The clone's `main` tracks
  `origin/main`.
- At that moment this clone's HEAD == `main` == `origin/main` == `local/main`
  (`fb5cf3f`): the thread had no changes relative to any of them.
- Differences from HEAD: vs the thread's original base `7551425` 49 files; vs
  `local/delta/yaks-b5a0` 27 files; vs `local/docs` 50 files.
Hypothesis: the thread diff is "this clone's HEAD (plus uncommitted changes)
against the project's/remote's branch tip", so it was non-empty until the squash
landed and the tip caught up, and it is empty for `main` now. This does not by
itself explain why other branches show nothing.
Experiment: this commit exists only in this clone (not pushed anywhere). If the
diff view is HEAD vs `main`/`origin/main`, it should show exactly this commit's
changes (this file and three yak files) and nothing else. Then switch the
project branch and see whether the view follows (compare `git for-each-ref` and
`git rev-parse origin/main` here before and after). Result to be appended below.
Rule for the Delta skill meanwhile: tell the human what "landed" means for the
diff view (a thread's diff is empty once its work is on the branch it is compared
with), and never describe a thread as "unmerged" from the diff view alone.

## O36 [ok][skill] Where a thread's commits live, and what lands them  (delta-lead)
Answer to Joel's question about the Delta UI after a turn.
- Every commit made in any thread or worker clone is mirrored into the USER's repo as
  its own ref, `refs/delta/<thread-id>/<repo>/<full-sha>` (56 such refs at the time).
  My commit `d697c89` appeared there seconds after I committed it, under my thread's
  id; worker commits `1029c20`, `d2833d3`, `8fd40f8` each have a ref under their own
  worker thread's id. None of them is on a branch in the user's repo, so the user's
  `main` did not move: the thread's HEAD is, from the user's side, a detached commit.
  (The human's inference was right.)
- Correction to what I told Joel earlier: worker SHAs cited in squash messages do NOT
  depend on my `delta/*` archive branches; Delta's refs keep them reachable in his
  repo. Whether those refs outlive a deleted or archived thread is unknown, so the
  archive branches remain a cheap belt-and-braces, not a necessity.
- What moves the user's `main`: in every case so far an explicit `git push local
  <branch>:main` by an agent (the user's checkout accepts it when it is on that
  branch and clean; any push to a branch that is not checked out always works). I
  know of no agent-side "accept" tool; `merge_thread` only moves changes between a
  parent and its child thread. Whether the UI has an accept/apply control is for the
  human to say.
- Visibility consequence (team mode): asks and notes made in a thread reach the
  human's `yaks inbox` only after `main` moves. Joel could not see the two new asks
  until I pushed `d697c89`; afterwards `yaks inbox` read from his checkout listed
  them (5 entries).
- GitHub `main` moved to `fb5cf3f` without any push from me: the human pushed, and
  Delta refreshed this clone's `origin/main` (reflog `3a1e5ec -> fb5cf3f`).
- Standing permission (Joel): branches I created on `local` that are merged and no
  longer needed (`pr/*`) may be deleted by me without asking.

## O37 [skill] First cold-read of the restructured skills  (delta-lead)
A fresh Haiku scout was given ONLY the three draft skills (core, team, private) and
eight scenarios, told to answer from the text and quote it, and to say NOT COVERED
otherwise. Result: 6 right, 1 partly right, 1 NOT COVERED, plus 3 ambiguities it
named. All are skill bugs, not reader errors:
- Two workers appending notes to one shared yak: NOT COVERED. The draft said "one writer
  per yak" but never that an append IS a write (the update rewrites the file), nor
  named the shared-parent-yak trap. Fixed in private.
- Who decides a worker's question: "scope and mechanics" vs "design fork" had no
  boundary, so the scout could not say which a test-helper edit is. Added a rule and
  examples to core; "if unsure, treat it as a fork".
- "An explicit pointer will bound it" did not say what it bounds or that it is not
  built yet. Reworded with the planned behaviour (yaks-b4dc) and what applies today.
- The scout also blurred "answer in the note says you are the coordinator" with "the
  worker is told by message"; both are in the text, in different places.
The check is cheap (one scout, ~2 minutes) and found real gaps in a fresh draft;
make it a gate for every skill change, and run a second pass on the full set before
the old yaks-coordinating is replaced.

# Trial 4 (yaks-5c9f): short briefs, workers load yaks-working themselves  (delta-lead)

## O38 [ok][skill] What the skills did without the brief, and the bug that hid them
Two Sonnet 5.5 workers, briefs of ~10 lines (yak id, name, scope, evidence, "load the
project skill yaks-working"), real yaks: 4A yaks-aa49 (`yaks path`), 4B yaks-800d (lock).
| behaviour the old briefs spelled out | 4A | 4B |
|---|---|---|
| built its own binary, no npx / PATH copy | yes | yes (noted the PATH copy and skipped it) |
| `YAKS_ACTOR` on every command | yes, notes `[t4-path]` | yes, `[t4-lock]` |
| stopped at a real design fork and asked, edits left alone, nothing parked | n/a | yes (lock location, `.gitignore`) |
| stayed in scope | yes (reverted `cargo fmt` ripple in 3 files) | yes |
| ONE commit incl. shorn move + artifact, pathspec ok | yes (`a46d866`) | n/a (parked) |
| evidence: tests + attached transcript of the built binary | yes | baseline reproduced: 554 of 25600 notes lost |
| reported what the skill lacked | yes (see below) | yes |
My re-run: 331 + 28 pass; scenario reproduced; code review clean; the gap was
update-every-surface (README table, the shipped skill's command list), which I fixed
as a follow-up. Landing: 4A was a clean fast-forward; 4B (no commit) arrived as a
modified yak file, committed byte-for-byte so a later landing merges against it.
Both workers followed the skill, NOT a brief, on every point the brief omitted.

## O39 [skill] The guard could not see what the harness could
Both workers reported `yaks-working` failed to load via the skill tool ("mapping values
are not allowed ... line 3 column 121"): my new description contained `: `, invalid in a
plain YAML scalar. Four of the five new skills had the same flaw. My guard test read
frontmatter line by line and passed. They read the file directly and followed it, which is
why behaviour did not suffer, but any reader relying on the catalog would not have got
the skill. Fixed by quoting, plus a guard rule (verified to fail on the unquoted form), and
all eight repo skills now load through the real tool. Lesson: validate a harness-facing
format with the harness (or a real parser), not a convenience read; and make loading the
skill through the real tool part of every skill change's evidence.

## O40 [skill] Delta merges against what it last applied: how to use that
Corollary of O33. A parked worker's applied edits are safe to COMMIT byte-for-byte (current
equals Delta's base, so its next landing merges cleanly) and unsafe to edit on top of
(current differs from base, so it comes back as conflict markers). So for a design-fork ask
on a parked worker's yak: commit its file as-is so the ask reaches the human's inbox, and
state the coordinator's lean in chat, not as a second note on that yak. Done for yaks-800d.

## O41 [skill] A human's uncommitted yak edits are a landing blocker  (Joel: "commit aggressively in public mode")
Joel's answers to three asks sat uncommitted in his checkout, so (a) his tree was dirty and
`git push local main:main` would be refused, and (b) my clone could not see the answers;
I could only read them from his working tree, read-only. Habit to adopt in public mode:
commit yak edits promptly (his "Yak herding." commits). Agent side: when his tree is dirty,
keep working on a branch and do not touch the yaks he has edited (a second writer produces a
merge conflict later; I touched one anyway and resolved the trivial append). Tool idea filed.

# Trial 5: three workers in parallel, landed with the new Delta skill  (delta-lead)
Workers (all `claude-sonnet-5-5`, short briefs): 5A yaks-12a0 (actor from Delta env), 5B yaks-06be (`yaks preflight`),
and the resumed 4B yaks-800d (farm lock). Each landed by SHA; three different failure shapes appeared.

## O42 [skill] A parked worker pins your history; a squash plus reset breaks its landing  (delta-lead)
4B was parked on a design ask, then resumed hours later. Meanwhile I had landed trial 4 as squashed commits and run
`git reset --hard local/main`, so 4B's base (`44a93ea`) was no longer an ancestor of my branch. When it returned: (a)
`MERGE_HEAD` pending, but `git merge-tree --write-tree HEAD <sha>` reported add/add CONFLICTS (merge-base was the old
`8433de1`); (b) Delta's applied working tree was the worker's old snapshot and had reverted 16 files I had since
changed (skills text, yaks-5c9f's notes and the human's note in it) and resurrected deleted yak files. Only the commit
itself was trustworthy. What worked: `git merge --abort`, restore the tree, `git cherry-pick <sha>` (applies only the
worker's own change; one modify/delete conflict on its yak file, resolved by taking the deletion after checking the
shorn copy held every note). Rule added to yaks-coordinating-delta (step 7): check
`git merge-base --is-ancestor <worker's parent> HEAD`; if not an ancestor, cherry-pick; and do not rewrite your branch
while a worker is running or parked.

## O43 [ok][skill] The merge-tree check earns its keep  (delta-lead)
5B: the applied tree differed from the correct merge in two ways the worker's commit did not contain: a stale
`shaving/yaks-06be.md` beside the new `shorn/` file (a duplicate status dir) and 16 lines of `cargo fmt` reformatting in
`src/skills.rs` that the worker had reverted before committing (intermediate state leaked into the landing). A blanket
`git add` would have committed both. Landing by SHA (abort, restore, remove verified-identical untracked leftovers,
`git merge --no-ff`) gave a tree identical to the computed one. 5A, by contrast, applied cleanly: the check passed and
the by-SHA merge was byte-identical. So the working tree after a landing is sometimes right and sometimes badly
wrong, and only the comparison tells you which.

## O44 [ok][cli] The farm lock, reviewed and measured  (delta-lead)
Head-to-head with the worker's before binary, same script (32 parallel updates, 800 notes, one yak): old kept 798 of
800, new 800 of 800, both 4 s, `doctor` clean. `store::lock` is std-only (`File::lock`), RAII, released by the OS on
crash; mutating `Farm` methods take it once at the outermost level (not re-entrant, documented in AGENTS.md; the worker
split `rename_many` to avoid self-deadlock). Residual risks, not acted on: a filesystem where locking fails makes every
write fail loudly (no unlocked fallback); Windows is untested; the first write in an existing committed farm creates an
untracked `.yaks/.gitignore` that `yaks preflight` will flag once until it is added.

## O45 [cli] Preflight's default scope was noise; fixed  (delta-lead)
As specified ("else all shorn yaks") it failed on dozens of old shorn yaks that never ran verify, on a clean tree:
useless as a gate. Now the verify scope is the ids given, else (with `--all`) every shorn yak, else the shorn yaks that
are part of the change in git (staged, modified or new, including a new `artifacts/<id>/`); a private farm, which git
cannot describe, still checks all. Three tests pin it (the scope test fails when the scope is removed). My own clone
now reports `preflight: ok`. Lesson: dogfood a gate on a real tree before accepting its default.

## O46 [open] A file I had just edited was overwritten with the human's version  (delta-lead)
Twice in this trial a file I had just edited changed under me. First, landing 4B (O42). Second, with no worker running:
within about 40 seconds of an edit, `yaks-coordinating-delta/SKILL.md` in my working tree became byte-identical to the
version on the human's `main`/`origin/main` (older than my commits), discarding one section that was already
COMMITTED and one I had just added, while every other modified file was untouched. `git status` and `git checkout --
<file>` recovered the committed text; the uncommitted step was lost and redone. Cause unknown (timing coincided with the
human's new commit appearing on `local/main`, and with a `sed -i` on the file). Mitigation until understood: commit
right after every edit batch, compare `git diff HEAD --stat` with what you meant to change before moving on, and treat
an unexpected diff against HEAD as a Delta sync event, not as your own edit. Candidate follow-up: a Delta-side
explanation of when it writes into a thread's working tree.

## Trial 5 scorecard
| | 5A actor (12a0) | 5B preflight (06be) | 4B lock (800d), resumed |
|---|---|---|---|
| Result | done, 5 tests, transcript | done, 9 tests, transcript | done, 3 tests, stress 25046 -> 25600 of 25600 |
| Gate in my checkout | 336 + 28 | 340 + 28 | 348 + 28 |
| Landing shape | pending merge, tree clean, by SHA | pending merge, stale shaving file + fmt leak, by SHA | pending merge on a pinned history: add/add conflicts + mass revert, cherry-pick |
| Defect found by me | bracket in actor broke the stamp round-trip | default scope was noise | none (reviewed; risks logged) |
| Skill gap the worker reported | how to record a shorn summary | whether to self-shear when the coordinator judges | where in-farm files belong |

# Trial 6: two more workers (yaks-b4dc discovery bound, yaks-2cd4 `yaks commit`)  (delta-lead)

## O47 [ok][cli] The walk-up hazard is closed, verified on the real Delta layout  (delta-lead)
6A (yaks-b4dc, e5136f4) bounded discovery as the human decided: `$YAKS_DIR` first (the `.yaks` dir, a dir containing
one, or a pointer file; a bad value is an error, not a fall-through), then the walk, which now stops at the first
directory holding a `.git` and no `.yaks`, naming that git top-level and the three fixes (`yaks init`, a pointer file,
`YAKS_DIR`); outside git the walk to the filesystem root is unchanged. Landing was the cleanest yet: base an ancestor
(step 7 passed), merge-tree conflict-free, applied tree equal to the computed merge, by-SHA merge byte-identical.
My own check used the toy private repo from trial 3, whose real Delta checkout still sits at
`/tmp/yaks-private-trial/.delta/worktrees/<id>/yaks-private-trial`: before, `yaks list` there silently read the live
farm (O11); now it exits 1 with the error, `YAKS_DIR=<farm> yaks list` works, and the outer repo and its subdirectories
still resolve normally. Consequence to expect: Delta x private and in-tree git worktrees of a private farm need
`YAKS_DIR` (or a pointer) in the brief and on every command; the private/worktrees/delta skills now say so. The
worker noted one gap: no way to ask which farm a command resolved without a yak id (yaks path <id> is indirect).

## O48 [ok][cli] `yaks commit`: landed, and the one edge that mattered for Delta  (delta-lead)
6B (yaks-2cd4, 552ac2b) built `yaks commit [-m] [--dry-run]`: stages the farm and runs `git commit --only -- .yaks`, so
code and other staged files stay out; generated message per yak (`yaks: created A; shorn B; ...`); hooks run; never
pushes; private farm fails clearly; no-op when clean. It chose the leans for all three forks and recorded them rather
than asking, reasonably (none was a user-visible risk). Landing: base an ancestor, merge-tree conflict-free, but a stale
`shaving/yaks-2cd4.md` sat in my applied tree beside the new `shorn/` file (third time this pattern: 5B, 6B, and the
O22 resumed worker), so the by-SHA merge was again the right move. The deliberate overlap with 6A on docs/cli.md, the
README and the skill table merged automatically, three files, no conflict (adjacent added rows).
My own testing found the edge the worker's did not: during a PENDING MERGE (what a Delta landing leaves) git refuses a
partial commit and the command reported a bare "git commit failed ... staged, not committed". Fixed with a
MERGE_HEAD/CHERRY_PICK_HEAD guard that stages nothing, exits 1 and says what to do (test fails without the guard).
Pattern worth keeping: a coordinator's review of a command that wraps git should exercise it in the states the
workflow actually produces (pending merge, failing hook, other staged files), not only the clean one.

## Trial 6 scorecard
| | 6A bound discovery (b4dc) | 6B yaks commit (2cd4) |
|---|---|---|
| Result | rules + 8 tests, Delta-layout transcript | command + 6 tests, transcript |
| Gate in my checkout | 359 + 28 | 366 + 28, then 367 + 28 with my guard |
| Landing | clean: ancestor, merge-tree equal, by-SHA identical | ancestor, merge-tree clean, stale shaving copy removed by by-SHA |
| Defect found by me | none; verified on the real Delta checkout | pending-merge edge (fixed + test) |
| Forks | decided in the claim; one call recorded (bad YAKS_DIR is an error) | three leans taken and recorded, no ask |
| Skill gap reported | no way to ask which farm resolved without a yak id | none |

## O49 [skill] The landing procedure is now a tested script  (delta-lead)
By-SHA landing was done by hand eight times across trials 1-6; its failure shapes are known and mechanical (stale
`shaving/` copy, unexplained reverted files, history-pinned base, untracked leftovers that must match the commit).
`.agents/skills/yaks-coordinating-delta/land.sh <sha> [--dry-run] [--force-discard]` encodes it: reports the pending
state, chooses merge or cherry-pick from whether the worker's base is still an ancestor, computes the correct tree with
`git merge-tree --write-tree`, REFUSES (exit 3, files named) if the working tree holds changes the worker's commit does
not explain, clears the applied state, lands, and checks the result equals the computed tree; it never pushes. A
`--selftest` builds temp repos for five scenarios (12 checks: merge with a Delta-like applied state, refusal that keeps
the edit, --force-discard, dry run, rewritten-history cherry-pick); `cargo test` runs it
(`delta_land_script_selftest_passes`, skipped on git older than 2.38). The first selftest run caught two bugs in the
test itself (macOS `wc` padding; a setup that did not really rewrite history) and one in the script (running doctor with
no farm). Not yet exercised on a real landing; use it on the next one and compare with the manual checklist.

# Cross-machine threads (yaks-99ec)

## O50 [open][skill][cli] A thread shared to another machine gets managed bare clones, not the user's checkout  (delta-lead, from Joel's handoff subthread)
Source: read-only inspection plus one probe by the handoff subthread; the first three facts below I re-checked on the
Linux machine, the rest are as reported and not yet re-run by me. (The relay called this O56; the log's next number is O50.)
Setup: Joel shared this thread from his laptop to a Linux machine. The thread has two Delta worktrees (yaks, edtui). On the
laptop `local` is `~/src/yaks/.git` (a real checkout) and worktrees live in `~/src/yaks/.delta/worktrees/<ns>/`. On the
Linux machine the same thread got clones under `~/.local/share/delta/worktrees/<dir>/{yaks,edtui}` and `local` was a
Delta-managed BARE repo (`~/.local/share/delta/user_*/managed-repositories/<uuid>/repository.git`).
1. Cause (reported): Delta keeps a per-machine "user checkout" per repository record. When none is known ("No user checkout
   is known for LocalRepositoryId(...)") it falls back to a managed bare clone of GitHub `origin/main`. It does not link
   `~/src/edtui` by remote URL even though the URL is identical.
2. The managed `local` is bare (`core.bare=true`): only `origin/*`, Delta's `refs/delta/<dir>/<repo>/<sha>` pins and tags.
   The `receivepack ... updateInstead` config is still written into the clone but means nothing for a bare repo.
3. NOT carried to the other machine: the laptop's local-only refs (`delta/trial23|4|5|6`, `delta/yaks-b5a0` archives) and any
   uncommitted yak answers in the laptop checkout. Only what is on `origin` crosses. Consequence for the skills: "to
   reach my main, `git push local <branch>:main`" assumes a real checkout, which is false for an imported thread.
4. Directory names are opaque: the `<dir>` names are Delta's per-mount names (SQLite `app_worktree_mounts`); the tool
   worktree UUIDs are per-thread handles and a fork reuses them, so one UUID maps to different directories in parent and
   fork. Reliable lookups: `pwd` from a terminal call with the `worktree` parameter, or `git for-each-ref refs/delta`
   (lists dirs, not owners).
5. Repointing `local` works at the git level only. Joel ran `git remote set-url local /home/joel/src/yaks/.git` in the yaks
   worktrees (parent and subthread): fetch and push --dry-run reach his real checkout (I re-checked here: `git fetch local`
   works, the checkout is non-bare, on `main`, equal to `origin/main`). But a probe commit was pinned as `refs/delta/...` in
   the MANAGED repo, not in `~/src/yaks`: Delta's own bookkeeping follows its database record, not `remote.local.url`.
   Here `git for-each-ref refs/delta` in this clone lists nothing (pins are in the managed repo, as reported). The edtui
   worktrees' `local` has NOT been repointed.
6. Delta features seen only in app strings and its DB, none exercised: a "Work In" picker (Isolated Delta Worktree /
   Existing Local Checkout / Existing Git Worktree), "Add Project to Thread", "Set as Primary Project", and the text "The
   workspace cannot be changed after the agent has used this project". We do not know whether a shared thread can be
   rebound to a local checkout on the receiving machine; we expect not once the agent has run.
7. Older threads on that machine DID get a linked checkout (`~/src/yaks/.delta/worktrees/bearqc...`, `local` =
   `~/src/yaks/.git`), because the project was added from the folder there. So whether a thread gets a user checkout depends
   on how its project was added on that machine.
Joel's goal: work in Zed and CLI tools (including `yaks`) without chasing where Delta put the worktrees; he is open to
this not being the intended workflow (then he would only open these repos from Delta).
What it means for yaks: worktree visibility gets more urgent (yaks-38dc lanes view, yaks-70e5 `yaks diff`), across git
worktrees AND Delta clones, and across machines; every such facility must exist as a CLI command.
Open questions (tracked on yaks-99ec): how yaks finds sibling worktrees when `local` is bare, absent, or at a different path
per machine; what `preflight`/`path`/`discover` say there; what `yaks-coordinating-delta` and `land.sh` assume about
`local`; and what carries farm state and archive branches across machines (only `origin`, which we never push without Joel).

## O51 [ok][skill] What a worker sees on a shared-thread (managed-layout) machine  (delta-lead, probe-1 / yaks-f49c, landed f3a95d2)
Evidence: `.yaks/artifacts/yaks-f49c/probe-1.md` (probe-1's commands and real output; I re-ran the pin and layout checks).
- Layout: checkout `<root>/<dir>/yaks`, its git dir the SIBLING `<root>/<dir>/yaks.git` (`.git` is a `gitdir:` file,
  `core.worktree` set); no `.delta/` anywhere. `git worktree list` prints the git dir itself, so it cannot enumerate
  Delta checkouts. `yaks path/preflight/doctor` work unchanged (discovery stops at the `.git` FILE).
- The worker's `local` is the managed BARE repo (not repointed: the coordinator's `set-url` is per clone and is not
  inherited). It has no `refs/heads`, so `git fetch local` prints nothing, `local/main` does not exist, and
  `git push --dry-run local HEAD:refs/heads/x` is ACCEPTED. Inferred, not run: a real `git push local <branch>:main` from
  a managed-layout clone would succeed into the managed repo, where the human never looks. The skills' "push local" step
  is unsafe there; it needs a check that `local` is a checkout.
- `objects/info/alternates` names the managed repo, which holds the pins for every thread dir on the machine
  (`refs/delta/<dir>/yaks/<sha>`; the worker's final commit was pinned under its own dir 3d4gr26p09ng, 3 pins in all).
  Pins give dir -> commits but not owner, and the dir list does not mean the checkout still exists.
- Workers are not isolated from siblings: the worker could `ls` the coordinator's checkout and read its
  `target/release/yaks` (a worker may use the coordinator's built binary; it could also modify it, so the brief forbids it).
- `DELTA_THREAD_TITLE` in a worker is the spawn title, not the worker's name (as O15); `git status` there said
  `ahead 2` = the coordinator's two unpushed commits at spawn time.

## O52 [ok][skill] First real landing with land.sh, and a conflict I caused  (delta-lead)
Landed yaks-f49c (6345ab0) with `land.sh --dry-run` then `land.sh`. The pending merge left the worker's files untracked
beside a half-finished merge ("All conflicts fixed but you are still merging"); the script reported the base as an
ancestor, found that `git merge-tree` conflicted, fell back to cherry-pick, stopped with exit 5 and told me what to do.
I resolved by hand and `git cherry-pick --continue`; `yaks doctor` clear. The conflict was my doing: after spawning I
committed a note to the worker's yak (the model, which the skill tells the coordinator to record, but which is only known
from the spawn confirmation, AFTER the claim is committed). The worker then moved that file shaving -> shorn with its own
note, so both sides changed it. Skill fix to make: do not edit a worker's yak after spawn; record model and base on the
parent/umbrella yak or in the landing note instead. Also: the script's "merge by SHA" line followed by "falling back to
cherry-pick" reads as a contradiction; say "merge conflicts, so cherry-picking" once.
No landing problem otherwise: no stale shaving copy, no reverted files.

## O53 [ok][cli] Handles for finding sibling checkouts without Delta's database  (delta-lead)
Both layouts expose the same three things, so a git-only discovery is possible:
(1) the clone's `objects/info/alternates` -> the repo that hosts the pins (the human's checkout on the laptop, the managed
bare repo here); (2) `refs/delta/<dir>/<repo>/<sha>` in that repo = which dirs exist or existed and their commits; (3) a
checkout is `<root>/<dir>/<repo>` in both layouts (`<root>` = `<repo>/.delta/worktrees` on the laptop,
`~/.local/share/delta/worktrees` here), i.e. `$(dirname $toplevel)/../<dir>/<repo>` for a sibling. What is missing: owner
(which thread), and liveness beyond "the directory exists". Plain git worktrees need only `git worktree list`.

# Trial 7: lanes CLI (yaks-e545) and transition records (yaks-7149), two workers in parallel on the Linux machine  (delta-lead)

## O54 [ok][cli] attr-1 (yaks-7149 part 2, 46f1abc): landed clean, and my review found a regression the tests could not  (delta-lead)
Landing: Delta left a PENDING MERGE with the worker's files untracked/modified, plus the human's own new yak untracked in
my tree. `land.sh --dry-run` computed a clean merge and refused, correctly, because Joel's `yaks-699b` was an unexplained
file. `git merge --abort` left the applied files in place (it does not discard them; `land.sh` does), I committed
Joel's yak by itself, then `land.sh` landed by merge, "result identical to the computed merge", doctor clear. First
land.sh landing with no conflict: base ancestor, computed tree equal. Worker: 8 end-to-end tests that fail without the
change, a two-actor run that is attributable from the file alone, docs parity done, 3 rustfmt leaks reverted.
My review defect: with option A a move is a NOTE, so every shorn yak now has one, and `doctor --strict`'s "shorn yak with
no recorded note" check (the evidence-before-shear rule in yaks-working) could never fire again. Reproduced on a scratch
farm (shave + shorn, no note: strict "All clear"). Fix `649b48a`: `store::is_transition_text` (exactly `moved: <status> ->
<status>`) and strict ignores those; the test fails with the old logic and passes with the fix (I swapped the condition
back and watched it fail). Same pattern as O25/O48: a feature that reuses an existing record shape changes what every
consumer of that shape means; grep the consumers (here `parse_notes` callers) and ask what each assumed. Other callers
checked: `last_verify_passed` matches `verify: ` and is unaffected.
Also fixed: a `\u{25b8}` escape in a doc comment, and the stale coordination-skill lines (moves are attributed; do not
edit a worker's yak while it runs). `yaks-8265` shorn as covered. Cost to watch (the worker recorded it too): a move edits
the yak body, so two lanes moving or noting the same yak now conflict more often (O52).

## O55 [ok][skill][cli] Lanes landed, reviewed on the real thread, and the first push using the "is local a checkout" check  (delta-lead)
lanes-1 (yaks-e545, 3e4659b) landed by cherry-pick: land.sh stopped on three doc-table conflicts (its `lanes` row beside
7149's reworded `log` row, same three files in every case), I kept both rows, cherry-pick --continue, gate 378 + 8 + 28.
Review by running it where siblings are known to me: it listed every sibling Delta checkout with the right HEAD and mtime,
and it was read-only (index, HEAD, ref and yak-file mtimes identical after two runs). It had one design gap the worker
named: the farm delta was against THIS checkout, so every stale lane showed phantom moves reading backwards
(`yaks-7149 shorn -> hairy` for lanes that never touched 7149). lanes-2 (yaks-d738, f5561ed) fixed it by diffing against
the merge-base (read with ls-tree + cat-file into a temp dir) and adding BEHIND; landed with land.sh in one pass (base an
ancestor, computed tree equal). On the thread: stale lanes say "no changes of its own"; a worker's real changes still show.
Known limit: a lane whose work was landed by merge or cherry-pick still shows "+1 ahead" and its changes (the commit is not
an ancestor); `git cherry` could mark such lanes "landed". Not built.
Push: before pushing main I ran the new check from the Delta skill on my own clone (`git ls-remote --heads local main`
prints main; `rev-parse --is-bare-repository` at local's URL says false), built three per-lane squash commits cut at the
tested tips (final tree identical to the tested tip, d8034b7), pushed the branch, an archive `delta/trial7` (unsquashed
history) and `local main`; Joel's checkout (clean, on main) advanced 4fc959e -> dab94a1 and stayed clean; origin untouched.
Deleted my merged pr/ branch on local; re-synced my clone with `reset --hard local/main` after checking that no worker was
running or parked and the trees were equal.
Trial 7 scorecard: probe (f49c) 1 conflict I caused; 7149: clean landing + 1 regression found in review (strict doctor);
e545: 3 doc-row conflicts + 1 design gap found in review; d738: clean. Four landings, three coordinator-side defects, all
found by running the thing, none by the worker's own tests.

# Trial 8: lane labels (yaks-c29a), `preflight --push-main` (yaks-d1bd), unknown ahead/behind (yaks-1b39)  (delta-lead)

## O56 [ok][cli] Delta's thread sync leaves clones SHALLOW; "no merge-base" is the normal case on Delta machines  (delta-lead)
Joel pasted `yaks lanes` from his laptop: `+374 -3` and `(vs this checkout)` on three lanes. I reproduced it exactly by
running my build from his primary checkout: `git rev-parse --is-shallow-repository` was true, `.git/shallow` listed the
thread's own pinned commits (dab94a1, 2c28d23), `HEAD` saw 3 commits, so git had no merge-base with a lane at 4fc959e.
My own thread clone on the laptop was shallow the same way (12 commits visible). The timestamp of `.git/shallow`
matched the arrival of the thread's commits, so the mirror/sync is the likely cause (not verified in Delta). Fixes by hand
(Joel authorized touching his checkout): `git fetch --unshallow origin` in the primary, `git fetch --unshallow local` in the
clone; both histories connect again, one boundary stays (2c28d23, a pinned unreferenced commit whose parents live only on
the other machine). `lanes` was correct to fall back; only the counts misled, and the lane directories Delta had removed in
the meantime correctly vanished. yaks-1b39 turns the counts into `?` and prints one `git fetch --unshallow` hint when the
repo is shallow.

## O57 [ok][skill][cli] Trial 8: three small workers, clean landings, one check that is deliberately not default  (delta-lead)
lbl-1 (c29a, 5d69ae0): landed clean (computed tree equal, gate 390 + 8 + 28). The worker rejected both in-progress rules I
proposed and chose a third, with the reason on the yak: a worker forks AFTER the coordinator's claim, so "the lane moved it
to shaving" would leave every real worker lane unlabeled. A brief that offers two rules and lets the worker test them
against the real layout is worth keeping. pre-1 (d1bd, 6703870): landed clean (399 + 8 + 28). It put the check behind
`--push-main` rather than the default scope (a worker cannot fix the human's checkout, and O45 showed what noisy defaults
cost); I agree and ran it against the real local (ok), a managed-style bare repo and a dirty checkout on main (both fail
with the fallback command). Gap it reported: a clean checkout on main without `receive.denyCurrentBranch=updateInstead`
would also refuse the push; not checked. unk-1 (1b39, e63806a): landed clean (402 + 8 + 28); tests use a real depth-1 clone.
First push that used the new check (`preflight --push-main` ok), two squash commits at tested tips, final tree equal to the
tested tip, `local main` 5a07d6a, archive `delta/trial8`, clone re-synced after verifying equal trees. Titles of the spawns
began with the worker names (Delta skill rule from Joel's question about delta:<title>).
Process notes: terminal output was silently dropped three times when a command chained several outputs (one landing looked
done and was not: HEAD unchanged, MERGE_HEAD still set); re-running the step alone and checking `git log`/`MERGE_HEAD`
caught it. Keep verifying state instead of trusting a missing line.

## O58 [open][skill] How Delta uses shallow files: evidence from its log and binary (Joel asked to understand it)  (delta-lead, 2026-10-05)
Sources (read-only; no Delta source code): `~/Library/Application Support/delta/delta.log`, `strings` on `Delta.app/Contents/MacOS/delta-app`,
the git dirs of the thread clones and the primary checkout. Facts, then inference.
Facts. (a) The app binary contains code to copy, write and replace a git shallow file ("failed to copy shallow file from",
"failed to write git shallow file", "failed to replace git shallow file", "shallow-file:base-backed"), uses `GIT_SHALLOW_FILE`,
`--update-shallow`, `git rev-parse --is-shallow-repository`, and `objects/info/alternates`. (b) Every Delta clone's git dir holds
`delta/copy-baseline`. (c) The log, on this thread's mount: `worktree_mount: ... steady-state error: Repository(git merge-base
without shallow boundaries failed: error: Could not read 46f1abc...)` at 20:57Z and again for d51a3cb at 21:05Z on Oct 4, i.e. the
moments commits from the other machine / worker arrived; Delta runs its own merge-base IGNORING shallow boundaries and logs when an
object is missing. (d) `thread::worktrees` logs "failed to resolve worktree <uuid>: resolution did not finish within 15s; its history
may be unreachable while offline" and "worktree preparation started ... repository=local:<uuid>"; `deltadb::worktree` logs "mount
remote refresh ... remote=origin tracking_refs=N". A thread's worktree has an id and a HISTORY held in Delta's own store (deltadb,
`[share-sync] full-sync HistoryId(...)` over a websocket) and is resolved per machine; a "mount" is its checkout. (e) After my
`git fetch --unshallow` of the primary (15:47 local) and the clone (15:55), `.git/shallow` in each still lists exactly one commit
(2c28d23, my pre-squash merge commit, whose parents exist only on the Linux machine) and the commits made on the laptop since
then (a1ee94f, 5a07d6a...) added no boundary.
Inference (not verified). Shallow boundaries are DELIBERATE: Delta imports thread commits into a mount/primary without their
ancestors and writes them as boundaries so git never needs the missing parents; cross-machine arrival (history from the share-sync)
is what produced ours (dab94a1, 2c28d23 are exactly the commits that came from the Linux machine); locally created commits did not.
So `git fetch --unshallow` is a harmless workaround as far as we saw but Delta may re-add boundaries the next time a thread comes
in from another machine. Untested: whether Delta rewrites the file on the next cross-machine import; the next machine switch will
show it. Questions only Delta's authors can answer: which operation writes a boundary, and whether unshallow is supported.
Consequences for yaks: (1) `yaks lanes` showing `?` and one hint is right. (2) land.sh decides "base is an ancestor of HEAD" with
`git merge-base --is-ancestor` and computes the merge with `git merge-tree`; in a shallow repo both can be wrong in either
direction (a missing parent looks like "not an ancestor"). It worked every time here because the base was always a recent commit
of ours, but the script should say when the repo is shallow (filed as yaks-a398).

# Trial 9: project-local skills install (yaks-3859) and `yaks init --mode` (yaks-814a)  (delta-lead)

## O59 [ok][cli][skill] Two sequential workers, both landed clean; what each review added  (delta-lead)
ins-1 (3859, 4671760) and ini-1 (814a, 1a1e10d) were sequential on purpose (init reuses the install function), each landed by
`land.sh` in one pass (base an ancestor, computed tree equal), gates 414 + 8 + 28 + 3, then + 19 for init's new tests (which
the worker ran against a pre-change checkout: all 19 fail there). My own checks: install into a temp git repo from a
SUBDIRECTORY with a temp HOME (9 files at the git top-level, land.sh byte-identical and 0755, differences only the stamps and
the source README, the user dir untouched); the six coordination skill descriptions changed text and still load through the real
`skill` tool; each init mode driven in temp repos (plain init + hint; private: `git status` empty, exclude lines, second run
"Nothing to change", a different mode errors naming what would have to change; pointer from a subdirectory with the right
YAKS_DIR line). No defect found in either; I did not run `skills install` in the source tree (a failing guard would overwrite the
source), I relied on the worker's transcript and the test for the refusal.
Choice logging (Joel's question): ins-1 recorded its autosync decision and the consequence on the yak (a note at 21:05:21Z)
because the brief asked for it by name; the same held for lbl-1's in-progress rule and pre-1's scope decision. Unprompted choices
are the risk. Plan: the brief template and yaks-working get a line to write a `Decision:` note (the choice, the alternatives
rejected) for anything a reviewer could argue about, with the next skills pass and its cold-read.
Coordinator-side: the ini-1 report named stale prose in yaks-coordinating-private (fixed, 3715922). Pushed as two squash commits
on Joel's f441b3b (`local main` 4bdf6db, archive `delta/trial9`), preflight --push-main ok first. Open: yaks-36b6 (auto-update of
project-local skills; lean: notify) is Joel's call.

# Trial 10: stdin/file input (yaks-c53a), the stale-skills notice (yaks-5c83), a skills pass with a cold-read gate  (delta-lead)

## O60 [ok][skill][cli] Two workers landed clean; the cold-read gate found three real gaps and one wrong suggestion  (delta-lead)
inp-1 (c53a, 5e5a1e9) and ntf-1 (5c83, b750092) landed by `land.sh` (base an ancestor, computed tree equal), gates 417 + 8 + 11
+ 28 + 19 + 3 and 424 + ... + 3 + 3. Both worker briefs required `Decision:` notes with the rejected alternatives; both
delivered (seven and eight), and each named its consequence (ntf-1: notice only for strictly newer, an unwritable marker means no
notice, `--json` detected anywhere in argv; inp-1: stdin on a TTY is refused). My own checks: a markdown heredoc with backticks,
quotes, `$` and a fence through create, update, ask and answer came back byte-for-byte; a current install is silent, writes no
marker and leaves the tree clean (the false-positive check). Notes the workers made BEFORE the new flag worked were one-liners
(inp-1 said so itself): the guidance only helps once the tool does.
Delta's applied state again reverted a section I had COMMITTED in the working tree while ntf-1 was in flight (my markdown
guidance vanished from `.agents/skills/yaks/SKILL.md` on disk); a by-SHA landing restored it. Same family as O31/O46: never
trust the tree during a landing, and commit before spawning so the by-SHA merge has something to keep.
Cold-read gate (first real use of the rule): cold-read-1, a Haiku scout reading only the skill files, loaded all skills and
answered lanes, private farm, landing, the push check, the spawn title and the yakherd correctly. It missed the styled-markdown
constructs (deep in the long `yaks` skill, which workers do not read; the facts now also sit in the short `yaks-working`), the
`Decision:`-vs-`ask` line (core §8 has it, `yaks-working` did not), and what "the model" to record means. cold-read-2 re-ran
exactly those and got all three right. One of its suggested fixes was wrong (escaping backticks inside a quoted heredoc would put
backslashes in the note): a weak reader is good at finding gaps and not to be trusted on the cure. The reusable brief is attached
to yaks-5c9f and the rule is in `.agents/skills/README.md`.
My own defect: the land.sh shallow note (yaks-a398) fired on any shallow repo; in this clone one leftover boundary (a pinned
commit) made it fire on every landing. Fixed to fire only when a boundary commit is reachable from HEAD or the worker commit
(selftest scenario 7). Pattern, again: a warning that is true but irrelevant trains people to ignore it; test it on the real
machine before shipping.
Also this round: yaks-75cc and yaks-0a87 shorn (already covered by shipped skill text and tools), thoughts on the farm vs git commits
on yaks-5f12, analysis of c061 vs f313 on f313 (background jobs do not survive a Delta terminal call, tested), yaks-7acf (the yakherd)
shorn, yaks-2c22 asked (where the brief template lives).

## O61 [open][skill] Workers died silently when the thread changed machines  (delta-lead, 2026-10-06)
Two workers on the same yak (yaks-2c22, `yaks brief`) ended without a final message and without a commit: brf-1 (last file write 04:02Z,
silent for about 15 hours; its lane showed an untracked `src/brief.rs`, an edit to `src/main.rs` and one note) and brf-2 (spawned 19:51:44Z,
copied brf-1's draft, claim note at 19:52:06Z, completion event "finished without a final assistant message" about a minute later). I could
not read either from the threads tool ("history ... was reset while its state was awaited"), and a status request to brf-1 got no answer.
Evidence for the cause (inferred, not confirmed in Delta): the thread moved from the laptop to the Linux machine at about 19:53Z. My last
laptop shell read 19:52:57Z and my first Linux shell read 15:53:57 EDT, the same minute, and brf-2's death falls inside it. brf-1's silence is
consistent with the laptop sleeping or the app closing. Joel's own guess was a Delta sync issue making the thread look stuck; this fits.
Practical rules: (1) do not leave workers running across a machine switch; ask the human to say before switching, or wait for the completions;
(2) when a worker is overdue, `yaks lanes` is how to see it (a `who: <worker>` label, dirty files, an old FARM ACTIVE time) and the lane's files
are readable; (3) a dead worker's lane can be seeded into a replacement, which copies and reviews, never edits in place; (4) after two
consecutive deaths on the same task the coordinator finishes it (here: from brf-1's draft, with tests and docs added by me) instead of a third
try of the same shape.
Side effects seen on the Linux clone after the switch: Delta kept re-reporting the dead worker's applied state (`src/brief.rs` "created", the
brf-2 claim note, the `Brief` variant in `src/main.rs`) as external edits several times. `git status` stayed empty and my committed edits were
never touched, so it was noise; the check is `git status --short` plus a line count of the file after each replay. The thread clone on each
machine is a separate git repo; my commits had synced to this one (HEAD was my latest), and `git fetch origin` showed GitHub's `main` at the
commit I had pushed from the laptop.

## O62 [skill] What `local` is, per machine, and what it means for keeping two machines in sync  (delta-lead, 2026-10-06)
Joel asked, after a week of the thread moving between his Mac and a Linux box: what is `local` now, and can a thread shared to another
machine update THAT machine's checkout? Everything below was observed in this thread.
- `local` is per machine and per thread clone. On the Mac it is `/Users/joel/src/yaks/.git`, his real checkout (the project was added
  there as a folder). A thread shared to the Linux box first got a Delta-managed BARE repo as `local` (O50, O51), because that machine had
  no linked checkout for the repository record; Joel then repointed `local` by hand to `/home/joel/src/yaks/.git` in the clone, which made
  pushes to `main` land in his Linux checkout (updateInstead). While the thread ran on Linux I pushed there; when it moved back to the Mac,
  `local` pointed at the Mac checkout again (each machine's clone keeps its own config). That is why `~/src/yaks` on the Mac did not have
  yaks-2c22 for a while: the push had gone to the Linux checkout.
- Delta never moves a branch in the human's checkout. It pins every thread commit as `refs/delta/<dir>/<repo>/<sha>` in whatever repo
  `local` stands for (O36); `main` moves only when a coordinator pushes it.
- The two checkouts are separate repos with no shared state: after my pushes the Mac checkout, the Linux checkout and GitHub held
  three different `main` commits (09b2130, 991c0d2, deea536) until I made them equal.
- So the answer to "can Delta update the other machine's checkout?" is: not by itself. The one carrier that both machines can reach is
  GitHub (`origin`). A coordinator on machine B can bring B's checkout level: `git fetch origin`, `yaks preflight --push-main`, then
  `git push local origin/main:main` (a fast-forward; updateInstead updates the working tree). Nothing can push to the OTHER machine's
  checkout from here (a different filesystem), unless the human gives each machine an ssh remote to the other, which we have not tried.
- Today's sync, in order: squash commits built on the checkout's own `main`, pushed to the Mac `local`, then to GitHub (a fast-forward on
  the human's say-so), then this thread's clone reset to GitHub's `main` after checking the trees were equal and archiving the unsquashed
  history as `delta/trial14`. The Linux checkout still needs `git fetch origin && git reset --hard origin/main` (it holds only my earlier
  equivalent commit).
