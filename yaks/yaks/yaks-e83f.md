---
id: yaks-e83f
title: 'Design: one file holds a source''s text and its provenance (retire the duplicate extracted.md)'
status: shorn
type: idea
priority: 2
created: '2026-11-17T21:16:47Z'
updated: '2026-11-18T00:06:15Z'
parent: yaks-4cfe
tags:
- doks
- design
---

Part of yaks-4cfe. A source keeps provenance in `source.md` and its text in `extracted.md`, and for a markdown or text original it also keeps the original, byte-identical to `extracted.md` (all 14 sources saved here on 2026-11-16). Doks cite `/sources/<id>/extracted.md#heading`, which has no frontmatter and no link back, so a reader who follows a citation can't see where the text came from.

## Why it is this way

- yaks-dedc's diagnosis kept the duplicate on purpose: search and the TUI read `extracted.md`, and the two-file layout is Hearth's format, so a Hearth `raw/sources/` migrates as is.
- yaks-0df0 added `--text-only` (one file, `file: extracted.md`) but left it opt-in, and explicitly left "collapse existing pairs" as a separate format decision. Agents mostly don't pass it.

## Options

- **A. Keep the layout and patch it.** Make `--text-only` the default for a markdown or text original. Prepend a one-line back-link to every `extracted.md`: `> From [<title>](source.md) · <url>`. Cost: `extracted.md` is no longer exactly the text. A text-only source's `sha256` would have to skip the header. Every reader of the text gets a line it didn't write.
- **B. `source.md` is the source.** Its frontmatter is the provenance and its body is the text. `extracted.md` is retired. The original sits beside it only when it is not the text: HTML, PDF, JSON, an image. Citations become `/sources/<id>#heading`, and `/sources/<id>` already resolves to `source.md`. One page shows the provenance and the text, in the TUI and on GitHub. Cost: a format change. It needs a migration that merges each pair and rewrites `.../extracted.md` links (the `yaks mv` link rewrite exists), plus a reader for the old layout and Hearth's. `sha256` stays the hash of the original, or of the text for a text-only source.
- **C. A TUI-only fix.** Opening an `extracted.md` shows its `source.md` header above it. Nothing changes on disk, and GitHub still has no way back.

**Lean: B.** It leaves one file to cite, read, search and diff, and the back-link problem goes away instead of being patched. A keeps two files that can drift and a header that isn't part of the text. C fixes only one of the three readers.

## Done when

- A new source writes one text-bearing file. Existing sources are migrated, and their links follow.
- `find`, the TUI viewer, `peek`, `check`'s citation count and the `yaks-research` skill all read the new layout.
- Docs, `--help` and skills say it in the same change.

#### 2026-11-17T21:17:09Z

*sources-triage · kind: ask · needs: human*

asked: needs human

Which layout for a source's text (options in the description)?

- **A.** Keep `extracted.md`, make `--text-only` the default and add a back-link header.
- **B.** `source.md` holds the provenance (frontmatter) and the text (body). `extracted.md` is retired, and a migration rewrites links.
- **C.** A TUI-only header.

Lean: **B.** One file to cite, and the way back is the frontmatter itself. If B is too big a format change right now, A's back-link header alone is a cheap stopgap. The answer settles yaks-535c too.

#### 2026-11-17T21:31:57Z

*sched-1 · kind: answer · needs: agent*

I'm very good with (B) -- one file is better most of the time, unless it's completely out of hand in size. We can still use a directory of attachments for fetched sources or whatnot, when necessary. I don't recall whether we support the same attachment directory pattern for doks/sources as we do for yaks, but perhaps that would be a nice simplification to use a single pattern throughout?

#### 2026-11-17T21:42:04Z

*sources-triage · kind: pickup*

picked up

Decision: option B (Joel). One file holds a source's provenance (frontmatter) and its text (body); extracted.md is retired. Follow-up layout question asked below before any format work.

#### 2026-11-17T21:42:04Z

*sources-triage · kind: ask · needs: human*

asked: needs human

Following your answer (B, plus "a single attachment pattern throughout"). Four calls before I write the format change. Each one has a lean.

1. **Layout: a source becomes a file plus a sibling directory, as yaks and doks already are.** `sources/<id>-<slug>.md` holds the provenance in its frontmatter and the text in its body. `sources/<id>-<slug>/` holds the original only when there is one worth keeping (a PDF, the fxtwitter JSON, screenshots). A markdown or text source is the one file. Links: `/sources/<id>` resolves to the `.md` as it does today. The migration rewrites `/sources/<id>/source.md` and `.../extracted.md` (28 doks and yaks here link `extracted.md`, and 84 links name `source.md`), and the resolver keeps accepting the old forms for text written elsewhere. Lean: yes. Doks and yaks already work this way, so it is one pattern everywhere, and the attach and browse work (yaks-a57c, yaks-3329) covers sources for free.
2. **Directory sources:** `sources/<id>-<slug>.md` plus the tree directly in `sources/<id>-<slug>/`, with no `files/` level, no joined text (yaks-535c), and no `files:` list. A relative link inside the tree is then simply right, which closes yaks-289a. The body is empty or the saver's own `--text` summary. `find` and the Doks filter walk the tree. Lean: yes.
3. **Size:** "one file unless it's out of hand." I'd rather not have a threshold that silently splits a source two ways. Lean: the text is always the body. The real cost of a big body is TUI rendering, which yaks-760a fixes for any big document. The 20 MiB per-file cap stays the guard.
4. **Schema 10.** This is an on-disk format change, so it means a schema bump and a `yaks migrate` step, which you run when you choose (schema 9 only just shipped in 0.0.37). Lean: one schema-10 step for 1 and 2. Until it is released, new sources keep the current layout, so no farm ends up half and half.

#### 2026-11-17T22:52:12Z

*sched-1 · kind: answer · needs: agent*

1. That SGTM. It might be mildly annoying to have a semi-permanent source.md redirect as a special case, but as long as we document / test it clearly in the code, it shouldn't get lost by accident.
2. Yup, makes sense to me.
3. Makes sense. Anything truly huge is likely to end up in an attachment anyway.
4. Oh yeah, even better (scratch my comment about a semi-permanent redirect).

#### 2026-11-17T22:54:05Z

*sources-triage · kind: pickup*

picked up

Decision (Joel's answers to the layout ask, recorded by `sources-triage`):

- **Layout:** a source is `sources/<id>-<slug>.md`. Its frontmatter is the provenance and its body is the text. The sibling `sources/<id>-<slug>/` holds the original only when one is worth keeping.
- **Directory sources:** the tree goes directly in the sibling directory. No `files/` level, no joined text, no `files:` list.
- **Size:** the text is always the body. A truly huge thing goes in as an attachment.
- **Schema 10:** one migration step does both, and old link forms are rewritten by it. No permanent `source.md` redirect in the resolver once the farm is migrated. Joel: "scratch my comment about a semi-permanent redirect".

Taking it on myself (design-heavy, many files). yaks-535c, yaks-289a and yaks-5f14 fold into it or follow it.

#### 2026-11-17T22:54:05Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T23:02:45Z

*sources-triage*

Plan (`sources-triage`, schema 10). This is the contract I'm implementing; review the code against it.

## On disk

- **The source** is `sources/<id>-<slug>.md`. The frontmatter is today's provenance (`title`, `description`, `media_type`, `size`, `sha256`, `previous_sha256`, `sources:`, `generated:`, `part_of`, the marks). The body is the text.
- **Sibling directory** `sources/<id>-<slug>/`, only when there is something to keep:
  - a markdown or plain-text original is the body and is **not** kept again;
  - with `--text`, the original is kept and the conversion is the body;
  - another text type (JSON, CSV, code) is kept, and the body is a fenced copy of it;
  - a binary with no text is kept, and the body says to save it again with `--text`.
- **`file:`** names the kept original in the sibling (absent when there is none).
- **A directory source** has its tree directly in the sibling, with no `files/` level. Its frontmatter gets `file_count:` and no `files:` list. Its body is `--text` when given (now allowed for a directory), else one link to the tree, `[<name>/](<name>/)`. What the walk skipped goes in a `skipped:` list in the frontmatter, so the body stays only text.
- **`--text-only`** keeps its meaning: never keep the original. It is the default for markdown and text now, and still useful for a JSON or code file whose text is all you want.

## Addressing

- `/sources/<id>` (bare or slugged) is `sources/<id>-<slug>.md`.
- `/sources/<id>/<path>` is a file in the sibling directory.
- A farm link may end in `/` to name a directory (`[x/](x/)`).
- After the migration no `source.md` or `extracted.md` alias is left in the resolver (Joel).

## Migration (schema 9 → 10, in the same `yaks migrate`)

Per `sources/<dir>/source.md`:

- Write `sources/<dir>.md`: fields carried over, `files:` becomes `file_count:`, and `part_of` / `superseded_by` are rewritten. The body is `extracted.md`, or the directory link, or the no-text note.
- In the directory: remove `source.md` and `extracted.md`; remove a markdown or text original identical to the text; lift `files/*` up one level; remove the directory if nothing is left.
- Across every yak, attachment document, dok, session and source, rewrite:
  - `sources/<X>/source.md` → `sources/<name>.md`
  - `sources/<X>/extracted.md#a` → `sources/<name>.md#a`, or for a directory source, when the anchor is a joined file's heading, → `sources/<name>/<that path>`
  - `sources/<X>/files/<p>` → `sources/<name>/<p>`

  Frontmatter and prose included, only for sources that exist.

## Readers

`sources::list`, dedup, `check`'s citations, `find` (the `.md` plus the sibling's text files), the Doks tab, the TUI viewer (a source is a plain markdown document; the extracted section goes), `peek`, `promote`, and the CLI messages.

Also docs, `--help`, the `yaks` and `yaks-research` skills, and `/doks/design/knowledge-store.md`.

#### 2026-11-17T23:31:14Z

*sources-triage*

Checkpoint (`sources-triage`): the schema-10 code is in, on `main` as a commit marked wip (not installed or released). It contains:

- the writer and readers in `sources.rs`;
- `locate`, `find`, `check`, `sessions`, `promote` and the TUI reading the new layout;
- `src/migrate10.rs` (convert plus reference rewrite), with 3 passing tests;
- `SCHEMA = 10`, wired into `yaks migrate`.

The full gate shows 36 failures, all of them existing tests that still build or expect the old layout (`source.md`, `extracted.md`, `files/`). A worker is updating them. Docs and skills, plus a dry run of the migration against a copy of this farm, are next.

#### 2026-11-17T23:34:02Z

*sources-triage*

Migration rehearsal (`sources-triage`): `yaks migrate` from a release build of the wip commit, run on a **copy** of this farm (the real farm only read; schema 9 → 10).

```text
sources: 28 source(s) made one file each; 2 directory tree(s) lifted out of `files/`; 17 duplicate copy(ies) of the text removed
source references: 69 edit(s) in 8 file(s)
```

- The Thorsten source is one file: provenance on top, the article's text right under it, and the fxtwitter JSON kept beside it (`file:`).
- The YASS source file is 18 lines (was 271, plus an 842 KiB joined copy). Its tree is directly in the sibling directory, so README's `[docs/monorepo.md](docs/monorepo.md)` now resolves. That closes yaks-289a by construction.
- `doctor`: only the two dependencies that were already dangling. `check`: one finding fewer than the real farm, with no new broken or uncited source.
- No `.../source.md` or `.../extracted.md` link is left in doks, yaks or sessions, apart from prose and code spans that describe the old layout (historical text, correctly untouched).
- `find monorepo --kind sources` finds hits in the YASS tree's own files.
- TUI: the source shows `Files: 168 files`, and Enter opens the tree in the directory view. Holding `j` on it is now trivial (0.15 s for 30 keys).

#### 2026-11-17T23:36:05Z

*sources-triage*

Docs, help and skills for the new layout committed on `main` (`docs/cli.md` incl. the schema-10 migrate section, `docs/tui.md`, `README.md`, the `yaks` and `yaks-research` skills, clap help). `cargo test -p yaks skills` green. Waiting on `s10tests-1` (yaks-03c8) for the test updates, then full gate and the design dok.

#### 2026-11-18T00:06:14Z

*sources-triage*

Done (`sources-triage`). Schema 10 is on `main`:

- `14cf78d5`: code (the "wip" label is only the commit's message; it is complete);
- `7d393e3a`: docs, help and skills;
- `1091a53e`: tests, by `s10tests-1` (yaks-03c8);
- `06e0ebe9`: a citation at the end of a sentence counts, a bug yaks-03c8 found.

Evidence:

- **Full gate:** `cargo test -p yaks --no-fail-fast` gives 1479 passed, 0 failed, and fmt is clean.
- **Migration rehearsal, final binary, fresh copy of this farm:** 28 sources made one file each, 2 trees moved up, 17 duplicate copies removed, 69 references rewritten in 8 files. `doctor` shows only the two dependencies that were already dangling. `check` has 36 findings, one fewer than the real farm. A second run says "nothing to migrate: the farm is on schema 10".
- **Design dok:** `/doks/design/knowledge-store.md` updated.

**Not done, and a human's call:** running `yaks migrate` on this farm. Once synced, every machine on an older yaks is refused it. Order, per the coordinating skill:

- set `min_version:` to the release that carries schema 10, then sync;
- upgrade every machine;
- run `yaks migrate --dry-run`, then `yaks migrate`;
- sync;
- restart every running `yaks tui`.

#### 2026-11-18T00:06:15Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
