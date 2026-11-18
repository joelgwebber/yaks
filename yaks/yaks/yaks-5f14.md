---
id: yaks-5f14
title: 'source: add or replace extracted text for an existing source'
status: shorn
type: feature
priority: 3
created: '2026-11-08T22:04:33Z'
updated: '2026-11-18T00:54:23Z'
parent: yaks-4cfe
depends_on:
- yaks-e83f
tags:
- doks
---

Found by Hearth's calcifer import (hearth-f0cf). A source saved without text (a scanned PDF before OCR existed) cannot get its text later: saving the same bytes again reports the existing source, and `yaks source add --refresh` needs a `--url`. Hearth will OCR old scans after the fact, and any user may extract text from a source they saved earlier.

Want: a way to set or replace a source's `extracted.md` by id (for example `yaks source add --text FILE` on bytes that already exist updates the text and says so, or a `yaks source text <id> FILE`), one commit, `source.md` keeps its provenance and notes the new extraction. Not by editing files in the farm.

#### 2026-11-18T00:06:12Z

*sources-triage*

Unblocked by yaks-e83f (schema 10). The want is now simpler: set or replace a source's **body** (its text) by id, keeping its frontmatter. E.g. `yaks source text <id> FILE`, or `source add --text` on bytes already saved updating the body and saying so. Still to do.

#### 2026-11-18T00:48:47Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T00:54:23Z

*sources-triage*

Done (`sources-triage`): `yaks source set <id> --text <file|->`. It extends the existing `source set` (title and/or text, one write, one commit) rather than adding a new command.

- **What it does:** replaces the source's body and stamps `text_updated: {by, at}`. The frontmatter, `sha256` and any original kept beside the file are untouched, so `source add` of the same bytes still finds the source.
- **Refused:** no `--title` and no `--text`, and empty text.
- [`set`](code:src/sources.rs) and its caller in `src/main.rs`.
- **Docs:** `docs/cli.md`, `README.md`, the `yaks-research` skill ("Text added later"), and `--help`.

Evidence:

- **Test:** `set_text_gives_a_saved_source_its_text_later` covers a binary saved without text getting its text, the hash, title and original kept, a repeat being a no-op, dedup still finding it, title and text in one write, and the refusals.
- **Real run (release build, scratch farm):** a PNG saved, then `--text -` from stdin replaced the body and printed `Updated ... (text replaced)`. The same text from a file printed `No changes`, and `set` with neither flag exited 1. Each change is one farm commit.
- **Full gate:** 1480 passed, 0 failed; fmt clean.

#### 2026-11-18T00:54:23Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
