---
id: yaks-96d8
title: 'Research: Clint''s haks, the orchestration layer over yaks'
status: shorn
type: task
priority: 2
created: '2026-11-03T22:40:30Z'
updated: '2026-11-11T20:00:41Z'
tags:
- research
- design
---

Joel asked for a read of https://github.com/subtextdev/haks (Clint Ayres, private, early) and of Clint's design sketch, then a discussion and findings in the wiki.

Findings go in `/wiki/research/haks-orchestration-layer.md`. Also records the "doks" naming idea as an open question.

---
▸ 2026-11-03T22:42:09Z [delta:haks-review]

Findings written to [the haks page](/doks/research/haks-orchestration-layer.md): the layering, patterns worth keeping, where Haks works around yaks, risks, general-vs-Clint split, and the "dok" naming question (open, no decision).

Source read: `subtextdev/haks` at `8fbe3ec`, via `gh repo clone`. Not saved as a source (private repo).

Open questions are checkboxes on the page. Waiting on Joel for discussion.

---
▸ 2026-11-03T22:42:09Z [delta:haks-review]

kind: move
from: hairy
to: shaving
thread: ksQQ5qxMm5B_TcKPJ13IMRVoupLOAEJMk5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T23:09:05Z [delta:ksQQ5qxMm5B_TcKPJ13IMRVoupLOAEKh-5IBxBCA]

Added provenance: a summary of Clint's Indent thread, plus his pushed `yaks` branch, folded into [the haks page](/doks/research/haks-orchestration-layer.md) (new sections "Provenance" and "What the thread adds", three new open questions).

- Source: [Indent thread summary](/sources/2026-11-03-81364278/source.md). Produced by a read-only Indent sandbox agent that read the conversation text only (no tool calls, no screenshots). A model's reading, not the transcript.
- Farm finding: Clint's `haks` herd has 8 yaks, all from before the thread, and none of the build is filed there. The design history exists only in the thread and in git.
- Corrected my first pass: the "asks for upstream" were written by the thread's agent, not Clint, and the sketch looks like that agent's lifecycle summary.

---
▸ 2026-11-04T17:18:22Z [delta:ksQQ5qxMm5B_TcKPJ13IMRVoupLOAEMQgpIBxBCA]

Naming question split out: the rename is filed as yaks-cc40 (full rename, no aliases, schema 7). The page's naming section stays as the rationale; the design questions on the page wait until that lands.

---
▸ 2026-11-04T21:18:37Z [Joel Webber]

Note: We committed to the "doks" naming and changed it throughout the code.

---
▸ 2026-11-11T20:00:40Z [sched-1]

Done

---
▸ 2026-11-11T20:00:41Z [sched-1]
kind: move
from: shaving
to: shorn
---

moved: shaving -> shorn
