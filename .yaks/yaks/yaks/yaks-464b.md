---
id: yaks-464b
title: 'Refresh /doks/design/knowledge-store.md: it still describes wiki/, yaks page and import-hearth'
status: shaving
type: task
priority: 3
created: '2026-11-08T16:24:07Z'
updated: '2026-11-10T21:14:45Z'
parent: yaks-1c89
tags:
- doks
---

Found while reading for [the agent-memory research](/doks/research/agent-memory-dreaming.md#3-let-evidence-not-only-the-clock-say-what-to-re-check) (yaks-e4f5). The dok says pages live in `wiki/`, are written by `yaks page new`, and that sessions are imported by `yaks import-hearth`; the farm now has `doks/`, `yaks dok`, and no import-hearth. Its `/wiki/x.md` link example and the "pages" vocabulary are the same drift.

Rewrite it to the current model (doks, sources, schemas, sessions) and add a `stale_after:`. It is design history that the Hearth framing explains, so keep what still holds and say what changed rather than only overwriting.

---
▸ 2026-11-10T21:11:51Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:14:45Z [herd-unlock]

Done (herd-unlock-2): [/doks/design/knowledge-store.md](/doks/design/knowledge-store.md) rewritten to the current model.

- Pages/`wiki/`/`yaks page`/Wiki tab became doks/`doks/`/`yaks dok`/Doks tab; sessions are `yaks session note`, import-hearth gone; a NOTE at the top says what changed and when, so the history stays legible.
- New "Keeping it current" section pointing at the upkeep loop; open questions updated (three closed, yaks-6cf2 added as the check that would have caught this drift).
- Frontmatter: `stale_after: P6M`, `verified: 2026-11-10`, tags `doks, design` (was `wiki`), description rewritten.
- Every yak it links exists; `yaks doctor` reports nothing for it.
