---
id: yaks-535c
title: Directory sources store their text twice (files/ plus a joined extracted.md)
status: shaving
type: idea
priority: 4
created: '2026-11-07T15:58:16Z'
updated: '2026-11-17T22:54:06Z'
parent: yaks-4cfe
depends_on:
- yaks-e83f
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

#### 2026-11-17T21:17:08Z

*sources-triage · kind: ask · needs: human*

asked: needs human

Two related calls for directory sources:

1. **Drop the joined `extracted.md`?** Lean: yes. `find` and the Doks filter walk `files/` for text files instead. The source view shows `source.md` and then the file browser (yaks-0229). Agents read the file they need, which they already do from disk. It is a few lines in `find.rs` and `doks_tab.rs`, and it removes yaks-289a and most of yaks-760a.
2. **Drop the `files:` list from `source.md`?** Lean: yes. Keep `file_count:` and `size:`. The `files/` tree is the record, and `sha256` already covers it. A skipped-files line (yaks-3da7) still goes in the body.

Existing sources would be migrated in the same pass as yaks-e83f.

#### 2026-11-17T21:33:26Z

*sched-1 · kind: answer · needs: agent*

1. Agreed, as mentioned in yaks-e83f as well.
2. Yeah, I think that would be very helpfully simplifying.

#### 2026-11-17T21:37:51Z

*sources-triage · kind: pickup*

picked up

Decision recorded: drop the joined extracted.md and the files: list (Joel's answer). Folded into the same format change as yaks-e83f; done with it.

#### 2026-11-17T22:54:06Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
