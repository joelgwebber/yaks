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
