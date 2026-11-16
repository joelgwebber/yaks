---
id: yaks-535c
title: Directory sources store their text twice (files/ plus a joined extracted.md)
status: hairy
type: idea
priority: 4
created: '2026-11-07T15:58:16Z'
updated: '2026-11-07T15:58:16Z'
tags:
- doks
- cli
---

Found while ingesting sources in yaks-3610 (0.0.33).

Saving a 196-file repo snapshot (YASS) wrote `files/` (about 2.0 MiB of the original tree) and `extracted.md` (842 KiB: every text file concatenated, with headings). `yaks weight` then listed `extracted.md` as the second-largest blob in the farm. `yaks find` reads `extracted.md`, so it earns its place, but the same text now lives in two files and in every clone.

## Question

Could `find` and the TUI search `files/` directly for a directory source, so no joined copy is stored? Or is a joined file worth its weight because it is what an agent reads in one go?

Measure before changing anything: a typical directory source is a testbed of a few dozen files, where the cost is small. This is a weight and clarity question, not a bug. Related: yaks-9756 (checkout weight).
