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
