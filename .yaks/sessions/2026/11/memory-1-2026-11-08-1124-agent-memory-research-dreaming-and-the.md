---
type: session-review
title: 'Agent-memory research: dreaming and the doks maintenance stack'
source: session-note
actor: memory-1
person: Joel Webber
host: j15r
harness: delta
thread: ksQQobtw8z0aRtm7OxvMZr8p4JLOAE33iJIBxBCg_8DSgeNASrgoPHtnOl5O
day: 2026-11-08
reviewed: '2026-11-08T16:24:26Z'
good: 1
---

## Summary

Joel asked what Anthropic's "components of agent memory" article teaches about keeping doks and lessons current. I saved the article, mapped its stack (transcripts, notes, representations, context) onto the farm, and wrote [the findings](/doks/research/agent-memory-dreaming.md) ([the shear note](yaks-e4f5#2026-11-08T16:24:17Z)). Follow-ups are filed under yaks-1c89, with the skill and convention changes held on [one ask](yaks-2b40) to Joel.

## Turns

- good: read the article and say what to take for doks and lessons; delivered as a dok, a source and three yaks, with one ask. Awaiting Joel's reaction.

## Discussion

The finding that shaped the rest: the upkeep loop is built but not running (three session notes, no `lessons.md`, no recurring upkeep yaks), so the useful advice is about what it should do once it runs, and one cheap start (a hand-run first reflection).

## Saved

- yaks-2b40: Design: lessons dok budget and index, promotion measured, a dok index (created)
  - [ask: Six points from the research, each with my lean. All of them change the `yaks...](yaks-2b40#2026-11-08T16:24:17Z)
- yaks-464b: Refresh /doks/design/knowledge-store.md: it still describes wiki/, yaks page and import-hearth (created)
- yaks-6cf2: check: flag doks whose cited code has changed since the pin or last verification (created)
- yaks-e4f5: Research: Anthropic's agent-memory 'dreaming' and what it means for doks and lessons (created, hairy -> shorn)
  - [Done. Findings in /doks/research/agent-memory-dreaming.md; the article is sav...](yaks-e4f5#2026-11-08T16:24:17Z)
- created [/sources/2026-11-08-f2310691/source.md](/sources/2026-11-08-f2310691/source.md): Anthropic, 'The components of agent memory' (Lance Martin, 2026-11-06), text only; figures not read
- created [/doks/research/agent-memory-dreaming.md](/doks/research/agent-memory-dreaming.md): Agent memory and "dreaming": what to take for doks and lessons

## Lessons

- I ran `yaks sync` at the end and it pushed 8 farm commits; this repo's `AGENTS.md` makes publishing the farm the human's call. The `yaks` skill says to sync after landing work, which conflicts. Check `AGENTS.md` before any `sync` that may push.
- `yaks check` can't see a dok drifting from code, which is why `knowledge-store.md` went unflagged (yaks-6cf2).
