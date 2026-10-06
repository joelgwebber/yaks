---
id: yaks-c061
title: 'Answered questions are hard to find: no ''answered, pending agent pickup'' surface'
type: feature
priority: 3
created: '2026-09-24T12:39:58Z'
updated: '2026-10-06T04:08:28Z'
labels:
- cli
- ui
---

Coming back to a farm (a fresh agent context, or a human after a break), there's no reliable way to find **questions the human has answered that an agent now needs to pick up**. Found by accident on 2026-09-24 during the lightning-round wrap-up (yaks-77a5).

## What happened

- The coordinator raised `yaks ask` on yaks-7cb3 and yaks-97a2, both **shorn**: they were post-merge follow-up questions.
- The human replied (2026-09-24T01:04Z / 01:06Z) as **plain notes**, probably via a comment rather than the TUI `a` key. `needs: human` stayed set on both (committed that way in 47c8ef4).
- The coordinator only noticed because it happened to dump `yaks log --since … --json` for an unrelated timeline, and the human's notes were at the end. Nothing pointed at them.

## Why a fresh context wouldn't see it

1. **A reply that isn't `answer` leaves the block set.** `yaks inbox` kept listing both as "awaiting a human" when they were really waiting on an agent. The only clue is reading each inbox yak's notes and seeing that the latest one is from someone other than the asker, after the ask. Nothing surfaces that.
2. **Even a proper `answer` on a non-hairy yak vanishes.** `answer` clears `needs`, which "returns the yak to `next`", but `next` lists only **hairy** yaks. An answered **shorn** (or shaving) yak drops out of `inbox` *and* `next`, so it's invisible either way. Asking on a shorn yak is natural (post-merge review questions); the CLI already warns "blocking finished work", but the round-trip has no way back.
3. **No "answered, pending pickup" state exists.** Once `needs` is cleared, nothing records that an agent still owes follow-up work. The answer note is the only trace.

Checked in code: the TUI `a` on a blocked yak opens the Answer prompt and submitting clears `needs` (handlers.rs `open_ask_or_answer` → `EditAction::Answer` → `set_needs_edit`), so `a` itself isn't broken. The gap is the other reply paths, and what happens after the answer.

## Candidate fixes (none chosen)

- **Detect replies:** `inbox` (and the TUI Inbox view) flags a blocked yak whose latest note after the ask is from a different actor as "replied", or sorts it apart. Cheap, no format change, derived from notes.
- **Nudge at reply time:** commenting on a yak with `needs` set asks "clear the block (answer)?", or offers answer as the default action.
- **Make answered work findable:** `answer` on a non-hairy yak either leaves a pickup marker (e.g. `needs: agent`, reusing the existing field with the direction flipped), or `next`/a new `inbox --answered` lists yaks answered since X that nobody has touched since.
- **Skill habit (no tool change):** the yaks skill's session-start step could include `yaks log --since <last session>` filtered to notes from non-agent actors on yaks the agent asked about. That's how this was actually found.

The `needs: agent` flip is attractive: it keeps one field, gives `inbox` a symmetric counterpart ("awaiting an agent"), and survives any context loss because it's in the file. A human answering (`a` or `yaks answer`) would set it on yaks an agent asked about; an agent clears it when it picks the work up.

---
▸ 2026-10-05T22:36:13Z [Joel Webber]
Possibly the same as yaks-f313?

---
▸ 2026-10-06T03:58:04Z [delta-lead]
Decision (Joel, 2026-10-06, in chat on the coordinator's question "should I take the first slice; the `needs: agent` flip is a fork for you"): "I do like `needs: agent` as an approach." Recorded here by the coordinator (delta-lead) because the question was asked in chat, not as an ask on this yak (his uncommitted edits on this file blocked me from writing here at the time); his words are quoted exactly.

Joel also noted it is interesting that this, of all cases, did not arrive as a question on the yak itself. It is the same gap this yak describes, one level up: a reply or a question that travels outside the yak leaves no durable trace and no pickup state.

## Settled design (mechanics decided by the coordinator, behaviour as Joel approved)
1. `needs` is already free-form (`yaks ask --needs`, default `human`). Convention: `needs: human` = awaiting a human; `needs: agent` = answered, awaiting an agent to pick the work up.
2. `yaks answer <id> --note ...` on a yak whose `needs` is `human` records the answer note and sets `needs: agent` (it no longer just clears). On a yak with no `needs`, behaviour is unchanged (an error, as today). `yaks answer --done` records the note and clears `needs` entirely, for an answer that needs no follow-up.
3. The agent clears it with a new explicit verb, `yaks pickup <id> [--note ...]`: clears `needs: agent`, appends a note attributed to the actor ("picked up"). No other command clears it implicitly.
4. A `needs: agent` yak is not blocked: a hairy one appears in `yaks next` (marked as answered), and any status appears in `yaks inbox`.
5. `yaks inbox` shows two sections, "awaiting a human" (`needs: human`) and "answered, awaiting an agent" (`needs: agent`); `--for human|agent` narrows it; `--json` carries each yak's `needs`.
6. Slice 1 (no format change): in `inbox`, a `needs: human` yak whose latest note after the ask is from a DIFFERENT actor than the asker is flagged `replied` (a plain-note reply that did not use `answer`), derived from the notes. This is the failure that started the yak.
7. The TUI must not show `needs: agent` as awaiting a human (check where `needs` is read; keep TUI changes minimal and verify with a headless frame); a distinct badge is welcome but not required.
Out of scope: cross-checkout visibility (yaks-7204), a waiting command (yaks-f313), coordination skills' prose (the coordinator updates them; the shipped `yaks` skill sections on asking/answering are yours).

---
▸ 2026-10-06T03:58:04Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T03:58:04Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker agt-1. Scope: src/main.rs (answer, inbox, the new pickup verb, next), src/farm.rs and store as needed for the needs transitions and the replied detection, the TUI only where `needs` is read, docs/cli.md, README.md, `.agents/skills/yaks/SKILL.md` (asking/answering sections and the command table), `--help`. Judge: coordinator re-runs the gate and drives the loop in a temp farm (ask, plain-note reply, answer, inbox both sections, pickup). Forbidden: changes to the on-disk format beyond the existing free-form `needs:` values, the farm lock rule, coordination skills prose.

---
▸ 2026-10-06T03:59:04Z [agt-1]
agt-1 start. `ls .yaks/*/yaks-c061.md` -> `.yaks/shaving/yaks-c061.md`; `date -u` -> Tue Oct  6 03:58:40 UTC 2026. Own binary built: ./target/release/yaks.

![inbox-answered-frame](artifacts/yaks-c061/inbox-answered-frame.txt)

---
▸ 2026-10-06T04:08:10Z [agt-1]
Headless TUI frame (yaks tui --headless, Inbox view): yak-77d4 (shorn, answered) and yak-205d (answered) carry the answered badge, only yak-8e8f (needs: human) carries the hourglass.

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: how `replied` finds the asker. `ask` now always records a note whose first line is `asked: needs <who>` (then a blank line and the question, if any); `replied` = the latest `asked:` note's actor vs the latest later note that is not a `moved:`/`picked up` entry. Rejected: (a) no marker, guess the asker from note order (false positives: a human comment before the ask would flag it); (b) a new frontmatter field for the asker (on-disk format change, forbidden); (c) matching timestamps against `updated:` (fragile). Cost: `ask` with no `--note` now writes a one-line note too (it also names the asker). A `needs:` set by hand or before this change has no `asked:` note, so it is never flagged `replied`; nothing migrates.

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: `agent` is the only special `needs` value; every other value (`human`, or a free-form `--needs foo`) counts as an open question: it blocks `next`, lands in the "awaiting a human" section, and `answer` hands it to an agent. Rejected: restricting the hand-off to exactly `human` (an `ask --needs foo` yak would keep today's clear-on-answer, a second rule for the reader).

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: `answer` on a yak with no `needs` keeps what the code does today (the settled-design text says "an error", but today it records the note, prints "Answered" and changes nothing): it records the note, leaves `needs` unset, and the message now says "no needs block was set (note recorded)". Rejected: making it an error (a behaviour change nobody asked for; the TUI `a` key cannot reach it anyway). `answer` on `needs: agent`: state kept, note appended, message "still needs agent". `answer --done` on any `needs` clears it; `--done` on no needs just records the note.

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: `ask` always allowed on `needs: agent`; it sets `needs: human` and prints "(was answered, awaiting an agent; now awaiting a human again)". Rejected: refusing until pickup (a human follow-up question on an answered yak is natural, and the answer note stays in the history).

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: `pickup` writes the note `picked up` (then a blank line and `--note`, if given); on a yak that is not `needs: agent` it exits 1 with "not awaiting an agent (needs: human)" / "(no needs block set)" and changes nothing. Rejected: a no-op success on a clear yak (hides a mistaken id/state; an agent loop would think it took work it did not). No `moved:` entry: a `needs` change is not a status move.

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: `inbox` text output is two titled sections (a section is omitted when empty; `--for` shows one); `--json` stays ONE flat array (human rows first), each row the normal task object plus a `replied` boolean (always present on inbox rows, `false` for agent rows), so a script filters on `needs`. `list --needs`/`bulk --needs` keep selecting ANY `needs` value (inbox's predicate). Rejected: a JSON object `{human:[],agent:[]}` (breaks the existing array shape consumers rely on); `replied` only when true (optional keys are harder to script).

---
▸ 2026-10-06T04:08:10Z [agt-1]
Decision: TUI. Rows/detail: `needs: agent` gets a `✅` badge (no hourglass, no warning accent; detail line `Needs: agent (answered, awaiting pickup)`), `⏳` is reserved for awaiting-a-human. `a` answers only a yak awaiting a human; on a `needs: agent` yak it opens ASK (a human follow-up that flips it back), since the TUI has no pickup (an agent verb). Answer in the TUI hands to an agent like the CLI (no `--done` key). The Inbox view is unchanged (`needs_only` = any `needs`). Also fixed `disp_width` to count U+2705 as width 2 (the badge was mis-measured and dropped from the row). Rejected: a `P` pickup key (humans do not pick up) and showing the replied flag in the TUI (not asked; CLI-only for slice 1).

---
▸ 2026-10-06T04:08:28Z [agt-1]
Evidence. Gate: `cargo test -p yaks` green (436 unit + integration incl. new tests/needs_loop.rs; new tests reference the new `ask`/`answer`/`pickup` API and `needs: agent` rules, so they cannot pass on the old code).
Real run in a temp farm (own binary, actors coord/joel/agt):
```
coord$ yaks ask yak-04d8 --note "Which way: A or B?"   -> Asked yak-04d8: needs human (dropped from next until answered)
joel$  yaks update yak-04d8 --note "B, please."
joel$  yaks inbox
Awaiting a human:
  [H] yak-04d8  p3 task     Q yak ⚠ needs:human ↩ replied
joel$  yaks answer yak-04d8 --note "Confirmed: B."     -> Answered yak-04d8: needs agent (awaiting pickup; see `yaks inbox`)
joel$  yaks inbox
Answered, awaiting an agent (`yaks pickup <id>`):
  [H] yak-04d8  p3 task     Q yak ✓ answered (needs:agent)
joel$  yaks inbox --for human   -> Inbox empty: nothing awaiting a human.
coord$ yaks next
  yak-04d8  p3 task     Q yak ✓ answered, awaiting pickup
agt$   yaks pickup yak-04d8 --note "Doing B."          -> Picked up yak-04d8: needs cleared
agt$   yaks pickup yak-04d8   -> error: yak-04d8 is not awaiting an agent (no needs block set); nothing to pick up
agt$   yaks inbox               -> Inbox empty: nothing awaiting a human or an agent.
```
`inbox --json` rows carry `"needs": "human"|"agent"` and `"replied": true|false`. TUI frame attached above.

---
▸ 2026-10-06T04:08:28Z [agt-1]
moved: shaving -> shorn
