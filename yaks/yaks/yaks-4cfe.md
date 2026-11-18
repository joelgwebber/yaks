---
id: yaks-4cfe
title: 'Sources: one coherent layout for text, provenance, directories and browsing'
status: shorn
type: task
priority: 2
created: '2026-11-17T21:15:46Z'
updated: '2026-11-18T00:54:25Z'
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

#### 2026-11-17T21:42:25Z

*sources-triage · kind: move · from: hairy · to: shaving · child: yaks-3da7 · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:43:02Z

*sources-triage*

Batch 1 (yakherd `sources-triage`, 2026-11-17). Base `52d80dd3` (v0.0.37). Model `claude-sonnet-5-5` for every worker (AGENTS.md's standing rule). The format work (yaks-e83f, then yaks-535c, yaks-289a, yaks-5f14) waits on [the layout ask](/yaks/yaks/yaks-e83f.md).

| Worker | Yaks | Files |
|---|---|---|
| `srccli-1` | yaks-3da7, yaks-4dd3, yaks-06aa, yaks-fb15 | `src/sources.rs`, `source` arms of `src/main.rs` |
| `attach-2` | yaks-a57c | `src/farm.rs` attach, `Attach` arm of `src/main.rs` |
| `srcperf-1` | yaks-760a | TUI text viewer render |
| `browse-1` | yaks-3329 | new TUI directory view |

Expected overlaps, landed in this order: `srccli-1` before `attach-2` (the shared walk), and `srcperf-1` before `browse-1` (handlers and render).

#### 2026-11-17T21:45:02Z

*sources-triage*

Batch 1 spawned (Delta subthreads, `claude-sonnet-5-5`, base `52d80dd3`):

- `srccli-1`: thread `ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzJXZIBxBCA0aLJyrhJPreUvVwLftHG`
- `attach-2`: thread `ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzKtpIBxBCA0aLJyrhJPreUvVwLftHG`
- `srcperf-1`: thread `ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzOWZIBxBCA0aLJyrhJPreUvVwLftHG`
- `browse-1`: thread `ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzP4JIBxBCA0aLJyrhJPreUvVwLftHG`

Held for the human: yaks-e83f's second ask (layout, schema 10). yaks-535c, yaks-289a and yaks-5f14 wait on it. yaks-93ee (archive browsing) is filed top-level for later.

#### 2026-11-18T00:06:32Z

*sources-triage*

Batch 2 (`sources-triage`): schema 10.

- I wrote the code, docs and migration myself.
- `s10tests-1` (`claude-sonnet-5-5`, base `14cf78d5`) updated the tests (yaks-03c8): it landed cleanly, and it found one real bug (prose citations at the end of a sentence), fixed in `06e0ebe9`.

Results so far, by model:

- **Sonnet workers:** 5 batches, 9 yaks. All landed. Two needed coordinator follow-ups that their tests had missed:
  - `attach-2`: the directory link didn't parse;
  - `srccli-1` ↔ `attach-2`: a merge over the shared walk.
- **Opus (coordinator):** design, landings, schema 10.

Left under this umbrella: yaks-5f14 (set a source's text later), now simpler and unblocked, but not part of this pass. The real farm's `yaks migrate` waits for Joel.

#### 2026-11-18T00:54:24Z

*sources-triage*

Every child is shorn (`sources-triage`): yaks-5f14 closed with `source set --text`. Joel migrated his farms to schema 10 without a hitch. Closing the umbrella.

#### 2026-11-18T00:54:25Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
