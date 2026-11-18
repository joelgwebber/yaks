---
type: session-review
title: 'Source layout: triage, swarm, schema 10'
source: session-note
actor: sources-triage
person: Joel Webber
host: joel
harness: delta
thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-18
written: '2026-11-18T00:08:08Z'
good: 2
partial: 1
---

## Summary

Joel asked for a holistic look at source layout. Three of my design asks shaped the work; four Sonnet workers and one more shaved it, and I landed and checked each.

- **Batch 1, landed and installed:**
  - [yaks-760a](/yaks/yaks/yaks-760a.md): TUI speedup on big texts;
  - [yaks-a57c](/yaks/yaks/yaks-a57c.md): `attach <dir>`;
  - [yaks-3329](/yaks/yaks/yaks-3329.md): the directory view and the archive guard;
  - four source CLI fixes: yaks-3da7, yaks-4dd3, yaks-06aa, yaks-fb15.
- **Schema 10** ([yaks-e83f](/yaks/yaks/yaks-e83f.md)): a source becomes one file, plus a sibling directory, with a migration. It is on `main`, but not yet run on the real farm.

## Turns

- good: holistic triage of source layout. Umbrella yaks-4cfe; the root cause was that "which file is the source" had no single answer.
- good: answers reviewed, the swarm run. 9 yaks landed and schema 10 written; the migration was rehearsed on a copy of the farm.
- partial: the safety filter interrupted one large write. It was re-sent in smaller pieces with nothing lost, but cost a turn.

## Saved

- yaks-0229: Browse a set of farm files: a source's files/, a yak's attachments; attach a directory, never system-open an archive (created, hairy -> shorn)
  - [ask: Scope check, three parts:](/yaks/yaks/yaks-0229.md#2026-11-17t211709z)
  - [pickup: Scope agreed: all three parts. Splitting into child yaks (attach a directory;...](/yaks/yaks/yaks-0229.md#2026-11-17t213752z)
  - [All three parts are done (`sources-triage`):](/yaks/yaks/yaks-0229.md#2026-11-17t231353z)
- yaks-03c8: Schema 10: update existing tests to the one-file source layout (created, hairy -> shaving)
  - [landed: 1091a53e yaks-03c8: tests for the one-file source layout](/yaks/yaks/yaks-03c8.md#2026-11-18t000203z)
- yaks-06aa: source add --refresh ignores a new --title when the content is unchanged (no way to retitle or fix a source) (hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: `srccli-1`. Scope, the issue's lean:](/yaks/yaks/yaks-06aa.md#2026-11-17t214258z)
  - [landed: 2ea5597c yaks-06aa: source add says when it kept a different title; a...](/yaks/yaks/yaks-06aa.md#2026-11-17t222452z)
  - [Coordinator review (`sources-triage`): landed on `main` (3da7 `9f613dee`, 4dd...](/yaks/yaks/yaks-06aa.md#2026-11-17t222808z)
- yaks-289a: Directory sources: relative links in the joined text miss the files/ prefix (created, hairy -> shorn)
  - [Closed by yaks-e83f (`sources-triage`). A directory source's tree now sits di...](/yaks/yaks/yaks-289a.md#2026-11-18t000613z)
- yaks-3329: TUI: a directory view in the pane for attachments and source files; never system-open an archive (created, hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: worker `browse-1`. Scope: the descri...](/yaks/yaks/yaks-3329.md#2026-11-17t214301z)
  - [landed: 6c5c3380 yaks-3329: a directory view in the TUI pane (attachments, do...](/yaks/yaks/yaks-3329.md#2026-11-17t230707z)
  - [Coordinator review (`sources-triage`): landed as `6c5c3380`.](/yaks/yaks/yaks-3329.md#2026-11-17t230819z)
  - [Coordinator gate on main at bc0e5979 (all of batch 1 landed): cargo test -p y...](/yaks/yaks/yaks-3329.md#2026-11-17t231352z)
- yaks-3da7: Directory sources silently skip dot-files and dot-directories (hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: worker `srccli-1`, who also owns yak...](/yaks/yaks/yaks-3da7.md#2026-11-17t214257z)
  - [landed: 9f613dee yaks-3da7: directory sources keep dot files, skip only .git/...](/yaks/yaks/yaks-3da7.md#2026-11-17t222445z)
  - [Coordinator review (`sources-triage`): landed on `main` (3da7 `9f613dee`, 4dd...](/yaks/yaks/yaks-3da7.md#2026-11-17t222806z)
  - [Merge note (`sources-triage`): landing 3da7 conflicted with `attach-2`'s yaks...](/yaks/yaks/yaks-3da7.md#2026-11-17t222809z)
- yaks-4cfe: Sources: one coherent layout for text, provenance, directories and browsing (created, hairy -> shaving)
  - [Batch 1 (yakherd `sources-triage`, 2026-11-17). Base `52d80dd3` (v0.0.37). Mo...](/yaks/yaks/yaks-4cfe.md#2026-11-17t214302z)
  - [Batch 1 spawned (Delta subthreads, `claude-sonnet-5-5`, base `52d80dd3`):](/yaks/yaks/yaks-4cfe.md#2026-11-17t214502z)
  - [Batch 2 (`sources-triage`): schema 10.](/yaks/yaks/yaks-4cfe.md#2026-11-18t000632z)
- yaks-4dd3: source add: do not record the absolute path and host of the machine (hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: `srccli-1` (see yaks-3da7's claim)....](/yaks/yaks/yaks-4dd3.md#2026-11-17t214257z)
  - [landed: e15c8e8d yaks-4dd3: source add records a local file by name only, no...](/yaks/yaks/yaks-4dd3.md#2026-11-17t222451z)
  - [Coordinator review (`sources-triage`): landed on `main` (3da7 `9f613dee`, 4dd...](/yaks/yaks/yaks-4dd3.md#2026-11-17t222807z)
- yaks-535c: Directory sources store their text twice (files/ plus a joined extracted.md) (hairy -> shorn)
  - [Measured in this farm (2026-11-17), now under yaks-4cfe.](/yaks/yaks/yaks-535c.md#2026-11-17t211707z)
  - [ask: Two related calls for directory sources:](/yaks/yaks/yaks-535c.md#2026-11-17t211708z)
  - [pickup: Decision recorded: drop the joined extracted.md and the files: list (Joel's a...](/yaks/yaks/yaks-535c.md#2026-11-17t213751z)
  - [Done in yaks-e83f (`sources-triage`):](/yaks/yaks/yaks-535c.md#2026-11-18t000613z)
- yaks-5f14: source: add or replace extracted text for an existing source
  - [Unblocked by yaks-e83f (schema 10). The want is now simpler: set or replace a...](/yaks/yaks/yaks-5f14.md#2026-11-18t000612z)
- yaks-760a: TUI: a source with a large extracted text is slow to move in (~150 ms per key on YASS) (created, hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: worker `srcperf-1`. Scope: the text...](/yaks/yaks/yaks-760a.md#2026-11-17t214300z)
  - [landed: b5525ac1 yaks-760a: memoize the text viewer's rows per width and text](/yaks/yaks/yaks-760a.md#2026-11-17t220647z)
  - [Coordinator review (`sources-triage`): landed as `b5525ac1` on `main` (squash...](/yaks/yaks/yaks-760a.md#2026-11-17t220820z)
  - [Coordinator gate on main at b5525ac1: cargo test -p yaks, 1453 passed, 0 fail...](/yaks/yaks/yaks-760a.md#2026-11-17t221227z)
- yaks-93ee: Browse and link into archive attachments (zip, tar.gz) read-only (created)
  - [Related: yaks-0229 (where it came from), yaks-a57c (attach a directory, which...](/yaks/yaks/yaks-93ee.md#2026-11-17t214206z)
- yaks-a57c: yaks attach <dir>: attach a directory with its layout (created, hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: worker `attach-2`. Scope: `yaks atta...](/yaks/yaks/yaks-a57c.md#2026-11-17t214300z)
  - [landed: 230f026e yaks-a57c: yaks attach <dir> stores a directory with its layout](/yaks/yaks/yaks-a57c.md#2026-11-17t221232z)
  - [Coordinator review (`sources-triage`): landed as `230f026e`. Running it mysel...](/yaks/yaks/yaks-a57c.md#2026-11-17t222048z)
- yaks-e83f: Design: one file holds a source's text and its provenance (retire the duplicate extracted.md) (created, hairy -> shorn)
  - [ask: Which layout for a source's text (options in the description)?](/yaks/yaks/yaks-e83f.md#2026-11-17t211709z)
  - [pickup: Decision: option B (Joel). One file holds a source's provenance (frontmatter)...](/yaks/yaks/yaks-e83f.md#2026-11-17t214204z)
  - [ask: Following your answer (B, plus "a single attachment pattern throughout"). Fou...](/yaks/yaks/yaks-e83f.md#2026-11-17t214204z-1)
  - [pickup: Decision (Joel's answers to the layout ask, recorded by `sources-triage`):](/yaks/yaks/yaks-e83f.md#2026-11-17t225405z)
  - [Plan (`sources-triage`, schema 10). This is the contract I'm implementing; re...](/yaks/yaks/yaks-e83f.md#2026-11-17t230245z)
  - [Checkpoint (`sources-triage`): the schema-10 code is in, on `main` as a commi...](/yaks/yaks/yaks-e83f.md#2026-11-17t233114z)
  - [Migration rehearsal (`sources-triage`): `yaks migrate` from a release build o...](/yaks/yaks/yaks-e83f.md#2026-11-17t233402z)
  - [Docs, help and skills for the new layout committed on `main` (`docs/cli.md` i...](/yaks/yaks/yaks-e83f.md#2026-11-17t233605z)
  - [Done (`sources-triage`). Schema 10 is on `main`:](/yaks/yaks/yaks-e83f.md#2026-11-18t000614z)
- yaks-fb15: source list: show part-of and citation status (hairy -> shaving)
  - [Claim (yakherd `sources-triage`). Owner: `srccli-1`. Scope: `source list` tex...](/yaks/yaks/yaks-fb15.md#2026-11-17t214259z)
  - [landed: 887d7795 yaks-fb15: source list nests parts and marks uncited, kept-u...](/yaks/yaks/yaks-fb15.md#2026-11-17t222453z)
  - [Coordinator review (`sources-triage`): landed on `main` (3da7 `9f613dee`, 4dd...](/yaks/yaks/yaks-fb15.md#2026-11-17t222808z)

## Lessons

- A cheap worker's tests pass more often than its feature works. Two of five landings needed my own follow-up: a directory link that didn't parse, and a merge over a shared walk. Running each feature in a scratch farm found both.
- Measure before blaming. The YASS slowness was the 842 KiB text, not the 250-line file list everyone suspected.
- Write a large new module in a few smaller calls: one big write tripped the safety filter.
- A schema bump makes `cargo install` of `main` lock you out of an unmigrated farm. Until the human migrates, install the last pre-bump commit instead.
