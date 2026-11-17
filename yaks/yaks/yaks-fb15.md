---
id: yaks-fb15
title: 'source list: show part-of and citation status'
status: shaving
type: feature
priority: 3
created: '2026-11-07T15:58:16Z'
updated: '2026-11-17T22:16:45Z'
parent: yaks-4cfe
tags:
- cli
- doks
---

Found while ingesting sources in yaks-3610 (0.0.33).

`yaks source list` prints one flat row per source (id, media type, title, url). A source saved with `--part-of` looks the same as a top-level one, so I had to read each `source.md` to confirm my three manifesto screenshots hung under the tweet. `yaks check` knows about parts and about uncited sources, but `list` shows neither.

Add to the text output: indent a part under its parent, and mark uncited sources (and `uncited:` / `superseded_by:` marks). `--json` already carries `fields.part_of`.

Small and local: the `source list` arm and its listing function.

#### 2026-11-17T21:42:27Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:42:59Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: `srccli-1`. Scope: `source list` text output indents a part under its parent and marks uncited, `uncited:` and `superseded_by:` sources. Reuse `check`'s citation logic; don't duplicate it. `--json` unchanged unless a field is free to add. Evidence: a test, the gate, and the real listing of this farm pasted in the note.

[yaks-fb15-list](yaks-fb15/yaks-fb15-list.txt)
