---
type: topic
title: 'Haks: an orchestration layer over yaks'
tags:
- research
- design
- agent
sources:
- sources/2026-11-03-81364278/source.md
---

# Haks: an orchestration layer over yaks

Read on 2026-11-03 by Joel and `delta:haks-review`, from `subtextdev/haks` at `8fbe3ec` (Go plus a React board, about 15k lines, 38 commits, all in one day), the repo's `yaks` farm branch at `e3a9145`, a design sketch passed on by Clint Ayres, and [a summary of the Indent thread that built it](/sources/2026-11-03-81364278/source.md). See [Provenance](#provenance) for how those fit together. The repo is private and early, so this page stays at design level: no code is copied, and neither the repo nor the sketch is saved as a source. Tracked in yaks-96d8.

## What it is

Haks gives one person (and, later, a team) a single view across many herds. It does not store anything of its own.

- **Reads:** `yaks --json`, always against an explicit farm (`YAKS_DIR`). It never opens `.yaks/` files.
- **Writes:** only through yaks commands, stamped with its own actor (`haks`, or `haks:<login>` on a team farm).
- **Derives:** one snapshot (streams, herds, yaks, sheds, inbox, blocks, activity, review stage and order, stacks). The CLI (`haks board`, `haks reviews`, `haks next`) and the web board both render that snapshot. The board holds no business logic by design.
- **Local-only:** spend and control. Run registry, cost, `haks stop`, the pause flag.

Three kinds of farm: personal (your streams and to-dos), team (shared PR work, one yak per PR) and watch (a read-only mirror of someone else's farm).

## What the layering confirms

This is the first outside evidence for the line drawn in [the knowledge-store design](/doks/design/knowledge-store.md): yaks owns what can be decided from the files alone; an agent layer owns what needs a clock, a network, a model or knowledge of who is asking.

- Everything Haks adds sits on the agent side of that line: GitHub intake, the review runner, sessions, budgets, the board, ordering.
- It got by with yaks as a CLI and a data format, with no library access and no fork. The daily job that follows yaks `main` and records the tested commit in a lock file is a workable way to treat the `--json` shapes as an API.
- The coordination vocabulary was enough for a real workflow: `ask`/`answer`/`pickup`/`inbox` carry the human-in-the-loop review, notes with `kind=` metadata carry state, `sheds` carries "who is on what".
- It uses the `yaks.farm` pointer as intended: one hub farm, each code clone pointed at it, each repo a herd.

## Patterns worth keeping

- **Everything is a yak.** The test it states is "could another machine continue this from the farm alone?". Questions, evidence (`review.md`, `fixes.patch`, `context.md`), run start and end, session ids and skips all live on the yak. Local state is limited to what is genuinely per-machine, and labelled so.
- **A state machine written as notes.** A review's stage (queued, reviewing, waiting, posting, done) is derived from the yak's notes and fields, never stored. Any machine computes the same stage.
- **Claims that expire.** A run is live while its start note has no end and something has been written within 5 minutes; silence makes it stale and a new start takes it over. No coordination service, only sync latency.
- **Idempotent, append-only verbs.** Every action can run twice with the same result, and two machines' notes merge without conflict.
- **One verb entry point.** The board and the CLI call the same verb layer, so a new action needs no new route.
- **Context before work.** `haks context` attaches earlier yaks, same-path and same-author reviews, the stream's decisions and linked commits as `context.md` before an agent starts. That is the yaks-64c4 (archaeology) idea, built for one use.

## Where Haks works around yaks

The first four are the "Asks for Joel" in the repo's design doc, which the thread shows the agent wrote and marked unconfirmed, not Clint. The rest are visible in the code.

| Workaround | What yaks lacks | Fits yaks? |
|---|---|---|
| `haks reconcile` merges duplicate PR yaks (oldest wins, rest slaughtered as duplicates) | `yaks create --key <source-url>` that returns the existing yak | Yes. Decidable from files alone; makes intake idempotent and deletes reconcile. |
| Per-reviewer state kept as tags (`reviewer:`, `skipped:`, `reviewed:`) | A query over note metadata (`yaks find --meta kind=ask`) | Yes, for the query. The per-actor tag pattern is a separate question. |
| Cross-farm references are "not built yet" | A way for a personal yak to point at a team yak and back | Yes in principle, but hard: it touches the link grammar and `yaks.farm`. |
| `run_seen` frontmatter field set with `--set` to keep a run alive without a note | A touch or heartbeat with an expiry | Probably a primitive (touch, no note, no activity line) in yaks, with the window left to Haks. Overlaps yaks-0071 and yaks-039b. |
| Review options, lean, verdict and PR author parsed out of note and description text | Structured note fields | Partly. Better solved by Haks writing the structure itself (`--meta`) than by yaks parsing prose. |
| First creation of a team farm's branch is not race-safe | Idempotent `yaks init` against a remote branch | Probably. Clint's workaround is "one person starts it and pushes". |

## Provenance

Haks was built in one day, 2026-11-03, almost entirely by agents under Clint's product direction. The Indent thread (37 turns, 12:03 to 19:20 UTC, Clint sent every message) is the only place the sequence of decisions survives. The summary is a model's reading of the transcript, saved as [a source](/sources/2026-11-03-81364278/source.md); it did not read tool calls or subagent internals.

### How the design grew

1. **12:03, restore.** An earlier Indent session had built a first Haks, and its sandbox was gone, so the first hour re-created it: the run registry, `haks stop`, opt-in reviews, a $10 daily budget. Spend control came first and was agent-proposed (`review.auto: false`, because Clint's running server would otherwise start reviews by itself).
2. **12:14 to 12:44, ending work.** Clint was marking yaks shorn just to stop an agent that was asking a question. The agent's first answer was "Don't shear it" and pointed at `yaks answer --done`, which exists. Clint's own ask was simpler ("I just need a way to click on an agent and say - slaughter"), and that became Slaughter and Dismiss on every card.
3. **13:34 to 15:00, review features, mostly by confusion.** Side panels replaced modals ("the modals are killing me"). Clint set the default: "if there is a finding to fix we'll fix it", which became `review.approach: fix`, a Changes panel and `fixes.patch`. At 14:15 he reported "I asked to make changes together and now I have no idea what's going on": the board only recognised options starting with "post" or "dismiss", and the reviewer misread the answer and ended with nothing posted.
4. **15:13, the principle.** Clint: "if everything is a yak I can run the review anywhere and have it stay in sync. that's the design philosophy to follow." The agent wrote it into `AGENTS.md` and listed where Haks already broke it (run state in a local directory, details not on the yak, edits left in `/tmp`). That list drove the run start and end notes.
5. **16:15, the plan.** Clint asked for a "lego brick" design: team farm plus personal farm, every assigned PR a yak, earlier yaks consulted, reviews tied to an agent session, UI as a thin slice of the CLI, no coordination beyond yaks. The agent wrote `docs/design.md`, including the build order, the 5 and 2 minute run windows, and the asks for upstream, then built steps 1 to 5 with three parallel worktree agents after "implement the design".
6. **19:02, the sketch.** Clint asked for a lifecycle summary. The structure of the sketch Joel was given (the data and protocol, the orchestration layer, inbox, review lifecycle, "proven so far") matches that request. That is an inference, not something the thread states, but it means the sketch is the agent's summary of the work, not an independent design statement.

### Who decided what

- **Clint:** the product shape and the rules: Slaughter and Skip, side panels, fix-by-default, stack ordering, the to-do herd, "everything is a yak", the lego-brick and team-farm vision, no AI attribution.
- **The agent:** most of the mechanism: Dismiss, the budget cap, the state machine and heartbeat model, `reconcile`, context assembly, run notes, build order, and every default for an open question (where the team farm lives, shared evidence, the 5 minute stale window). Those open questions were asked once and not clearly answered, so they are still defaults.

## What the thread adds

- **The farm does not record the build.** The `yaks` branch holds 8 yaks in the `haks` herd, all created at 10:03 or 11:29 UTC, before the thread; the only later notes are Clint's two answers at 13:10 and 13:12. The principle was stated at 15:13 and then none of steps 1 to 5, the three worktree agents or `docs/design.md` appear as yaks there. The design history lives in the thread and in git commits. Caveat: this is Clint's pushed branch; a local farm may hold more, and nothing says the thread's agents were told to file yaks for Haks itself. It is still the clearest case for yak archaeology (yaks-64c4): the reasoning sat in a chat transcript, and the farm that was meant to carry it had none.
- **The reviewer runs PR code with the owner's credentials.** The thread records the allowlist being widened at 15:05 to `go`, `direnv`, `make` and `npm`, which the agent itself called "a real change in what a review can do". Running a PR's tests, makefile or install scripts is running its author's code on the owner's machine with `gh` and `git` access. This sharpens the earlier risk: the prompt rule not to post is one weakness, but the sandbox the reviewer runs in is a bigger one, and it needs a design, not a rule, before anyone else's PRs flow in.
- **Free-text options are parsed by pattern.** The 14:15 confusion came from Haks guessing which ask options were first-pass buttons. The reviews package still derives options and the lean from the ask's prose. That is a concrete case for structured ask options in yaks, for example repeated `--option` flags stored as note metadata, so a UI renders what the agent offered instead of parsing it.
- **A slaughtered review came back.** The 60 second GitHub sync recreated a yak Clint had slaughtered, so Haks now keeps dead review yaks suppressed. Same root as create-by-key: intake needs "find by source URL, including dead ones", which yaks cannot answer.
- **`yaks answer --done` was found only when asked.** The feature Clint needed already existed, and the agent surfaced it only after he complained. Haks's Dismiss button is a thin wrapper over it. Worth checking whether the skills make that flag findable at the moment an agent or human is stuck on an unwanted question.
- **Joel's own sessions were fed back in.** At 13:10 Clint's agent read a separate yaks/surveyor session and reported seven learnings: shorn parents keeping open children, a question on a shorn yak being lost, unreadable raw output, evidence recorded at shear time, agents merging on their own lean, binary version drift, and side tasks belonging in a sub-thread. It offered to add three of them to the `haks` skill and the thread shows no follow-up. Several are already yaks behaviour (shear and slaughter refuse open children), which suggests the same lessons are being relearned per project.
- **What was never exercised.** Per the thread's own closing list: a fix-mode review on a real PR, a session on a real review, Skip on a live yak, and any real team farm. It also names Haks's gaps: racy first creation of a team branch, stack detection missing same-base siblings, non-Claude sessions resuming only on the originating machine.

## Observations and risks

- **The "nothing posts before you answer" gate is a prompt rule.** The reviewer run is a headless agent with `gh`, `git` and `yaks` shell access and file writes, told by a skill not to post or push before the owner's answer. Nothing mechanical stops it, and the PR it reads is untrusted input. Fine for a first try on your own machine. For team use it deserves a real gate (a token without write scope until the answer lands, or posting done by Haks rather than by the agent).
- **Authority is split between skills.** The `haks` skill says "where they differ, this one wins" over the `yaks` skill (sync after every write, shear only after merge, no self-created streams). That is reasonable, but it is a fork of policy that drifts as upstream skills change. The daily job copies upstream's skills into the repo, which helps; the override list should stay short and explicit.
- **Team path is simulated.** The two-machine test is a good harness, but real use has not happened. Same for fix-mode reviews and sessions: the thread's own closing list agrees with the sketch.
- **The gate is weaker than a prompt rule.** See "What the thread adds": the reviewer's allowlist now includes `go`, `make`, `npm` and `direnv`, so it runs the PR author's code with the owner's credentials.
- **Tags carry meaning.** `reviewer:<login>` and friends make the farm greppable, but they overload tags, which the yaks skill wants few and broad. Worth deciding whether per-actor state needs a first-class form.
- **Dependence on `main`.** Following upstream `main` daily is cheap for Haks but makes every `--json` change a breaking change. If Haks is the first of several consumers, the JSON shapes want a stated stability promise.

## General versus Clint's own setup

The repo already separates most of this, but not all.

- **General:** snapshot, verbs, review lifecycle, runs, sessions, context, intake, multi-farm routing, the board, the `haks` and `haks-review` skills.
- **Clint's own:** the herd list in `haks.yaml` (his repos), `prompts/review.md` (his Go and Fullstory review rules), the `fs-next` and `surveyor` farm examples, the daily-pull workflow pinned to this one upstream, and the `haks-hub` persona, whose voice and boundaries (for example "never post to Slack") are personal.
- A clean split would make the registry and the review prompt user data (`~/.config/haks/`, a template in the repo) and keep the hub persona as an overridable skill.

## Naming: "dok"

Joel proposed calling wiki pages **doks** (yaks, haks, doks). Notes so far, not a decision:

- **For:** short, singular and plural are clear, unambiguous in context, and it removes the "wiki document in the yaks document folder" circumlocution. A farm would hold yaks and doks, plus sources and schemas.
- **Boundary to settle first:** does "dok" mean only what is now a page, or also a source's `extracted.md` and a schema? The vocabulary only works if the noun covers one thing.
- **Cost:** `wiki` appears about 640 times across 78 files (code, tests, snapshots, docs, SVG frames), `yaks page` is the command family, `wiki/` and `<herd>/wiki/` are on-disk paths and link prefixes (`/wiki/...`), and Haks writes `wiki/digest/`. The on-disk layout is the contract, so a path rename needs `yaks migrate`.
- **Cheaper staging:** (1) concept, docs, skills, TUI tab label and `--help`; (2) `yaks dok` as an alias of `yaks page`; (3) the directory rename only if wanted, with a migration and link rewrite. Stage 1 alone leaves the old folder name visible, which partly recreates the confusion.
- **Minor:** "Doks" is also the name of a Hugo documentation theme, and "dok" is a plain word in Dutch and Afrikaans ("dock"). Neither seems a real problem.

## Open questions

- [ ] Should yaks take structured ask options (repeated `--option`, stored as note metadata) so UIs stop parsing prose?
- [ ] Should a harness's design decisions be filed as yaks by default, so a build like this leaves its reasoning in the farm and not only in a chat transcript?
- [ ] Is the reviewer's execution sandbox (PR code run with the owner's `gh` and `git` access) in scope for the first team trial, or does it block it?
- [ ] Which of the four upstream asks should yaks take first? Create-by-key and note metadata queries are the cheapest and remove the most Haks code.
- [ ] Does yaks want a touch or heartbeat primitive, and is the window (5 minutes live, 2 minutes quiet) Haks policy or a yaks default?
- [ ] What should the real gate on posting to GitHub be: a prompt rule, a scoped token, or Haks doing the posting?
- [ ] Is the hub (one farm for every repo, via `yaks.farm`) the layout we recommend, or one option beside a farm per repo?
- [ ] Is "dok" the word, and what does it cover? If yes, which stage of the rename do we do?
- [ ] Should Haks's JSON dependence on yaks `main` become a versioned contract?
