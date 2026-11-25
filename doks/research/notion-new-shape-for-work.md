---
type: topic
title: 'Notion''s "A New Shape for Work": context systems and factories, read against yaks'
created: '2026-11-25T04:32:18Z'
tags:
- research
- agent
description: What Notion's guide says about shared context systems and agent factories, how much of it a farm already does, which ideas were filed as yaks and which are out of scope; read before designing evals, run history or recurring agent work.
stale_after: P1Y
sources:
- sources/2026-11-25-100f7779-a-new-shape-for-work-notion-labs-guide-v10.md
---

# Notion's "A New Shape for Work": context systems and factories

Read on 2026-11-25 for yaks-822d. Joel asked for an overview of Notion's guide [A New Shape for Work](/sources/2026-11-25-100f7779-a-new-shape-for-work-notion-labs-guide-v10.md) (v1.0, published 2026-11-20, about 10,000 words), read skeptically because Notion sells what it recommends, and for what applies to yaks. The live page was taken down within days; the source is the Internet Archive copy of 2026-11-23.

## In one paragraph

The guide says AI used through one-off personal chats gives little leverage because nothing accumulates. Its fix has two layers: a shared **context system** that every person and agent reads before working and writes back to, then **factories**, shared agents for one kind of work that a few people own and improve. Most of the first layer describes what a farm already is, and yaks is in places more careful about it (rot, provenance, history). The useful lessons are about the second layer: keeping prompts under evals, and making runs visible to whoever owns them. The parts that point at a hosted workspace (sync everything in, row and column permissions, triggers) are out of scope for yaks.

## What the guide says

**Context system.** "A computer without a hard drive" is its picture of chat-only AI. The context system has six jobs: store everything; give each thing a logical place; govern it with permissions and version control; display it in any view; collaborate well for humans; work with any agent. Supporting ideas:

- **Ontology:** a small set of core typed collections with relations (Notion's own: Projects, Tasks, Documents, People, Teams, Customers). Agents follow relations instead of searching: go to the "Q4 2026" row of a Quarterly Plans table and its links, rather than search "Q4 roadmap".
- **Fractal structure:** small, temporary projects get a little structure too.
- **Syncs:** copy everything from other systems into the one store, rather than give agents many connectors.
- **Meetings** as first-class objects; **permissions** by group, down to rows and columns.
- **Validation:** bad information can't get in. **Context rot** is "Enemy No. 1", and agents can now do the upkeep.

**Factories.** A factory is **specific** (one kind of task), **shared** (many people use it) and **scrutinized** (someone watches its runs and improves it). A skill is only a "proto-factory": the text is shared, but it runs in each person's harness, model and laptop beside other skills, so failures can't be seen ("works on my machine"). The simplest factory is a To do / Doing / Done board plus one agent woken by a new card. Building one properly: notice recurring, verifiable, stable work; start simple and fix observed failures, not imagined ones; run in shadow mode, then raise the volume; keep versioned prompts backed by evals. Slogan: "Rent the intelligence. Own the context."

## Reading it skeptically

- **The argument is a funnel.** A context system must do six hard things; don't roll your own; sync everything into "one world-class provider". Each step is reasonable alone, and together they arrive at Notion.
- **"Own the context" is the reverse of the product.** In a hosted workspace the context is rented too. The guide's own "parlay" argument against tying yourself to one model vendor applies equally to tying all context to one workspace vendor. Plain files in git are context you own.
- **It argues against our design by name.** Agent-built "low-level databases or markdown files" create "as many problems as [they] solve". Its four objections (security, agent search, governance, collaboration) are fair questions for yaks, answered below, not a verdict.
- **No evidence.** Every figure is illustrative. "The rest quickly follow, without need for persuasion or change management" is asserted. The growth numbers quoted around it measure Notion's usage, not outcomes.
- **Factories suit uniform work.** Its own criteria (recurring, stable, average quality acceptable) admit that much engineering and creative work doesn't fit. Putting work behind an owner also makes a bottleneck and moves the opacity to the user's side, which it doesn't discuss.

## How the guide maps onto a farm

| Guide | Farm |
|---|---|
| Context system, read first and written back | The farm: yaks, doks, sources, schemas, session notes ([knowledge-store](/doks/design/knowledge-store.md)) |
| Version control, audit, safe agent edits | Every write commits with an actor; `yaks undo --by`; field-wise merge driver |
| Ontology and relations | Typed doks and schemas, `parent:`, `depends_on`, the link grammar, `refs --incoming` |
| Fractal structure | A family per piece of work; typed and table doks |
| Validation | Schemas advise and `yaks check` reports afterwards; never a refusal |
| Curing context rot | `stale_after`/`verified`, `yaks check`, the `yaks-upkeep` loop |
| Works with any agent | `--json`, the bundled skills, no model inside yaks |
| The simplest factory | hairy, shaving, shorn plus `needs: agent` and `--wake agent` |
| Scheduled and event triggers | `every:`, `due:`, `yaks tick`; the clock stays in the harness ([scheduling](/doks/design/scheduling.md)) |

The guide's answers to its four objections come back as: governance is git; agent search is `yaks find`, `peek` and the link grammar; collaboration is the TUI plus sync; security is repository access. The real gaps against a hosted product are mobile, notifications and fine-grained permissions, none of which yaks should grow.

## Ideas worth taking

| Idea | From the guide | Yak |
|---|---|---|
| An eval suite for the bundled skills, rerun on every skills change | Step 4, "Improve": versioned prompts backed by evals | yaks-a6c5 |
| Run notes, so recurring and skill-driven jobs can be reviewed from the farm | "Scrutinized"; skills as proto-factories | yaks-ff8e |

Two smaller points, not filed:

- **Shadow runs first.** A new recurring job could run in `--dry-run` or ask mode until it has a few clean rounds. A sentence in `yaks-scheduling` if run notes go ahead.
- **Start simple, fix observed failures.** Already how this farm works; the guide's phrasing ("why guess when you can know for sure?") is a good line for the skills.

## Out of scope or a poor fit

- **A grouping node for projects or initiatives**, the guide's Quarterly Plans example. Farms, herds, families and tags already give four ways to group; a fifth would mostly add choice (Joel, 2026-11-25).
- **Sync everything in.** `yaks-tracker` is a one-way projection and sources are dated snapshots, on purpose.
- **Refusing bad input.** Schemas advise, never refuse ([yaks-8e75](/yaks/yaks/yaks-8e75.md)): refusing agent writes breeds workarounds.
- **Event triggers, hosted factory agents.** Need a clock and a process; a harness's job, as in [Haks](/doks/research/haks-orchestration-layer.md) and [Vercel Board](/doks/research/vercel-board.md#out-of-scope-or-a-poor-fit).
- **Row and column permissions.** Repository access is the boundary; a separate farm is the answer.

## Open questions

- [ ] Is the guide's line between skill and factory real for a farm, or does a recurring yak plus run notes plus evals give a skill everything a factory has except a shared machine?
