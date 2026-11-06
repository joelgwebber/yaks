---
id: yaks-0df0
title: 'source add --text-only: save a markdown file as its extracted text alone'
status: shaving
type: feature
priority: 2
created: '2026-11-06T15:53:03Z'
updated: '2026-11-06T16:03:26Z'
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

---
▸ 2026-11-06T15:53:08Z [source-refresh]
kind: move
from: hairy
to: shaving
thread: ksQQcsYXd-T6RPSH25f_4VJRspLOAETlkJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T15:53:09Z [source-refresh]

Plan (test first, in [sources.rs](code:src/sources.rs)):

- `Request.text_only: bool`; in `add`, a `text_only` file is written once, as `extracted.md`, and `source.md` gets `file: extracted.md`.
- Errors, all before anything is written: a directory, non-UTF-8 content, `--text` given too, and a path whose type isn't text (an HTML original that has no text of its own, an image).
- Refresh needs no new logic: it already clears everything but `source.md`; the test proves it.
- Same change: `--help`, `docs/cli.md`, the `yaks` and `yaks-research` skills; then `gdoc_source.py` passes it (joel-5cee).

---
▸ 2026-11-06T16:03:26Z [source-refresh]

Done and tested.

- [add](code:src/sources.rs): `Request.text_only`; the saved file is `extracted.md` alone, `file: extracted.md`, hash of those bytes. Refused for a directory, a non-text or non-UTF-8 file, and with `--text`, before anything is written.
- A refresh whose bytes are the same but whose file layout changed is still a refresh (`same_file`): that is how an existing two-file source collapses. `previous_sha256` is only written when the hash actually changed.
- Help, `docs/cli.md` and `yaks-research` updated in the same change.

Evidence: four new unit tests (text-only saves one file, the refusals, text-only refresh, collapsing an existing two-file source), a scripted run of the built binary, and the full `cargo test` green (988 unit tests plus integration suites). The four saved Google Docs collapsed to `extracted.md` + `source.md` on refresh.
