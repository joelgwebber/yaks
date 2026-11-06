---
id: yaks-0df0
title: 'source add --text-only: save a markdown file as its extracted text alone'
status: hairy
type: feature
priority: 2
created: '2026-11-06T15:53:03Z'
updated: '2026-11-06T15:53:03Z'
tags:
- doks
- cli
---

`source add` always writes the original and `extracted.md`; for a markdown or plain-text original those are identical files (git stores one blob, but the checkout shows two and a reader wonders which counts). A caller that already holds the markdown it wants read (a Google Doc export, joel-5cee) has no original worth keeping.

Opt-in `--text-only` on a single file: save just `extracted.md`, and record `file: extracted.md` so `file:` stays true. The `sha256` is of those bytes.

- Refused for a directory (its layout under `files/` is what keeps relative links working), for a file that is not UTF-8 text, and together with `--text`.
- Works with `--refresh`, and `previous_sha256` still records the hash it replaced.
- Without it nothing changes. Existing sources keep both files; collapsing those is a separate format decision (not made here).

Skills own the "when": `yaks-research` says when to use it, `gdoc-sources` always does.
