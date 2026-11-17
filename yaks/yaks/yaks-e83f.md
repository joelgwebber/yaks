---
id: yaks-e83f
title: 'Design: one file holds a source''s text and its provenance (retire the duplicate extracted.md)'
status: shaving
type: idea
priority: 2
created: '2026-11-17T21:16:47Z'
updated: '2026-11-17T22:54:05Z'
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
