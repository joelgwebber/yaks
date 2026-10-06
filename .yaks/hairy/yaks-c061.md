---
id: yaks-c061
title: 'Answered questions are hard to find: no ''answered, pending agent pickup'' surface'
type: feature
priority: 3
created: '2026-09-24T12:39:58Z'
updated: '2026-10-05T22:36:13Z'
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
