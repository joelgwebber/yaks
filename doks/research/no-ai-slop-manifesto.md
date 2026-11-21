---
type: topic
title: The No AI Slop manifesto, read against yaks
created: '2026-11-21T16:24:46Z'
tags:
- research
- design
description: What the No AI Slop manifesto asks of a tool like yaks, where yaks already does it, where it falls short (review, authorship, reader cost, outcome measures), and proposals; read before changing review, attribution or note conventions.
sources:
- sources/2026-11-21-7a2e5a46-no-ai-slop-a-manifesto-aie-code-summit-2026.md
stale_after: P6M
---

# The No AI Slop manifesto, read against yaks

Read on 2026-11-21 for [yaks-2c0d](/yaks/yaks/yaks-2c0d.md), at Joel's request: he agrees with the manifesto and asked how its principles show up in yaks, where they don't, and what yaks could change. Written by an agent (noslop-1); the numbers are from a script over the farm on that day, described under [Limits](#limits).

## In one paragraph

yaks already holds most of the manifesto's ideas about *verification*: evidence before shear, a lever named per repo, the coordinator's own re-run of every worker's change, lessons that turn into mechanisms. What it does not hold is the manifesto's central worry, **the reader's cost**. The farm cannot say which results a human looked at, it cannot always tell an agent's words from a person's, it asks agents to write notes for liveness rather than for a reader, and it measures itself in yaks shorn. Agents wrote about 98% of the farm's notes. The proposals below are small: record review and authorship as note metadata, keep heartbeats out of the reader's way, give the skills a size budget, and count rework instead of throughput.

## The manifesto in brief

[The source](/sources/2026-11-21-7a2e5a46-no-ai-slop-a-manifesto-aie-code-summit-2026.md) is short. It values outcomes over output, understanding over speed, our own thinking over a model's first draft, better work over easier work, and improving the system over patching the output. Its working definition:

> Slop is when we put less effort into producing something than we ask the consumer to spend understanding it.

The claims that bear on a tool like yaks:

- **Asymmetry.** Producing a thousand-line PR or a ten-page doc costs almost nothing; reviewing it costs what it always did.
- **Ownership and disclosure.** We own intent, taste and consequences. When an agent wrote it, say so, and check the parts that matter before it goes out.
- **Review something, by stakes.** Maybe not the code, but a plan, a design or a result. Agent reviews, linters and verifiers are not sufficient on their own. Unreviewed output is fine when the stakes are low.
- **Outcomes, not counts.** What shipped and whom it helped matters far more than lines, PRs, tokens or agents in parallel.
- **Compounding.** Slop in a knowledge base is what the next agent reads and copies.
- **Subtraction.** Slop grows from tools that lean toward adding.
- **Twice means a mechanism.** A mistake seen twice is fixed in the system; what a check can catch, a check catches.
- **Elicitation.** Tools should be built for efficient elicitation of human intent and efficient human understanding.

## Where yaks already lives it

| Principle | Mechanism in yaks |
|---|---|
| Outcome seen, not claimed | "Evidence before you shear" in the `yaks` skill; `yaks verify` records PASS/FAIL; `doctor --strict` flags a shorn yak without evidence; the brief says "Paste real output in notes; claim nothing unobserved" ([code:src/brief.rs@57c1d281#L203](code:src/brief.rs@57c1d281#L203)) |
| Verifiers aren't enough | `AGENTS.md`: verify through the project's lever, "not 'tests pass' alone", and look at the rendered frame; `yaks-coordinating`: "the cost is in review", re-run each worker's change, reject a claim you cannot reproduce |
| Understanding over speed | `Decision:` notes with the rejected alternatives; re-read the yak before starting; `yaks-research`'s "find before you write" and "save before you rely on it" |
| Human judgement where it counts | `yaks ask` for a design fork, with the agent's lean, recorded on the yak; agents never answer their own ask; never migrate a farm; `yaks-tracker` never posts to an outside tracker without a yes |
| Improving the system | The lessons dok has a budget, and a lesson that recurs "gets a yak for a mechanism, not more text" ([reflection design](/doks/design/session-reflection.md#budget-and-demotion)); `doctor` predicates; `cargo fmt --check` before landing |
| Compounding | `yaks check`: stale doks, uncited sources, doks whose cited code moved since the pin |
| Reader's time | Notes rules: short blocks, lists for parallel facts, a summary note that is neither "done" nor a design doc ([yaks-df02](/yaks/yaks/yaks-df02.md)); `--run-on` finds bodies that read as one paragraph |
| Honest self-assessment | Session notes grade each turn `good`/`partial`/`failed`, and call a correction "the signal that matters most" |
| Judgement outside the tool | "No model inside yaks": what can be decided from the files is a command, judgement is a skill ([knowledge store](/doks/design/knowledge-store.md#rules-that-hold-it-together)) |

The skills also shrink as well as grow: since 2026-10-21, 90 commits to `.agents/skills/` added 1,184 lines and deleted 1,611.

## Where it falls short

### 1. The farm cannot say what a human reviewed

Shorn means an agent decided it was done. Of 672 shorn yaks, 68 carry any note from Joel. That is a floor, not a measure: Joel reviews in chat and by reading in the TUI, and neither leaves a trace. So the farm cannot answer the manifesto's question "which subset did we review, and did it match the stakes?", and a later reader cannot tell a result a person checked from one only an agent vouched for.

[yaks-f814](/yaks/yaks/yaks-f814.md) rejected a `judge:` field: the judge is derivable from `verify:` (a script) and `needs: human` (a person). That covers sign-off *before* shear, which blocks the work. It has nothing for review *after* the fact, the common case and the cheap one.

The same gap repeats one level up. Session review (`yaks session promote`) and the weekly reflection are done by agents, so "reviewed" in `session list --unreviewed` means an agent read it. Agents reviewing agents is the case the manifesto calls useful but not sufficient.

### 2. Authorship is a naming convention, and it can fail toward the human

Every note carries an actor, which is good. Whether that actor is a person or an agent is not recorded: it is inferred from the name. The resolution order ([code:src/actor.rs@57c1d281#L1-L9](code:src/actor.rs@57c1d281#L1-L9)) is `--as`, `$YAKS_ACTOR`, `whoami --set`, a harness's environment, then git `user.name`. Two consequences:

- Only the harness-derived name carries an agent marker (`delta:`). The skills tell agents to `whoami --set` a name, and that name (`delta-lead`, `herd-unlock`, `noslop-1`) has none. The marker disappears exactly when an agent follows the rules.
- An agent in a harness yaks doesn't know (only Delta's variables are listed, [code:src/actor.rs@57c1d281#L56-L63](code:src/actor.rs@57c1d281#L56-L63)) that skips `whoami --set` writes as **the human**. That is the opposite of "when an agent wrote something, make that clear".

### 3. Notes written for liveness, not for a reader

The skills and the brief ask for a note "at least every 30 minutes" so that "your latest note is how others tell a busy worker from a dead one" ([code:src/brief.rs@57c1d281#L215](code:src/brief.rs@57c1d281#L215)). That makes a note do two jobs: a signal for `yaks sheds` and a message for a person. A heartbeat is output by design. It lands in the same pane, at the same weight, as a `Decision:` note.

Scale: the farm holds about 297,000 words of notes. Notes stamped by Joel come to about 4,500 (1.5%). The busiest agent actors write medians of 30 to 140 words a note and p90s of 300 to 380. None of that is wrong in itself, but the ratio is the manifesto's asymmetry in our own house.

### 4. The instructions are themselves a reading load

Every agent session reads the `yaks` skill (5,700 words) and often `yaks-coordinating` (3,100), `yaks-research` (2,500) or `yaks-upkeep` (2,500); `AGENTS.md` adds 2,000. The human reads them when they review a change. `docs/cli.md` is 20,000 words and `docs/tui.md` 13,000. The lessons dok has a budget (about 50 lines); nothing else does. The "update every surface in the same change" rule is right for parity, but each feature adds text to four to six surfaces, and each surface grows. In the last month `src/` and `docs/` gained 86,000 lines and lost 32,000.

### 5. Throughput is the visible measure

Session-note titles read "13 yaks shorn with Sonnet workers" and "seven workers, all landed". `yaks stats` counts by state. The batch note records each worker's model "so a later reflection can compare outcomes by tier", but the outcome recorded is that it landed. Nothing connects a shorn yak to what happened next: a bug filed against it, a regrow, a revert, Joel's correction in a later session. Agents also grade their own sessions: 21 `good` and 2 `partial` turns across this month's session notes, none `failed`.

### 6. Intent arrives secondhand

A yak an agent files from a conversation carries the agent's paraphrase of the request, not the request. This one does too: its description is mine, not Joel's message. Agents also file follow-ups freely (the skills tell them to), so the backlog fills with agents' ideas at the same weight as the human's. [yaks-02ac](/yaks/yaks/yaks-02ac.md) already notes that agents treat an `idea` yak as ready to implement.

## What yaks could do

Each is meant to be small and to fit the patterns yaks already has: note kinds rather than new fields ([yaks-f814](/yaks/yaks/yaks-f814.md)'s freight rule), derived queries rather than stored state, skills for judgement.

1. **A review note kind** (finding 1). `yaks review <id> --note "..."` writes a `kind: review` note with a verdict (`ok`, `concerns`, `redo`). A human's review is what counts; an agent may write one but it is labelled as an agent's. `yaks tick` and `inbox --for human` gain a short "shorn, not reviewed" list, ordered by stakes (priority, a `design` or format tag, size of the change), capped to a handful. A `redo` verdict regrows the yak. This also gives reflection its missing input: rework the human asked for.
2. **Authorship recorded, not inferred** (finding 2). `whoami --set <name>` records that the name is an agent's (a `--human` flag for the other case), and notes and the `Yaks-Actor` trailer carry it. Writes that fall through to git `user.name` from a known agent environment (Claude Code, Codex, Cursor and others each set a variable) are refused or marked. The TUI shows agent notes distinctly from human ones.
3. **Heartbeats out of the reader's way** (finding 3). A `kind: heartbeat` note (or `yaks update <id> --alive`) that `sheds` counts for liveness and the TUI and `show` fold by default, like moves. The skills then ask for a note at a milestone, and a heartbeat on the clock.
4. **A budget for the instructions** (finding 4). A test that fails when a bundled skill passes its word budget, set near today's size, so every addition argues for a subtraction. The lessons dok shows the pattern works.
5. **Rework as the outcome measure** (finding 5). A derived query, no new state: shorn yaks later contradicted, by a bug that links them, a regrow, a `redo` review, or a `yaks land` revert. Reflection reports it beside the verdict counts, by model tier, and session titles stop leading with counts.
6. **The human's words on the yak** (finding 6). A skill rule: a yak filed from a person's request quotes the request, then the agent's reading of it. Agent-filed follow-ups land as drafts ([yaks-02ac](/yaks/yaks/yaks-02ac.md)) that stay out of `yaks next` until a human accepts them.

What not to do: turn review into a gate on every yak. The manifesto says low-stakes unreviewed output is fine and that review should follow the stakes. A gate would also teach agents to ask for rubber stamps, which is slop of a different kind.

## Open questions

- [ ] Should a human's review stay optional and sampled (lean), or should some kinds of yak (a format change, a skill change) require one before shear?
- [ ] Is an agent-written `kind: review` useful at all, or does it blur the one distinction the note kind exists to make?
- [ ] Should yaks refuse an agent write that would be stamped with the human's git name, or only mark it?
- [ ] Do the skills' word budgets belong in a test (hard) or in `yaks check` (advice)?
- [ ] Does Joel want session notes to carry his own verdict beside the agent's?

## Limits

- Only the manifesto page was read. Its linked resources (swyx's talk, Tejas Kumar's post, Ghostty's AI policy, "Why Software Factories Fail") were not fetched.
- Note counts come from a throwaway script over `*/yaks/*.md`, splitting at note headings and skipping `moved:` notes. "Human" means the actor `Joel Webber`; agents that wrote under that name (finding 2) would be counted as Joel. Older notes without an actor (about 230) are counted as neither.
- Session-note verdicts were counted from the notes' `## Turns` lists, November 2026 only.
