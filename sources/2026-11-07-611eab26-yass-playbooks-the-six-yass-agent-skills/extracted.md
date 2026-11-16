## yass-log/SKILL.md

---
name: yass-log
description: Reconstruct the decision log from YASS change folders and git history. Who decided what, when and why, across active and archived changes, for a time range, a feature, or a person; and which code carried a decision out, or which decision is behind a piece of code. Use for "why did we…", "what did we decide about…", "why does this code look like this", release notes, retros, or reviewing decisions made without a human.
---

# yass-log

There's no separate log file. The record is the `## Decisions` sections in the yass folders' `changes/` and `archive/`, plus git history. `yass root` prints each yass folder: `yass/` in the repo (team folders too), or wherever a `yass.yaml` points. Boxes may cite the code they rest on (`— code: a1b2c3d`); those citations link decisions to code.

1. **Scope the question:** a time range, a feature or area, a person or agent, a single "why", or a piece of code (a file, a function, a commit).
2. **Find the decisions:**
   - Ask `yass decisions`: it lists the `## Decisions` entries of active and archived changes in every yass folder, newest first, with each change's created and archived dates. Narrow it with `--since`/`--until YYYY-MM-DD`, `--about "<words>"`, and `--change <name>` (which takes in the changes it follows and that follow it); it shows 50 at a time (`--limit N`) and says how many it left out. `--json` gives the same as data. It reads evicted months back from git when a query reaches them, and says when a shallow clone is missing one (`git fetch --unshallow`). Don't read `archive/` directly: it can hold thousands of changes, and evicted ones are only in git.
   - Each entry reads `- <decision> - <why> (<who>, <YYYY-MM-DD>)`; older ones have no date of their own, so `yass decisions` shows the change's dates instead.
   - **Plans kept apart from the code?** Each code commit names its change in a `yass:` trailer (`git log -1 --format='%(trailers:key=yass)' <sha>`), and `yass status <change>`, run in the code repo, lists the change's merged commits and, by their `ac:`, each criterion's. That's the most direct link, and it survives squash merges.
   - **Starting from code instead?** Find its commits in the code repo (`git log --format=%h -- <file>`, or `git log -S'<symbol>' --format=%h`), then ask which boxes cite each: `yass decisions --cites <sha>` (any prefix of the hash; it searches active, archived and evicted changes). A hit names the change, file and box; its Decisions and Log are the why. No hit means the record doesn't link that commit, so say so rather than guess from dates.
3. **Date them and check who made them** with git, because the files say *what* and git says *when* and *who committed it*:
   - `git log --format='%h %ad %an %s' --date=short -- <change folder>`; for an archived change, which was moved, follow its `change.md` across the move: `git log --follow --format='%h %ad %an %s' --date=short -- <archive folder>/change.md`
   - `git log -S'<distinctive words>' --format='%h %ad %an' -- <file>` finds the commit that added a decision line.
   - For intent edits (PRD, plan, design), `git log -p -- <file>` shows how the promise changed over time.
   - **Squash-merged history:** a subject ending in `(#N)` is pull request N, squashed. Name it next to the commit (`#16`), since its description and review are where the decision was argued, and read them when the decision line leaves the why unclear (with a code host CLI, e.g. `gh pr view N`).
4. **Follow the citations to the code.** For a change's boxes with `code:`, show each cited commit from the code repo: `git log -1 --format='%h %ad %an %s' --date=short <sha>`, and whether it's merged (`git merge-base --is-ancestor <sha> <branch>`; `branch:` in `yass.yaml`, else `origin/HEAD`, `main` or `master`).
5. **Read the change's context** (Goal, PRD, Log) for the *why* behind anything the decision line leaves unclear.
6. **Report** newest first, grouped by change, with the code that carried each decision out when it's cited:
   ```
   2026-09-15  offline-sync  Sync is last-write-wins per entry - entries are append-only (sam)
               code: 4f2c1aa 2026-09-18 sam "feat: per-entry LWW merge" (merged)
   ```
   Flag decisions with no recorded author, and goalposts that moved:
   - **Plans in this repo:** intent edits that landed in the same commit as code.
   - **Plans in another folder:** that can't happen, so look for the cross-repo version instead: an acceptance criterion reworded or dropped (`[-]`) *after* a box citing its code was marked `[x]`. Compare when the criterion changed (`git log -p` in the plans) with when the box was marked done.
7. Cite file paths and commit shas so every line can be checked. Don't infer decisions that aren't written down; say when the record is silent.

## Plans in another folder
When `yass root` prints a folder outside this repo, the plans and the code have separate histories:
- **Run git for the plans in their folder,** limited to it, since it may share a repo with other projects' plans: `git -C <folder> log … -- .`. Commit shas in your report should say which repo they're from.
- **Run git for the code here,** in this repo, for the cited commits.
- **No git at all?** A plain folder has no commit dates. The only dates are the Log headings (`### YYYY-MM-DD`), so place a decision between the Log entries around it, and say the date is approximate.


## yass-plan/SKILL.md

---
name: yass-plan
description: Plan a YASS change. Write plan.md (approach, acceptance criteria per milestone, validation), split a large change into PR-sized pieces, record hard-to-undo calls in design.md, or revise the plan after feedback. Can start from a plan drafted in the harness's plan mode. Use after shaping a PRD, before building a large change, when saving a harness plan as a YASS plan, or when the approach must change.
---

# yass-plan

`yass` is the YASS CLI, on your PATH. The plan says how, and how we'll know it's done. Plan text is intent: keep code out of the edits you make here. Don't commit unless you've been asked to.

Planning mostly runs on its own. The human made the big calls while shaping; turn the approved PRD into a plan without a back-and-forth, and ask only when the PRD leaves a real choice open. It ends at a written plan: it never starts building. The change stays `not started` until someone points `yass-work` at it. Record calls in `## Decisions` (or `design.md`), not the Log; the first Log entry marks the start of work.

## Write the plan
1. Read the change's `prd.md`, the code it touches, and its docs. If this session already has a plan (from the harness's plan mode, or a plan file the human points you to), start from it: its approach, file-by-file steps and risks become the plan's Approach, pieces and design calls. Don't redo research it already did; check that it still matches the code and the PRD.
   - **No change folder yet?** If the harness plan is for something large, run `yass-shape` first (it can start from the same plan). If it's small, `yass new "<title>"` and turn the plan into its Goal, Acceptance and Steps; it doesn't need a `plan.md`.
2. **Approach:** the shape of the solution, in a few paragraphs. Point at existing modules and docs rather than restating them.
3. **Acceptance,** per milestone:
   `- [ ] AC1 (R1) Given …, when …, then … — verify: <test id, e2e flow, or manual: steps + expected>`
   - Observable behavior only, one behavior each. Cover the edges that matter: empty, error, offline, slow, denied permission, killed mid-way.
   - Every requirement in the milestone has at least one criterion.
   - Prefer automated checks; use `manual:` only when automation is unreasonable.
   - Harness plans are usually lists of steps, not criteria. Write the criteria; don't relabel the steps.
4. **Hard-to-undo calls** (data model, dependencies, architecture, security, public API): write `design.md` with at least two real options and who decided. `yass new … --design` creates it for a new change; for an existing one, `yass template design > <change folder>/design.md` and replace `{{title}}`.
5. **Validation:** how the whole change is verified before it's called done.
6. **Hand it back.** Say the plan is ready and that building starts when someone points `yass-work` at it. If the plan came from an approved harness plan, say what you added or changed beyond it.

## Split into pieces
For anything bigger than one pull request:
- `yass new "<piece title>" --in <change> [--goal "…"]` for each piece. Pieces nest one level deep.
- One reviewable story per piece; the main branch stays green after each. Thin vertical slices beat layers; shared foundations go first.
- A branch working on a piece changes only that piece's folder (and code), so parallel branches on different pieces never touch the same file; the hook warns when a commit with code reaches past it.
- In each piece's `## Acceptance`, name the plan criteria it delivers (`Delivers AC1, AC3–AC5`), plus anything specific to the piece. `yass status` counts a criterion done once every piece delivering it is, and `yass archive` ticks it, so don't plan a closing commit for the parent.
- Under `## Pieces` in `plan.md`, say what order they go in and why. When a piece truly can't start before another is done, also set its `blocked:` to that piece's folder; `yass status` then shows what it's waiting on.
- Small changes don't need pieces, or a plan: their `change.md` Steps and Acceptance are enough.

## Plan mode
Harness plan modes (Claude Code's, and others') usually can't write files. Research and draft there; once the human approves the plan and you can write again, write it into the change as above.

## Revise the plan
When reality disagrees: edit `plan.md` (and pieces) without touching code, record what changed and why in `change.md` `## Decisions`, and tell a human. Rewording, removing or dropping (`[-]`) an acceptance criterion lowers the bar, so call it out explicitly. If code is already in progress, the revision goes underneath it; see "Intent changes mid-build" in `yass-work`.


## yass-shape/SKILL.md

---
name: yass-shape
description: Shape a large YASS change, in conversation with a human. Turn a problem or opportunity, or a plan already drafted in the harness's plan mode, into a prd.md (why, for whom, outcomes, non-goals, requirements, milestones) inside a new change folder, or revise the PRD of an active change after feedback. Use when someone wants to build a new feature, component or system, wants to save a plan as a YASS change, or when what a large change promises needs to change.
---

# yass-shape

`yass` is the YASS CLI, on your PATH. A PRD says what should change or exist, and why. It doesn't describe the current system; read the code and its docs for that, and link to them instead of restating them. The PRD is intent: keep code out of the edits you make here. Don't commit unless you've been asked to.

Shaping is a conversation. It's where a human decides what to build and why, so explore, offer options, and ask; don't rush to a finished document. It ends at an approved PRD: it never plans or builds. Record calls in `## Decisions`, not the Log; the first Log entry marks the start of work.

## A new large change
1. **Start from what exists.** If this session already has a plan (from the harness's plan mode, or a plan file the human points you to), it's your first draft: carry over its problem, scope, decisions and open questions rather than asking again. Ask only about what the PRD needs and the plan doesn't say. Otherwise, understand the problem first: who it's for, what evidence says it matters, and what success looks like.
   - Either way, read the relevant code and docs, check the archive for earlier work on the same thing (`yass status --archived`, and `yass decisions --about "<words>"` for what was decided), and `yass status` for anything in flight that overlaps.
2. **Frame options before requirements.** For a non-trivial problem, sketch 2–3 ways to scope it (appetite, rough solution, risks) and let a human pick. Skip this when the scope is already clear, or the human already picked one in their plan.
3. **Create it:** `yass new "<title>" --large [--source <link>] [--follows <archived change>]`. Fill in `prd.md`:
   - **Why:** the problem, with evidence.
   - **Users and outcomes:** who, and measurable targets.
   - **Requirements:** `- **R1** [M1] A user can …`. Observable, one behavior each. IDs are never reused.
   - **Non-goals:** everything you decided not to do. Agents treat these as walls.
   - **Milestones:** slices that each leave users better off.
   - **Open questions:** anything still undecided.
4. Write the change's one-paragraph Goal in `change.md`, and the big calls in its `## Decisions` with who made them. A choice the human made in their plan is theirs: credit them.
5. **Show a human the PRD** and get their go-ahead before planning, unless your team has told you otherwise. If it came from an approved plan, say what you added or changed beyond that plan; the rest is already agreed. In a fully autonomous run, where you've been put in charge of approvals, approve it yourself and record that in `## Decisions`.
6. Offer `yass-plan` next. If the harness plan also covered the approach, `yass-plan` can use it too.

## Plan mode
Harness plan modes (Claude Code's, and others') usually can't write files. Research and draft there; once the human approves the plan and you can write again, create the change as above. The harness plan lives with the session, the change folder lives with the repo.

## Revising an active change's PRD
Feedback, a failed assumption, or a cut in scope. Edit `prd.md` without touching code, record what changed and why in `change.md` `## Decisions`, and tell a human. Then update the plan to match (`yass-plan`).

A finished change is archived and never edited. New intent for archived work is a new change with `--follows`.


## yass-status/SKILL.md

---
name: yass-status
description: Report where YASS work stands. Summarize progress across active changes, what's blocked and why, decisions made recently, stale changes, and changes ready to archive, then list what needs a human decision. Read-only. Use when someone asks where things stand, what's next, or what needs them.
---

# yass-status

`yass` is the YASS CLI, on your PATH. This playbook only reads; it changes no files. Be brief: the human wants decisions, not narration.

1. **Gather** (quietly):
   - `yass status`, then `yass status <change>` for every change that isn't `not started`: its open boxes show which items are `[/]` (in progress), and it's where blocked and nearly done changes show their detail.
   - What happened recently (ask since when, or use a week): `git log --since=<date> -- <folder>` for each folder `yass root` prints (for one outside the repo, `git -C <folder> log --since=<date>`, if it's in a git repo) and the Log entries dated since then.
   - Decisions recorded in that time (the `yass-log` playbook does this well).
   - With plans kept apart from the code (`yass root -v` says `separate`), pull them first, and note what `yass status` says about them being behind or uncommitted. Run in a code repo, `yass status <change>` lists the change's merged code from commits' `yass:` trailers.
2. **One screen:**
   1. Progress per change (boxes ticked, pieces done), one line each, in the order `yass status` lists them: a yass folder's `queue.md` ranks its changes, top first. Keep changes in progress apart from ones that are planned but `not started`, and name the items marked `[/]`: that's what's actively being worked on.
   2. Blocked changes and the reason, and changes `waiting on:` others (say whether those are moving).
   3. Decisions made recently, with who made them, so the human can object.
   4. Changes `yass status` marks `done`: ready to archive.
   5. Stale changes: no activity in a while.
   6. Intent edits waiting underneath work in progress (a plan or PRD revision on its own branch), so they get reviewed first.
3. **What needs a human,** one line each, with the move that handles it:
   - **Blocked:** the question to answer. If the answer changes intent, `yass-shape` or `yass-plan` makes the edit.
   - **Disagreeing with a decision:** the reversal is recorded as a new decision, not by rewriting the old one.
   - **Ready to archive:** `yass archive <change>`.
   - **Versions out of step** (`yass status` ends with a note about the `yass` binary and the repo's YASS files): `yass-update` sorts out both.
   - **Stale or abandoned:** resume it with `yass-work`, or drop what's left (`[-]`, with a reason in Decisions) and archive it.
4. **Offer one next move.** Asked "what's next?", it's the first change in `yass status` order that isn't `done` or blocked; without a `queue.md`, say that nothing ranks the changes and offer to start one. Plan a shaped change, point `yass-work` at a specific change or piece, or shape something new. Don't make it until the human says so.


## yass-update/SKILL.md

---
name: yass-update
description: Update YASS on this machine and in this repo. Check the yass binary's version against the repo's and the latest release, get a verified binary with `yass update` (asking first), then `yass upgrade` the repo and hand back the diff. Use for "update YASS", "upgrade yass", "is yass up to date?", or when `yass status` notes that the binary and the repo's YASS files are out of step.
---

# yass-update

`yass` is the YASS CLI, on your PATH. YASS updates on two levels: the `yass` binary, once per machine (`yass update`), and YASS's files in each repo, the playbooks, the `AGENTS.md` section and the hook (`yass upgrade`, then a commit everyone gets). Keep them apart: the first changes this machine, the second changes the repo for the whole team.

1. **Check both.** `yass update --check` prints the binary's version and how it was built, the repo's YASS version, the latest release, and what `yass update` would install. Tell the human in a line or two. If nothing is behind, say so and stop.
2. **The binary.** It downloads a program and replaces the one on this machine, so ask first, unless you've been given latitude for it. Then run `yass update`.
   - By default it installs the repo's version when the repo is ahead of the binary, else the latest release. `--latest` goes past the repo; do that only if the human also wants to upgrade the repo, which is a team decision.
   - It verifies the download (checksum, and signed provenance when the GitHub CLI is there). If it says provenance wasn't checked, pass that on. If a check fails, stop and report it: never work around it, and never fetch a binary some other way.
   - A binary built from source isn't replaced: `yass update` prints the command for how it was built (`go install …`, or a rebuild in a clone). Show it, and run it only if the human says so.
   - A folder it can't write to: relay what it says (permissions, or installing yass into a folder of your own).
3. **The repo.** If the binary is now ahead of the repo's YASS files and the human wants the repo upgraded: run `yass upgrade` and show what changed (`git status`, a short diff summary).
   - It may also migrate the yass folder's layout and print a commit for that; it goes in its own commit, before the rest.
   - Commit only with the latitude to (`chore: upgrade YASS` for the rest); otherwise give the human the commands. Pushing and opening a pull request need latitude too.
   - Say that teammates need a `yass` at least this new once it lands: `yass update` gets it for them.
4. **Report:** the versions before and after, how the binary was verified, and any commits or commands left for the human.

Never downgrade, and never edit YASS's files by hand to make versions match: `yass update` and `yass upgrade` are the only ways.


## yass-work/SKILL.md

---
name: yass-work
description: Work on a YASS change you've been pointed at. Create it if it's new and small, resume from its Log, build against its acceptance criteria, keep progress (boxes marked in progress or done, Log, Decisions) up to date alongside the code, put needed plan changes on their own branch underneath the code instead of giving up, verify, and archive it when every box is checked. Use for "work on X", "keep going" on the change at hand, "fix this bug", and unattended runs that name a change.
---

# yass-work

`yass` is the YASS CLI, on your PATH. If nobody is watching, don't ask questions: do one change (or one piece), and stop when you need a human.

**Work only on the change you've been pointed at.** Picking what to build next is a decision, and it belongs to whoever is directing the work, not to this playbook. Being in progress isn't a reason to pick a change: several can be, and a freshly planned one (`not started` in `yass status`) may still be waiting for someone to approve its plan.

**Git is up to whoever is driving.** Don't commit, branch, rebase, push or open a pull request unless the human or your harness has given you that latitude; pushing and pull requests are public. If you have it, do it yourself (plain `git`, or a stacking tool like `gh stack` if the repo uses one). If you don't, and someone's watching, give them the exact commands to run. Either way, keep the work shaped so that intent and code can land separately. "Separately" means apart from code: if the team squash-merges, that's a pull request with no code in it, which can still hold other intent, progress or an archive move.

**Say where the change stands.** A commit or pull request for a change can show its overall progress, so a reviewer, or `git log` later, sees the whole arc and not just this step: `yass status <change> --brief` prints a line like `offline-sync: 2/3 pieces done, 14/20 boxes · offline-queue: 6/6 boxes, done`. It fits a commit message's body and the top of a pull request's description; a title can carry a short form, like `(offline-sync 2/3)`. Use what helps your team: it's a suggestion, not a step.

1. **Orient.** Find the change you were pointed at: named in the request (names can be shortened: `yass status queue`), or the one this session is already working on. Nothing else counts.
   - **Not pointed at one?** Show `yass status` and ask which. Unattended, stop and say so.
   - **Directing your own work?** If the harness or the human has put you in charge of choosing (a fully autonomous run), the choice is yours: pick deliberately, and before you start a `not started` change, check that its plan has been approved or approve it yourself as you've been allowed to. Record the choice, and any approval, in its `## Decisions` with `(<you>)` as who.
   - **No change yet?** For a small, clear request (a bug, a tweak), create one: `yass new "<title>" [--source <issue link>]` and write its Goal and Acceptance. If there's an approved plan from the harness's plan mode, turn it into the Goal, Acceptance and Steps. Anything that changes what users can do in a big way, or needs a design call, goes to `yass-shape` first.
2. **Load.** `yass status <change>` lists its files and open boxes. Read `change.md` and resume from the last **Next:** in its Log. For a piece, also read the parent's `prd.md` and `plan.md`. Read the code and docs you'll touch.
3. **Build.** Alongside the code:
   - mark a box `[/]` when you start on it, and `[x]` once it's finished (Steps, Acceptance, and a large change's plan criteria when it has no pieces). If you stop partway, leave it `[/]` so the next session sees where you were.
   - working on a piece, change only that piece's folder: tick its own boxes, including its `Delivers` box, and leave the parent's files alone. The parent needs no closing commit: `yass status` counts the criteria its pieces deliver, and `yass archive` ticks them.
   - append real choices to `## Decisions`: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`
   - keep `## Steps` current; it's your checklist
   - if `yass root` is outside this repo, cite the code commits; see **Plans in another folder** below
4. **Don't move the goalposts.** `prd.md`, `design.md`, plan text, a Goal and acceptance criteria (including dropping one) never change alongside code. If they're wrong, see **Intent changes mid-build** below.
5. **Out of scope?** Note it in the Log, or file it wherever this repo tracks issues. Don't build it.
   - **Waiting on another change?** Set `blocked: <its folder>` (several, comma-separated), say so in the Log, and stop. It clears itself once that change is done. A `yass status` line showing `waiting on:` means the same for the change you were pointed at: say so rather than building around it.
6. **Verify** each criterion the way it says (`verify:`). Re-run checks rather than trusting earlier output. If the repo has a separate reviewer or evaluator step, use it. Only mark `[x]` what has evidence; anything without it stays `[/]`.
7. **Before you stop,** append to `## Log`, so anyone can resume from the files alone:
   ```
   ### <YYYY-MM-DD> (<who>)
   - Did: …
   - Next: …
   ```
8. **Finish.** Merging a branch doesn't finish a change; a large change spans many branches. When every box is done or dropped (`yass status` says `done`), run `yass archive <change>` (it prints how to commit the move). Commit it apart from code; with squash merges, in a pull request with no code, which can carry the change's last progress too. Pieces are archived with their change.
   - A Step you no longer need can be dropped (`[-]`) any time, with a reason in Decisions.

## Intent changes mid-build
Building often shows the plan is wrong. Don't quietly fix it next to the code, and don't give up either: stack it.

1. **Reason it out.** What exactly should change in the PRD, design, plan, Goal or Acceptance, and why? Write that down.
2. **Not confident?** Set `blocked: <reason>` in the frontmatter, say what and why in the Log, and stop for a human.
3. **Confident?** (Plans in another folder? See below instead.) Put the intent edit on its own branch, cut from where your work branched off, with the reason in `change.md` `## Decisions`. Then rebase your work on top of it and keep building against the new plan. Reviewers see the intent change in its own pull request, below the code that depends on it, and can reject it without untangling anything.
   ```bash
   git switch -c <work-branch>-intent <base>     # with your in-progress work committed or stashed
   # edit prd.md / design.md / plan.md / Goal / Acceptance, and add the Decision
   git commit -m "yass: revise <change>" -- <the intent files>
   git switch <work-branch>
   git rebase <work-branch>-intent
   ```
   With `gh stack` or a similar tool, use its commands for the same shape.
4. **Lowering the bar** (dropping or weakening an acceptance criterion) is still an intent change, stacked the same way, but say so plainly in the Decision and the pull request: a human has to agree before it merges.
5. **Who runs it:** if you're allowed to drive git, run it yourself. If someone's watching but you aren't allowed, make the edits, then show them the commands. If nobody's watching and you can't commit, leave the edits in place, put the commands in the Log's **Next:**, and stop.

## Plans in another folder
When `yass root -v` says `separate` (a `yass.yaml` points outside this repo), plans and code are committed separately, often in different repos. Nothing can carry progress in the same commit as the code, so the routine changes:

- **Start by updating the plans.** Pull them (`git -C <plans folder> pull --ff-only`) before reading the change; `yass status` notes when they have uncommitted files or are behind. A plans repo with no upstream (local only) has nothing to pull: carry on with it as it is. Then resume from the Log as usual.
- **If the plans folder isn't there, stop and say so.** Never run `yass init` to fix it: it would create an empty plans folder that only this checkout sees. The fix is the human's: clone the plans where `yass.yaml` expects them, or set the variable its `path:` uses. From a linked worktree, YASS already looks in the clone's other checkouts.
- **Name the change in every code commit.** Run `yass use <change>` when you start (`--ac ac2,ac5` adds the criteria you're working toward); the hooks then add a `yass: <change>` trailer to each commit. It's per worktree, so parallel worktrees can work on different changes. Without the hooks, write the trailer yourself. A commit that belongs to no change gets `yass: none`.
- **Commit the code first, then the plans.** The code commit carries the trailer; the plans commit marks boxes and cites the code commits they rest on: `- [/] Disable Save while saving — code: a1b2c3d`. Name them in the Log entry too.
- **`[x]` waits for the merge.** A box is done when its code is on the main branch: cite the commit there (after a squash merge, the squash commit). Until then it's `[/]`, citing the branch commits. `yass status` warns about a cited commit it can't find, and about a done box whose code isn't merged (`branch:` in `yass.yaml` says which branch counts).
- **Boxes with no code** (a decision, a manual check) need no citation.
- **Cross-link them.** The code pull request names the change and its progress (`yass status <change> --brief`) and carries the `yass:` trailer line in its description, so a squash merge keeps it; the plans commit or pull request names the code's.
- **Latitude is per repo.** Being allowed to commit, push or open pull requests in the code repo says nothing about the plans repo, and the other way round; without it in one, show the human that repo's commands.
- **Intent changes go where the plans live,** as their own commit or pull request in that folder's repo, linked from the code's pull request. There's nothing to stack under the code; keep building against the revised plan once you're confident, as above.
- **The plans stay on one branch** there, usually main.

