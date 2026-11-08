---
type: topic
title: 'Agent memory and "dreaming": what to take for doks and lessons'
created: '2026-11-08T16:23:39Z'
tags:
- research
- doks
- agent
stale_after: P6M
sources:
- sources/2026-11-08-f2310691/source.md
---

# Agent memory and "dreaming": what to take for doks and lessons

Read on 2026-11-08 for [yaks-e4f5](yaks-e4f5). Joel asked what Anthropic's article says about keeping an agent's memory consolidated and current, and what that teaches us about keeping doks and lessons salient. This is a first read of one vendor article; what was and wasn't read is under [Limits](#limits).

## In one paragraph

The article describes a stack: **transcripts** (the log of what happened), **notes** (what the agent judged worth keeping), **representations** (wikis and indexes that make notes navigable) and **context** (the small layer loaded into every session). **Dreaming** is an offline pass that works across that stack: it checks notes against transcripts, builds and prunes representations, promotes a few notes to context and demotes the rest, and finds patterns no single session could see. The farm already has the shape of this stack and three of the dreaming passes (session review, reflection, `yaks check`). What it lacks is not the passes but three properties the article shows matter: a **budget on the top layer**, **evidence-driven checking** of what has drifted, and a **measurement** of whether a promoted lesson worked. And, today, the loop doesn't run: see [Where the farm stands](#where-the-farm-stands).

## What the article says

From [the saved text](/sources/2026-11-08-f2310691/extracted.md). Anthropic's own evaluations, so evidence of a claim, not of the truth.

- **Memory is a prediction about what a future session will need.** Strong models judge salience well on their own: told only that a memory folder exists, they used it well, and fixed rules ("record only failures") cost Opus 5 and 4.7 much of memory's gain. Older models needed steering (read your notes first; test your theories).
- **Store what isn't readable from the environment.** In Pokémon, Sonnet 3.7 wrote 413 files recording state the screen shows every turn; it went stale and the agent spent its steps tidying. Fable 5 wrote nine files and a `global.md` dashboard (current goal, a plan, its own rules for using memory) and got further.
- **Dreaming can correct memory.** A note was wrong from the start (which side of the mountain the landmark was on). Checking it against the transcript found the error; with the corrected note the agent finished the section, without it the run looped three times.
- **Dreaming can build representations.** A wiki built over 128 dreaming sessions took the score on 25 questions from 13% to 48%. Wiki plus search scored 61%, against 53% for search alone and 48% for the wiki alone, and saved about 20,000 tokens a question. The wiki helped on questions that combine several documents (87% against 73%) and not on ones a single document answers. On one task it grew larger than the documents it summarized.
- **Dreaming promotes and demotes.** The injected top layer must stay small: tokens cost, a wrong note is carried into every session, and longer instruction files lower adherence. Claude Code's auto memory loads only the first 200 lines or 25 KB of `MEMORY.md`, an index that points down. Much of dreaming's work at the top is moving detail down and leaving a link.
- **Dreaming can be tuned to a goal, and the tuning can be learned.** Several agents writing one shared table of contract terms overwrote each other (41 of 57 facts survived). A loop that reflected on the failure and rewrote the build instructions gave each contract its own page, quoting terms word for word (57 of 57).
- **Dreaming fills gaps across sessions.** A second pass repointed nine dangling links to an existing page and added an index.
- **The open costs:** in-session notes distract from the task; offline notes arrive late; checking everything is too expensive, so aim dreaming at what is likely to change and set the cadence by how often a memory is read.

## The stack, mapped to the farm

| Layer | The article | The farm today |
|---|---|---|
| Transcripts | session logs, Managed Agents transcripts | The harness's own thread (Delta, Claude Code) is **not** in the farm. What the farm holds is the event log: yak notes (timestamped, attributed), state moves, and one git commit per write. |
| Notes | memory files Claude writes as it goes | Yak notes (`Decision:` notes), and the **session note** an agent writes at a session's end (`## Turns`, `## Lessons`), which is written from the session's own memory, so it can be wrong. |
| Representations | wikis, indexes, links | Doks (`topic`, `howto`, typed doks), `sources/`, the link grammar, `yaks find`, `yaks refs`, yak families. |
| Context | `MEMORY.md`, injected at session start | `AGENTS.md`, the skills, and `/doks/lessons.md` (read at session start "when the farm has one"). |
| Dreaming | background passes | Session review (notes to doks), reflection (notes to lessons, one proposal per round), housekeeping (`yaks check`), the `stale_after:` / `verified:` fields. |

Two things the farm does that match the article without having read it: the session note ties together instead of restating (`## Saved` is generated, review "never copies what a yak already records"), which is "don't store what the environment shows"; and reflection can only *propose* edits to instruction files, one a round, through an ask, which is the article's caution that a wrong note in context is carried everywhere.

## Where the farm stands

The upkeep loop is built ([yaks-1c89](yaks-1c89)) but has barely run, so most of what follows is about what it should do once it does:

- There are **three** session notes, all unpromoted lessons. There is **no** `/doks/lessons.md`.
- None of the three recurring yaks the skill describes (review, reflect, tidy) exists. The only recurring yak, "Sweep the review queue" ([yaks-3979](yaks-3979)), is dead.
- So the stack stops at the notes layer. Nothing dreams, and nothing reaches context except by hand.
- The notes already contain exactly what the article says only a cross-session pass can see. `cargo fmt` reformatting files nobody touched is a lesson in all three notes. Parallel workers colliding on one shared doc section is in two. A flaky `tests/init_modes.rs` is in two.

## What to take

### 1. Admission rises with the layer

Write freely at the bottom, be strict at the top. The article's evidence points both ways: strong models do better judging salience themselves, but whatever reaches context is paid for in every session. The farm already has this gradient (yak notes are free, a session note is one per session, a lesson needs two sessions or a failed turn, `AGENTS.md` needs a human). Keep it, and don't add prescriptive rules for what an agent may write in notes or session notes. The threshold belongs on promotion.

### 2. The top layer needs a budget, and promotion needs a matching demotion

Nothing bounds what an agent reads at session start. Measured today: `AGENTS.md` is 225 lines and 12.9 KB; the `yaks` skill, the one every session needs, is 274 lines and 32 KB, over the 25 KB the article's example caps its index at. `/doks/lessons.md` will be read in full at every session start with no size limit, and the reflection procedure only ever adds to it: a lesson leaves only "when a later session showed it wrong".

Proposal: give `lessons.md` a budget (a line count, as Claude Code's is) and an index shape: a short top section of rules, each one line, linking down to the dok or session that holds the detail. Add the other way out for a lesson: **it is absorbed** (now in a skill or `AGENTS.md`, or the friction was fixed by a yak), so it leaves the dok with a link to where it went. Reflection's account then reports lessons removed as well as added.

### 3. Let evidence, not only the clock, say what to re-check

`yaks check` knows one kind of staleness: `stale_after:`, counted from `verified:` or the last commit. The article says to aim dreaming at what is likely to change, and to correct notes against ground truth. The farm has a worked example of drift going unseen: [/doks/design/knowledge-store.md](/doks/design/knowledge-store.md) still describes `wiki/`, `yaks page new`, a Wiki tab and `yaks import-hearth`; none exist now (`yaks page` is an unrecognized subcommand, the farm has `doks/`). The dok has no `stale_after:`, so no check could have flagged it. A smaller one: `AGENTS.md` lists four files `cargo fmt` reformats, and a `cargo fmt --check` today flags two.

Evidence the farm already holds: a dok's `code:` links name code, and `@sha` pins what was read. A model-free `yaks check` finding can say "this dok cites `src/x.rs`, which has changed in N commits since the pinned sha (or since the dok was last verified)". That is the file-level version of the article's "check the note against the transcript": the command finds *where to look*; an agent then reads the dok against the code and sets `verified:` (the existing rule that `verified:` means someone really re-checked). This follows the project's split: what can be decided from the files is a command, judgement is the skill.

Two cheaper habits go with it: say in a design dok which part of it describes current behaviour and which is history (the article's corrected note changed an outcome; ours would change what a worker builds); and give design doks that describe moving code a `stale_after:`.

### 4. A lesson carries its proof and its status

Opus 5's memory entries held a rule, how it knew the rule was true, the earlier answers, and warnings about traps; Opus 4.7 marked facts confirmed or unconfirmed. Lessons in the farm cite the sessions that taught them, which is provenance but not status. Worth adding per lesson: *how we know* (the failure it came from) and a **last seen** date or session. A lesson whose friction has not recurred in several rounds is a candidate for demotion (2); one that recurs is a different signal, next.

### 5. A lesson that recurs after being promoted didn't work; escalate to a mechanism

`cargo fmt` is the example in hand. `AGENTS.md` has warned about it since before the first of the three notes, and all three notes report it again; the cost is paid by every worker, by hand. The article says longer instruction files lower adherence, and our own project rule says the same thing from the other side: a warning in context is the weakest fix, a command is the strongest. So the reflection round, when it finds a lesson that is *already* in context and still recurring, should file the yak for the mechanism (here, probably formatting those two files once so `cargo fmt` stops touching them) and not propose more text. This costs one line in the reflection procedure and makes reflection check its own earlier promotions.

### 6. Keep a representation only while it pays, and keep search under it

Wiki plus search beat either alone, and the wiki's gain was on multi-document questions. For the farm this says the existing pairing (doks as the map, `yaks find` as the floor) is right, and that a dok should hold **claims with citations** that make the cross-document view cheap, never a second copy of what a yak's notes or a source already says. The failure to watch for is the article's map as large as the empire: a dok that restates its sources. At 11 doks there is nothing to fix; the signal worth having later is a dok larger than the material it cites.

### 7. One writer per load-bearing page, and quote what must survive

The shared contract table lost a quarter of its facts to overwrites; one page per contract, one agent each, terms quoted verbatim, lost none. The farm has the mechanisms (`dok set --if-hash`, the field-wise merge, conflict markers). The practice to keep: two parallel workers don't both extend one dok; the lessons dok has a single reflecting writer; a fact someone will act on (a flag name, a decision, a threshold) is quoted from its source with a link, not paraphrased into the dok.

### 8. A dok index is the entry point, as `MEMORY.md` is

`MEMORY.md` is an index "showing Claude what it knows and where to find it". The farm's equivalent is `yaks dok list`, which prints titles. Only one dok (`knowledge-store.md`) has a `description:`. A one-line description saying *when to read this* turns the generated list into the index an agent could load at session start in a few lines, and costs nothing to keep current because the list is derived from the files (the project's rule for any index). `yaks check` could flag a dok with no description.

### 9. Measure the loop the way the article does

The article's best result came from a loop that *re-ran and re-graded*: reflect on the failure, rewrite the instructions, rebuild, score again. Ours stops at the proposal: Joel accepts or rejects it, and nobody checks whether the friction went away. The material is there: each session note carries `good`/`partial`/`failed` counts and a `## Lessons` list. Cheap form: when a promotion is accepted, the yak says which friction it targets; the next round looks for it in the new notes and says "recurred" or "gone" in its account. With a handful of notes a round this is anecdote, not a score, but it is what makes (5) possible.

## What not to take

- **The scale.** 128 Opus sessions over 420M tokens of documents is not our corpus. The farm has attention, not compute, as its limit, and a handful of doks.
- **Wholesale model rewriting of the map.** The article's dreaming rebuilds wiki pages freely. Our farm commits every write, merges field by field, and has a human asking for review; keep edits small and attributable so a bad dream can be `yaks undo`ne.
- **Replacing the event log.** The article's wiki grew *out of* transcripts that stayed available as ground truth. Our ground truth is yak notes and farm history; never let a dok be the only place a decision lives.
- **Dreaming on every memory.** The article itself says checking everything is impractical. A fixed weekly cadence is a fine start; (3) is the cheap way to aim it.

## Recommendations

| # | What | Where it goes |
|---|---|---|
| 3 | `yaks check` finding for doks citing code that changed since pinned or verified | A feature yak (model-free) |
| 2, 4, 5, 9 | A budget and index shape for `lessons.md`, lessons with proof and last-seen, escalate recurring-after-promotion to a yak, check last round's promotions | One design yak with an ask: it edits `yaks-upkeep`, which is not edited unasked |
| 8 | Dok `description:` as the index line; flag when missing; maybe a session-start listing | The same design yak |
| – | Refresh `knowledge-store.md` for doks | A small yak (it is an instance of 3) |
| – | Start the loop: create the three recurring yaks, run the first reflection round by hand over the three notes | The human's call |

## Limits

- Anthropic describes its own evaluations. The Pokémon runs are one run each. The wiki scores are on 25 questions and a few tasks. Treat the shapes (small top layer, correct against ground truth, wiki plus search) as the finding, not the percentages.
- The figures were not read; every number above is from the article's text and captions.
- The two cookbooks the article ends with (dreaming, knowledge wikis) were not read.
- Mapping to the farm uses three session notes and one dok-drift example. The proposals are hypotheses about what will matter once the loop runs weekly.

## Open questions

- [ ] What does the cookbook for dreaming actually prescribe (prompts, cadence, what it logs)? It may answer the cadence question in (3).
- [ ] Should reflection also read yak notes directly, not only session notes? The article's dreaming reads transcripts; our nearest ground truth is the yak notes, which sessions link.
- [ ] Is there an "access" signal worth recording for doks (what an agent read) without committing per-user state? The article tunes cadence by it; the farm records no reads, and an index must stay derived, never a second source of truth.
