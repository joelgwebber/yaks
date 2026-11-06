---
type: topic
title: Reflecting on sessions
tags:
- design
- doks
---

# Reflecting on sessions

A proposal for [yaks-7168](yaks-7168): the weekly reflection Hearth planned (hearth-3285, never built), split between yaks and an agent harness by the rule the family follows. What can be decided from the files is a command; judgement is the `yaks-upkeep` skill; putting text in front of a model is the harness's.

## What review leaves undone

Review (the `yaks-upkeep` skill) reads one note at a time and moves its facts into doks. It never looks across notes, so it misses what only shows up in a pile of them:

- the same lesson learned twice, as schism's session notes already show (the `#[serde(untagged)]` trap, "look at the frame before calling it done");
- how the turns went over time, from the verdict counts each note already carries;
- friction with a tool, skill or doc that keeps coming back.

Reflection is that cross-note pass. It runs on a schedule, over the notes since the last round, reviewed or not.

## What it writes

- **A lessons dok**, `doks/lessons.md` (`type: topic`): short rules for working in this farm's project, grouped by subject, each citing the sessions it came from. A lesson goes in when two or more sessions taught it, or when one `failed` or `partial` turn did. Lessons that are about the project's subject (how Bevy resolves asset paths) are facts, and review already put them in subject doks; this dok holds how to work. It is edited like any dok: extend, merge duplicates, drop a lesson only with a note saying why.
- **Yaks for friction** with yaks or its skills, filed where the person tracks yaks' own work, as review already does.
- **Proposals, not edits, for instruction files.** A lesson that belongs in `AGENTS.md` or a skill becomes a yak awaiting the human (`yaks ask`), with the proposed text in it. Reflection never edits those files; at most one such proposal per round. This is Hearth's "at most one skill diff per cycle to review".
- **An account** as the recurring yak's shear note: the window, the verdict counts against last round's, lessons added or merged, yaks filed, the proposal if any.

## Closing the loop

A lessons dok helps only if agents read it. The `yaks` skill gains one line at session start: read `/doks/lessons.md` when the farm has one. A harness with its own memory (Hearth's `core/MEMORY.md`) can instead copy a few universal lessons into what it injects; that copy is the harness's, not yaks'.

## Mechanics yaks needs

- `yaks session list --since <date>`: the round's notes, reviewed or not. `session list --json` plus a `day` filter in the agent works today, so this is a convenience, after yaks-3f35's changes to `src/sessions.rs` land.
- Nothing else: the dok, the yaks and the asks use existing commands, and the round is a third recurring yak beside review and tidy (weekly, `--wake agent`).

## Stays in Hearth

Injecting memory into a model's context, judging transcripts the farm doesn't hold, and any trust rule about who may change instructions.
