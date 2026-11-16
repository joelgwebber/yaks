---
id: yaks-5f14
title: 'source: add or replace extracted text for an existing source'
status: hairy
type: feature
priority: 3
created: '2026-11-08T22:04:33Z'
updated: '2026-11-08T22:04:33Z'
---

Found by Hearth's calcifer import (hearth-f0cf). A source saved without text (a scanned PDF before OCR existed) cannot get its text later: saving the same bytes again reports the existing source, and `yaks source add --refresh` needs a `--url`. Hearth will OCR old scans after the fact, and any user may extract text from a source they saved earlier.

Want: a way to set or replace a source's `extracted.md` by id (for example `yaks source add --text FILE` on bytes that already exist updates the text and says so, or a `yaks source text <id> FILE`), one commit, `source.md` keeps its provenance and notes the new extraction. Not by editing files in the farm.
