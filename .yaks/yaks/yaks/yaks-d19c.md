---
id: yaks-d19c
title: 'source add: --text for converted text; no temp paths in provenance; HTML is not text'
status: shorn
type: task
priority: 3
created: '2026-11-04T17:23:10Z'
updated: '2026-11-04T17:29:21Z'
parent: yaks-dedc
tags:
- wiki
- cli
---

The tool half of yaks-dedc (diagnosis in its notes). In `src/sources.rs` (`add`, `is_text_name`, `Request`) and the `source add` arm of `src/main.rs`.

## Changes

- **`--text <file>`** on `source add`: the saver's own conversion of the original (HTML to markdown, PDF to text). It becomes the source's `extracted.md`; the original is saved beside it as today; `sha256` stays the original's. UTF-8 only; refused with a directory. This is what replaces "save the conversion as a second source with `--part-of`".
- **`html` is no longer treated as text**: it gets no `extracted.md` of its own. Its `source.md` note says to convert it and pass `--text`. (A directory's joined text drops `.html` files too: same rule.)
- **Provenance without temp paths:**
  - `description` is built from the title, the `--url` and the file's *name*, never an absolute path: `Saved copy of <url>` / `Snapshot of <file name>` / `Snapshot of the directory <name> (N files)`.
  - With `--url`, no `local_copy` entry.
  - Without `--url`, `resource` is the absolute path unless the file sits under a temp directory (`std::env::temp_dir()`, `/tmp`, `/var/tmp`, canonicalized), where it is the file name: a scratch path says nothing about where the thing came from.
  - `promote` keeps its `Promoted from the yak attachment <id>/<file>` description.
- Help text for `source add` / `--text` / `--url`, `docs/cli.md` (`source` section) and a line in the changelog if the repo keeps one.

## Evidence

- Unit tests in `src/sources.rs` that fail first: `--text` lands as `extracted.md` and is refused for a directory and for non-UTF-8; an `.html` original gets no `extracted.md`; no description or entry contains the temp path; a path outside temp still records its path.
- Run my own binary on the real scenario: the Confluence HTML plus a converted `.md`, saved once with `--url --title --text`; show the resulting `source.md` and listing.
- `cargo test`, `cargo fmt -- <my files>`, `cargo clippy` clean.

---
▸ 2026-11-04T17:23:26Z [source-fix]

kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T17:29:20Z [source-fix]

Done on branch `yaks-dedc-sources` (1720ba8), not merged to `main`.

- `source add --text <file>`: the saver's conversion becomes `extracted.md`; refused for a directory, a missing file or non-UTF-8.
- `html` is no longer "text", so an HTML original gets no `extracted.md`; its `source.md` says to convert and pass `--text`.
- Provenance has no scratch paths: `description` is `Saved copy of <url>` / `Snapshot of <file name>`; `local_copy` is gone; a file from a temp dir records its name as `resource`.

Evidence: four new unit tests in `src/sources.rs` (text lands, html alone, refusals, scratch vs real path); `cargo test` all green; my own binary on the Confluence case (one `source add page.html --url --title --text page.md`) gave a single source with the URL as resource and the markdown as `extracted.md`.

Not done: `promote` still records `Promoted from the yak attachment <id>/<file>` (a farm path, not a scratch path, so left alone).

---
▸ 2026-11-04T17:29:21Z [source-fix]

kind: move
from: shaving
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
