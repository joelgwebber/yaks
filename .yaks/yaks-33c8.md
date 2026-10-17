---
id: yaks-33c8
title: 'Idea: model-driven reformat of legacy yak bodies for CommonMark'
status: hairy
type: idea
priority: 4
created: '2026-10-17T04:25:58Z'
updated: '2026-10-17T04:25:58Z'
parent: yaks-e9de
labels:
- markdown
---

With CommonMark soft breaks (yaks-e9de), bodies written one-item-per-line (`Decision: …` / `Evidence: …` runs, pasted output) render as run-on paragraphs. A survey of this farm on 2026-10-17 found ~440 such adjacent plain lines vs ~90 genuinely hard-wrapped prose lines. Later: a `yaks` command or skill recipe that has a model rewrite a body into lists / blank-line paragraphs / fenced blocks, shown as a diff for approval. Not now.
