---
type: topic
title: 'Haks: an orchestration layer over yaks'
tags:
- research
- design
- agent
---

# Haks: an orchestration layer over yaks

Read on 2026-11-03 by Joel and `delta:haks-review`, from `subtextdev/haks` at `8fbe3ec` (Go plus a React board, about 15k lines, 38 commits, all in one day) and Clint Ayres's own design sketch. The repo is private and early, so this page stays at design level: no code is copied, and neither the repo nor the sketch is saved as a source. Tracked in yaks-96d8.

## What it is

Haks gives one person (and, later, a team) a single view across many herds. It does not store anything of its own.

- **Reads:** `yaks --json`, always against an explicit farm (`YAKS_DIR`). It never opens `.yaks/` files.
- **Writes:** only through yaks commands, stamped with its own actor (`haks`, or `haks:<login>` on a team farm).
- **Derives:** one snapshot (streams, herds, yaks, sheds, inbox, blocks, activity, review stage and order, stacks). The CLI (`haks board`, `haks reviews`, `haks next`) and the web board both render that snapshot. The board holds no business logic by design.
- **Local-only:** spend and control. Run registry, cost, `haks stop`, the pause flag.

Three kinds of farm: personal (your streams and to-dos), team (shared PR work, one yak per PR) and watch (a read-only mirror of someone else's farm).

## What the layering confirms

This is the first outside evidence for the line drawn in [the knowledge-store design](/wiki/design/knowledge-store.md): yaks owns what can be decided from the files alone; an agent layer owns what needs a clock, a network, a model or knowledge of who is asking.

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

Clint lists four of these himself as asks for upstream. The rest are visible in the code.

| Workaround | What yaks lacks | Fits yaks? |
|---|---|---|
| `haks reconcile` merges duplicate PR yaks (oldest wins, rest slaughtered as duplicates) | `yaks create --key <source-url>` that returns the existing yak | Yes. Decidable from files alone; makes intake idempotent and deletes reconcile. |
| Per-reviewer state kept as tags (`reviewer:`, `skipped:`, `reviewed:`) | A query over note metadata (`yaks find --meta kind=ask`) | Yes, for the query. The per-actor tag pattern is a separate question. |
| Cross-farm references are "not built yet" | A way for a personal yak to point at a team yak and back | Yes in principle, but hard: it touches the link grammar and `yaks.farm`. |
| `run_seen` frontmatter field set with `--set` to keep a run alive without a note | A touch or heartbeat with an expiry | Probably a primitive (touch, no note, no activity line) in yaks, with the window left to Haks. Overlaps yaks-0071 and yaks-039b. |
| Review options, lean, verdict and PR author parsed out of note and description text | Structured note fields | Partly. Better solved by Haks writing the structure itself (`--meta`) than by yaks parsing prose. |
| First creation of a team farm's branch is not race-safe | Idempotent `yaks init` against a remote branch | Probably. Clint's workaround is "one person starts it and pushes". |

## Observations and risks

- **The "nothing posts before you answer" gate is a prompt rule.** The reviewer run is a headless agent with `gh`, `git` and `yaks` shell access and file writes, told by a skill not to post or push before the owner's answer. Nothing mechanical stops it, and the PR it reads is untrusted input. Fine for a first try on your own machine. For team use it deserves a real gate (a token without write scope until the answer lands, or posting done by Haks rather than by the agent).
- **Authority is split between skills.** The `haks` skill says "where they differ, this one wins" over the `yaks` skill (sync after every write, shear only after merge, no self-created streams). That is reasonable, but it is a fork of policy that drifts as upstream skills change. The daily job copies upstream's skills into the repo, which helps; the override list should stay short and explicit.
- **Team path is simulated.** The two-machine test is a good harness, but real use has not happened. Same for fix-mode reviews and sessions (per Clint's sketch).
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

- [ ] Which of the four upstream asks should yaks take first? Create-by-key and note metadata queries are the cheapest and remove the most Haks code.
- [ ] Does yaks want a touch or heartbeat primitive, and is the window (5 minutes live, 2 minutes quiet) Haks policy or a yaks default?
- [ ] What should the real gate on posting to GitHub be: a prompt rule, a scoped token, or Haks doing the posting?
- [ ] Is the hub (one farm for every repo, via `yaks.farm`) the layout we recommend, or one option beside a farm per repo?
- [ ] Is "dok" the word, and what does it cover? If yes, which stage of the rename do we do?
- [ ] Should Haks's JSON dependence on yaks `main` become a versioned contract?
