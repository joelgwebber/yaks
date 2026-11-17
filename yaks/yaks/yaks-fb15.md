---
id: yaks-fb15
title: 'source list: show part-of and citation status'
status: shorn
type: feature
priority: 3
created: '2026-11-07T15:58:16Z'
updated: '2026-11-17T22:24:53Z'
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

[yaks-fb15-scratch](yaks-fb15/yaks-fb15-scratch.txt)

#### 2026-11-17T22:20:18Z

*srccli-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-17T22:20:32Z

*srccli-1*

Decision: reuse `check`'s rule by extracting it, not copying it: `check::citations(root, &sources) -> Citations {parent, top, is_cited, parts_of}` is now what `check::uncited` and `sources::list_rows` both call (a behaviour-preserving move; the existing `check` tests still pass). Rejected: a second copy of the citation scan in `sources.rs`.

Decision: text format. A part is `└ ` under its parent (one level of indent per depth, in the existing id order within a family); markers go at the end of the row: `[uncited]` only on a family head that `check` would list, `[kept uncited: <why>]` and `[superseded by <id>]` from the marks. A part never says `[uncited]`, because `check` treats the family as one. Rejected: a column before the title (it would shift every existing row), and marking parts (noise, and contradicts `check`).

Decision: `--json` unchanged. A free field was possible (`cited`), but the claim says leave it alone unless free, and `fields.part_of`, `fields.uncited`, `fields.superseded_by` already carry the inputs.

Decision: a `part_of` loop has no head; the walk lists anything it never reached at top level, so no source is dropped from the listing.

Evidence: the real farm's listing (`yaks source list`, attached as `yaks-fb15-list.txt`; 28 rows, 5 nested: the yass playbooks under the yass repo, three manifesto screenshots under the tweet, and one more). `yaks check` reports 0 uncited sources in this farm, and the listing has 0 `[uncited]` markers, so they agree. A scratch farm with every case (attached as `yaks-fb15-scratch.txt`):

    sources/2026-11-17-7427d152-old  text/markdown  Old  (o.md) [superseded by 2026-11-17-a4fb6214]
    sources/2026-11-17-a4fb6214-new  text/markdown  New  (n.md) [kept uncited: kept as a testbed]
    sources/2026-11-17-fe8edeeb-tweet  text/markdown  Tweet  (t.md) [uncited]
    └ sources/2026-11-17-cbc80bb5-shot  text/markdown  Shot  (s.md)

and there `yaks check` says `sources/2026-11-17-fe8edeeb-tweet is cited by no dok or yak (nor are its 1 part saved with part_of)`.

Test: `list_rows_nest_parts_and_mark_uncited_and_superseded` (nesting, family marker on the head only, cited, superseded, kept uncited, a lone uncited, and citing the part clearing the family). Docs: `source list --help`, `docs/cli.md`, `yaks-research` skill.

#### 2026-11-17T22:20:33Z

*srccli-1*

Summary: `source list` indents parts and marks `[uncited]`, `[kept uncited: why]`, `[superseded by id]`, using `check`'s extracted citation rule. Gate PASS (`yaks verify`), fmt clean.

#### 2026-11-17T22:20:34Z

*srccli-1 · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzJXZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-17T22:24:53Z

*sources-triage · kind: landed · commit: 887d7795969849ab3ede8a5cf21c795b6f17c983 · source: 8c0ec5d3d0fddc77e37a76e3f6fd0fa6e3d2f129 · branch: main*

landed: 887d7795 yaks-fb15: source list nests parts and marks uncited, kept-uncited and superseded sources (check's citation rule, shared)
