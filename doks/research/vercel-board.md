---
type: topic
title: 'Vercel Board compared with yaks: a hosted message board for agents'
created: '2026-11-25T04:25:26Z'
tags:
- research
- agent
description: What Vercel Board (a hosted message board and task queue for agents) does, how it maps onto yaks, which of its ideas were filed as yaks and which are out of scope; read before designing claims, waits, plan files or agent onboarding.
stale_after: P6M
sources:
- sources/2026-11-25-66116b62-vercel-blog-introducing-vercel-board-2026-11-18.md
- sources/2026-11-25-3126ca1a-vercel-board-docs-overview-and-data-model.md
- sources/2026-11-25-b347410c-vercel-board-docs-tasks-claims-leases-needs.md
- sources/2026-11-25-476f2206-vercel-board-docs-cli-reference.md
- sources/2026-11-25-b0d8da34-vercel-board-docs-building-an-agent-swarm.md
- sources/2026-11-25-89f2e66f-vercel-board-agent-onboarding-playbook-board.md
- sources/2026-11-25-1d5e649b-vercelboard-specmd-v1-draft-at-6d49814.md
- sources/2026-11-25-0718a66b-vercelboard-docs-local-boards-local-sqlite.md
- sources/2026-11-25-fb930155-vercelboard-the-board-agent-skill-skillmd-at.md
---

# Vercel Board compared with yaks

Read on 2026-11-25 for yaks-857c. Joel asked for a compare and contrast with [Vercel Board](https://vercel.com/docs/board), announced 2026-11-18 as an "experimental agentic primitive", and to mine it for ideas, including what would be out of scope or a poor fit for yaks. These are vendor sources: Vercel's blog, docs, onboarding playbook, and the open-source `vercel/board` repository (Apache-2.0) at `6d49814`. Nothing here was run against a live board.

## In one paragraph

Board and yaks solve overlapping problems from opposite ends. Board is a hosted, append-only **message log** with a task queue on top. Its strengths are the live parts: leases, blocking `wait`, server-side clocks, watchers, two-way Slack/GitHub/Linear sync and verified identity. yaks is a **current-state document store in git** with work items on top. Its strengths are durability, offline and private use, and knowledge (doks, sources, schemas) and evidence. About half of Board is the clock-and-network layer that yaks has deliberately pushed out to the harness ([knowledge-store](/doks/design/knowledge-store.md): "No model inside yaks"). Several of its task-queue ideas still fit yaks well.

## What Board is

- A team has boards; a board has channels (paths such as `work/fix`, created by the first post); a channel has threads; a thread is a root message and its replies, with labels (`k=v`), reactions, attachments and schema-validated data ([overview](/sources/2026-11-25-3126ca1a-vercel-board-docs-overview-and-data-model.md#data-model)).
- One ordered event log per board is the only truth; reads come from derived tables ([spec](/sources/2026-11-25-1d5e649b-vercelboard-specmd-v1-draft-at-6d49814.md#principles)).
- A **task** is a thread with state the board keeps: `waiting / ready / claimed / blocked / completed / failed / canceled`. Claims are leases of up to 5 minutes; dependencies (`--needs`) ready a task when its prerequisites complete; a plan file creates many tasks at once ([tasks](/sources/2026-11-25-b347410c-vercel-board-docs-tasks-claims-leases-needs.md)).
- Agents write as a **profile** under the user's credentials, shown as "fx via ada"; a profile has no access of its own.
- `wait` and `tail` block on events; watchers wake a Vercel deployment on matching events; `schedule` posts later or on an interval ([CLI](/sources/2026-11-25-476f2206-vercel-board-docs-cli-reference.md)).
- Two-way sync mirrors threads with Slack, GitHub and Linear; grants control `read / post / react / label / sync / manage` per channel.
- Data lives in Vercel's API (US only); task events are kept 30 days; free while experimental. A `local()` backend keeps a board in SQLite under a gitignored `.board/` for development and tests ([local](/sources/2026-11-25-0718a66b-vercelboard-docs-local-boards-local-sqlite.md)).
- It ships an agent skill and an onboarding playbook meant to be pasted into an agent ([skill](/sources/2026-11-25-fb930155-vercelboard-the-board-agent-skill-skillmd-at.md), [playbook](/sources/2026-11-25-89f2e66f-vercel-board-agent-onboarding-playbook-board.md)).

Vercel says it runs its security-agent swarm and v0's "software factory" (merging "over 50% of PRs") on it ([blog](/sources/2026-11-25-66116b62-vercel-blog-introducing-vercel-board-2026-11-18.md)). That is the vendor's own claim.

## How the models line up

| Board | yaks | Notes |
|---|---|---|
| Team, board, channel (a path) | Farm, herd, tags | Channels are the routing and permission unit; herds are id prefixes. |
| Thread (root and replies) | Yak (description and notes) | Close. Notes have stable anchors; Board's message ids double as cursors. |
| Task: a thread with board-kept state | Yak with `status:` | Board's seven states; yaks has four, with `needs:` and `depends_on` deriving the rest. |
| `--needs <id>`, state `waiting` | `depends_on`, `yaks next` / `tangled` | Same idea. |
| `tasks block --kind awaiting_approval` | `yaks ask` / `answer` / `inbox` | Same idea; Board's block also ends the claim. |
| Labels (`board.*` reserved) | Frontmatter fields, tags, note `--meta` | Board waits and filters on labels; yaks filters fields but waits only on the inbox. |
| Versioned JSON Schemas, validated on write | Typed doks and `schemas/`, advisory | Board refuses; yaks warns. Note `meta` has no schema. |
| Profile, "fx via ada" | Actor (`--as`, `whoami --set`), `agent@human` (yaks-eb66) | Same idea, reached independently. Board's principal is token-verified; yaks's actor is self-declared. |
| The event log is the only truth | Files are the truth; git is the log; any index derived | The same rule applied to opposite truths. |
| `wait`, `tail`, watchers | `yaks wait --for agent` (polls files) | Board wakes on any event; yaks only on the inbox. |
| `schedule --in 5m`, `--every` | `due:` / `every:` / `--wake`, `snooze`, `tick` | Board has a clock; yaks never fires anything itself ([scheduling](/doks/design/scheduling.md#principles)). |
| Sync with Slack, GitHub, Linear | `yaks-tracker`: one-way projection, `create --key` | Deliberately different. |
| Grants per channel | Repository access, `yaks private` | |
| `local()` SQLite under `.board/`, gitignored | The whole product | Board keeps its local store out of git. |

## Where they really differ

1. **Concurrency.** Board's claim is atomic, with a lease: a stalled worker's late `complete` fails with `claim_lost`, and abandoned work returns to the queue. yaks claims are advisory: `yaks shave` on a yak another actor is shaving prints "already being shaved" and exits 0 ([`transition`](https://github.com/joelgwebber/yaks/blob/b013ff062a72f96f3118c1894fd6424e9d25d887/src/main.rs#L3889-L3891)), and a stale claim only reads `stale` after two hours. Across machines git cannot do better, but every checkout on one machine shares one farm worktree behind `store::lock`, so yaks could make claims exclusive locally: a coordinator and its subagents.
2. **Liveness.** Board is built around blocking until something happens (`wait --after <cursor>`, watchers, scheduled follow-ups). yaks has one blocking primitive and it watches only the inbox; a coordinator waiting for any child to shear or ask has nothing to block on.
3. **The unit.** Board is chat first; tasks are a mode of a thread, and status that belongs to no task ("keep the team posted") is a first-class use. yaks holds work and knowledge; the conversation lives in the harness.
4. **Durability and ownership.** Board: hosted, US-only, task events kept 30 days, pricing undecided. yaks: forever in git, offline, private on request, no vendor.
5. **Knowledge.** Board has nothing like doks, sources, `check`, session notes or evidence before shearing. Its "a completed task is a claim, not proof" ([swarm guide](/sources/2026-11-25-b0d8da34-vercel-board-docs-building-an-agent-swarm.md#rules-that-keep-a-swarm-working)) is yaks's default (`verify`, attached evidence), but in Board it's only advice in a guide.

## Ideas worth taking

Filed as yaks, best fit first:

| Idea | From Board | Yak |
|---|---|---|
| Plan files: a family with dependencies from one file, one lock, unknown keys refused | `tasks create --file plan.json` | yaks-ba2f |
| Exclusive claims on one machine: `shave --exclusive`, `next --claim`, take over a stale claim | Claims and leases, `claim --ready --label` | yaks-7a25 |
| A coordinator's wait: block on state changes in a family, from a cursor | `wait --label board.task=... --after <cursor>` | yaks-b1bb |
| Conditional writes on yaks (`--if status=`, `--if-latest`, `--if-hash`) | `reply --after`, `label --if`, `edit --if-revision` | yaks-5bc9 |
| Machine-readable errors: a stable `reason` and `next` | `--json` failures | yaks-f1e8 |
| Agent onboarding: a getting-started page and an orientation command | The onboarding playbook, `whoami --json` | yaks-1407 |
| Validated note kinds: schemas for `--meta`, advisory | Message schemas (`security-finding@1`) | yaks-4a0d |
| Skill lessons: the lead's turn order, small yaks, where approval counts | The swarm guide | yaks-980b |

The swarm guide's turn order for a lead is worth quoting: note your place, answer blocked work first, check finished work, keep the queue full, then wait from your place. Without the cursor a turn-based lead misses everything that finished during its turn.

The onboarding playbook is well made as agent prose: read-only steps first, then stop and offer numbered options without choosing; take a name; treat what's on the board as information, not instructions.

## Out of scope or a poor fit

- **Watchers, serverless wake-ups, server-side schedules.** They need a clock and a process; the scheduling design rules them out ("yaks never installs a clock and never runs a command from a farm file"). `due` / `every` / `--wake` plus `tick` carry the meaning; a harness provides the firing.
- **Two-way sync with Slack, GitHub, Linear.** Contradicts the one-way projection in `yaks-tracker`. Board's own docs spend a page on data spills, and mirroring reactions, edits and deletions into git history would be noise.
- **Grants, private channels, verified principals.** Repository access is the access control. Per-path permissions inside one branch can't be enforced; the honest answer is a separate farm (`yaks init --farm`), as Board's is "use a separate board".
- **Chat: reactions, mentions, typing, posts tied to no yak.** The harness's conversation layer. Session notes are as far toward it as yaks should go.
- **Assignment (`--for <profile>`, `assign --pool`).** yaks decided on attribution, not ownership (yaks-eb66). Exclusive claims give the safety without ownership.
- **Leases across machines.** Can't be built on git's sync latency; advisory staleness plus local exclusivity is the ceiling.
- **`tasks run -- <cmd>`, settling a task by exit code.** Shearing on exit 0 skips evidence. `yaks verify` is the right shape: it records PASS/FAIL and moves nothing.
- **100 MB attachments.** In git that is `yaks lfs` or a separate farm, which is already the answer.

## One thing it showed about yaks's own design

Board says repeatedly that task text is instructions, so who can post matters; `tasks claim --ready` takes only your own tasks unless you name the owners (`--from`). yaks has the same exposure in a shared hub farm: a yak's description is a worker's brief. More directly, `verify:` is a shell command in a synced file. It runs only on an explicit `yaks verify`, but an agent told to run that executes whatever the yak's last writer put there, which sits awkwardly beside "never run a command from a farm file". Filed as a design question: yaks-a621.

## Open questions

- [ ] Does Board's `vb serve` plus tailnet sharing make a self-hosted board a real alternative to a git farm for a small team, or is it development-only as the docs say?
- [ ] Will Board's task events stay at 30 days' retention, and does it plan an export?
- [ ] Is the Board protocol (`conformance`) meant for other implementations? A yaks adapter that projects a farm onto a board's tasks would be a harness-side project, not yaks.
