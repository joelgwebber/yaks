---
id: yaks-4cfe
title: 'Sources: one coherent layout for text, provenance, directories and browsing'
status: hairy
type: task
priority: 2
created: '2026-11-17T21:15:46Z'
updated: '2026-11-17T21:15:46Z'
tags:
- doks
---

Umbrella for the source-layout problems Joel hit in this farm (2026-11-17). They look separate but all come from one gap: **a source has no single answer to "which file is the source"**. Text, provenance and the original are spread across `source.md`, `extracted.md` and the original (or `files/`), and each reader (a dok link, the TUI, `find`, GitHub) picks a different one.

## What is wrong today (measured in this farm)

- **Identical text twice.** All 14 sources saved on 2026-11-16 hold a markdown original *and* a byte-identical `extracted.md` (`cmp` says so for every one). `--text-only` (yaks-0df0) avoids it, but only when asked for, and the decision to collapse existing pairs was left open there.
- **No way back from the text.** Doks link `/sources/<id>/extracted.md#...`. That file has no frontmatter and no link to `source.md`, so the URL, title and retrieval date can't be reached from the text, on GitHub or in the TUI.
- **Directory sources store everything twice and break their own links.** YASS (`sources/2026-11-05-04642f1f-...`) is 170 files under `files/` plus an 842 KiB joined `extracted.md`. Relative links inside the joined copy (`[docs/monorepo.md](docs/monorepo.md)`) were written for `files/README.md`, so from the source root they lead nowhere.
- **`source.md` lists every file.** About 250 frontmatter lines for YASS, repeating what the `files/` tree already says.
- **The TUI is slow on that source**, at about 150 ms per `j`. The cause is the 842 KiB text rendered under `source.md`, not the file list (measured; see the perf child).
- **No way to browse a set of files**: a source's `files/`, a yak's attachment directory. A yak in another farm attached a `.tgz` to get a directory in, and system-open unpacked it into the farm.

## Children

The design questions are asked on the children, with leans. Settle those first, since the bugs below them may simply disappear.
