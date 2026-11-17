---
id: yaks-535c
title: Directory sources store their text twice (files/ plus a joined extracted.md)
status: hairy
type: idea
priority: 4
created: '2026-11-07T15:58:16Z'
updated: '2026-11-17T21:17:07Z'
parent: yaks-4cfe
tags:
- doks
- cli
---

Found while ingesting sources in yaks-3610 (0.0.33).

Saving a 196-file repo snapshot (YASS) wrote `files/` (about 2.0 MiB of the original tree) and `extracted.md` (842 KiB: every text file concatenated, with headings). `yaks weight` then listed `extracted.md` as the second-largest blob in the farm. `yaks find` reads `extracted.md`, so it earns its place, but the same text now lives in two files and in every clone.

## Question

Could `find` and the TUI search `files/` directly for a directory source, so no joined copy is stored? Or is a joined file worth its weight because it is what an agent reads in one go?

Measure before changing anything: a typical directory source is a testbed of a few dozen files, where the cost is small. This is a weight and clarity question, not a bug. Related: yaks-9756 (checkout weight).

#### 2026-11-17T21:17:07Z

*sources-triage*

Measured in this farm (2026-11-17), now under yaks-4cfe.

- YASS: `files/` holds 170 files (about 1.5 MiB). The joined `extracted.md` is 842 KiB. `source.md` has a 250-line `files:` list.
- The joined copy causes two real bugs on top of its weight. Its relative links are wrong (yaks-289a). Rendering it makes the TUI crawl (yaks-760a). The `files:` list costs nothing in the TUI (measured), but it repeats the tree.
- `find` (`src/find.rs`) and the Doks tab's filter read only `source.md` and `extracted.md`, so the joined copy is all that makes a directory source searchable.
