---
type: topic
title: 'Spec-driven development: YASS and the DeepLearning.AI course'
tags:
- research
- design
- agent
stale_after: P3M
sources:
- sources/2026-11-05-04642f1f/source.md
- sources/2026-11-07-611eab26/source.md
- sources/2026-11-05-ab6fc4f7/source.md
- sources/2026-11-05-9bfc1964/source.md
---

# Spec-driven development: YASS and the DeepLearning.AI course

Read on 2026-11-07 for yaks-3610. Joel's friend Jaime Yap built YASS and recommended the course for background, and Joel thought there was inspiration for yaks here. This page says what each one is, how it differs from a farm, and which ideas are worth borrowing. It is a first read, not a decision; the open questions at the end are for Joel.

## In one paragraph

The course argues for writing a spec before the agent writes code. YASS is one concrete, small implementation of that for a repo: a folder of plain markdown next to the code, where each change carries its own goal, acceptance criteria, decisions and log. Both are solving the problem yaks solves with a different shape: **yaks tracks the work as a queue of small items in a farm apart from the code; YASS keeps the plan inside the code repo and reviews it like code.** Most of what yaks could borrow is not the layout but three rules about when intent may change.

## The course

*Full Course: Spec-Driven Development with Coding Agents*, DeepLearning.AI with JetBrains, taught by Paul Everitt, 61 minutes ([metadata](/sources/2026-11-05-ab6fc4f7/source.md)).

**I did not watch it and have no transcript** (YouTube refused the caption download). Everything below comes from [Gemini's summary](/sources/2026-11-05-9bfc1964/source.md) that Joel pasted, which is a model's reading, with Gemini's own timestamps. Check a claim against the video before relying on it.

Per that summary:

- **A project constitution:** mission, tech stack and a roadmap, kept as an agent-agnostic contract.
- **A feature loop,** one branch per feature: plan (a spec with tasks and validation), implement, validate.
- **Replanning between features:** update the constitution and roadmap, and improve the workflow itself.
- **Skills** package the repeated parts (changelogs, validation); **MCP and ACP** keep the agent and the IDE swappable.
- **Legacy code:** have the agent reverse-engineer the constitution and roadmap from what exists.
- **Humans validate the architecture,** not low-level details, and keep changes small to avoid "AI fatigue".

## YASS

By Jaime Yap, snapshot at `1516c8d`, 11 October 2026, version 0.5.0 ([repo](/sources/2026-11-05-04642f1f/source.md), with [the six playbooks](/sources/2026-11-07-611eab26/source.md)). A Go binary, plain markdown, an optional git hook. The author's own account of itself; the comparison table in `docs/comparison.md` is his view of other tools.

### The shape

A change is a folder under `yass/changes/`, dated. Ceremony scales with scope: a bug fix is one `change.md`; large work adds `prd.md`, `plan.md`, an optional `design.md`, and PR-sized pieces as folders inside it. Finished changes move to `yass/archive/<year>/<month>/`.

A `change.md` has Goal, Acceptance (criteria with `verify:`), Steps, Decisions (`- <decision> - <why> (<who>, <date>)`) and a Log whose last line is `Next:`. Boxes are `[ ]`, `[/]`, `[x]`, `[-]`.

### The six rules that matter

1. YASS describes change, not the current system. The code and its docs are the current state, so there is no second description to drift.
2. A change is a folder; ceremony scales with scope.
3. **Progress travels with the code:** ticking boxes and appending Log and Decisions land in the same commits as the code.
4. **Intent changes get their own commit:** the PRD, design, plan text, and a change's Goal and Acceptance, including dropping a criterion. If the plan is wrong mid-build, the fix goes on a branch beneath the code and the code is rebased on top. An optional hook warns when a commit mixes code with an intent edit.
5. Done means every box is checked; then `yass archive`, in its own commit.
6. The archive is append-only; follow-up work is a new change with `follows:`.

### The playbooks

Six agent skills, about 5,000 words: `yass-shape` (a conversation that ends in an approved PRD), `yass-plan`, `yass-work`, `yass-status`, `yass-log` ("why did we pick X?", answered from the decisions and git) and `yass-update`. `yass-work` is explicit that it works only on the change it is pointed at: **choosing what to build is the human's call**, not the playbook's.

### Things it does that are worth noticing

- **`yass decisions`** lists every decision across active and archived changes, newest first, filterable by words, change, date or a cited commit. The farm holds about 655 `Decision:` lines across 161 yaks today, found only by `yaks find`.
- **Boxes cite the commit they rest on** when plans live in another repo, and CI can require a `yass:` trailer on commits. A done box whose code isn't merged is a warning.
- **A "next" line closes every Log entry,** so any agent can resume from the files alone.
- **Ceremony scales inside one unit,** rather than a separate roadmap or epic layer.
- **`yass-work` unattended:** "if nobody is watching, don't ask questions; do one change and stop when you need a human."
- **Self-upgrade:** version stamps on every installed file, `yass upgrade`, and a playbook that updates YASS. The farm's own skills-staleness problem (yaks-ec00) is the same one.

## How each differs from a farm

| | YASS | Yaks |
|---|---|---|
| Where it lives | in the code repo, reviewed with the code | its own branch, never in a code commit |
| Unit | a dated folder; one file for small work | one file per yak, flat ids, `parent:` for hierarchy |
| State | boxes in a file; "not started" is derived | `hairy`, `shaving`, `shorn`, `dead` as a field |
| Who is working | a branch view, no claim | `shave` is the claim; `yaks sheds` reads it |
| Coordination | one human directs; no inbox | `needs:` and the inbox; coordinator and workers |
| Evidence | `verify:` per criterion, ticked by the agent | `verify:` per yak and `yaks verify`, recorded as a note |
| Decisions | a section with who and when, listed by a command | a `Decision:` note by convention, found by text search |
| History | archive folders plus git | git history of the farm, `yaks undo` |
| Scope | one repo (monorepos by folder) | several herds and repos in one farm |

The deepest difference is rule 3: YASS puts progress in the code's commits so a branch carries its own plan. Yaks deliberately does the opposite, so code branches carry no task churn and every checkout sees one live list. Neither is wrong, and the farm's invariant (code commits never contain `.yaks/`) rules the YASS layout out.

## What yaks could take

Ordered by how little they cost.

1. **A tool for decisions.** Decisions are the most reused kind of note and only text search finds them. A `Decision:` note could carry `--meta kind=decision` (notes already take metadata), and a `yaks decisions` could list them with who and when. No new file, no schema change.
2. **Intent versus progress as a stated rule.** YASS's rule 4 names a real failure: an acceptance criterion quietly reworded next to the code that "meets" it. Yaks has `verify:` and evidence-before-shear but no rule about editing the yak's description after work starts. A skills line, not a feature.
3. **A closing `Next:` in every note** an agent leaves when it stops. The yaks skill already asks for a note saying where you stopped; the convention would make it findable.
4. **A constitution.** The course's project-level document (mission, stack, roadmap) is what `AGENTS.md` plus the farm's design doks are for. The reverse-engineering step for legacy code is `yaks-a096`'s territory (importing a repo's docs and history).
5. **A playbook that updates the tool.** Already filed as yaks-ec00.

What yaks should not take: the folder-per-change layout, and the archive. Both are answers to living in the code repo.

## What does not carry over

- **The course's loop assumes one human with one agent per feature.** Yaks has many agents, a coordinator and an inbox, which neither source addresses.
- **YASS's "no current-state spec" rule** is a bet that code and docs stay in step. Yaks' own docs rule ("a user-facing change is not done until its docs are") is the same bet, enforced by habit; neither source tests it.
- **Both are early.** YASS calls itself v0.1 in its README (0.5.0 by its changelog) and says its playbooks "haven't been battle-tested in each" harness.

## Limits

- The course: summary only, no transcript, not watched.
- YASS: read the README, the comparison, the changelog, one archived change, and the `yass-work`, `yass-shape`, `yass-plan` and `yass-log` playbooks. I did not run the binary, read its Go code, or read `yass-status`, `yass-update`, the monorepo or separate-plans docs. The snapshot also left out dot-directories until I added the playbooks separately (yaks-3da7).

## Open questions

- [ ] Is a structured decision note (`kind=decision` plus a listing command) worth building, given the farm already has hundreds of free-text ones?
- [ ] Should the yaks skill state a rule about editing a yak's description after it is shaving, as YASS does for intent?
- [ ] Does Jaime's "intent never changes in the same commit as code" have a farm equivalent, such as the description frozen once `shaving` and changes recorded as notes?
