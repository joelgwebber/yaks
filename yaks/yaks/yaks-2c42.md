---
id: yaks-2c42
title: 'Sources: yaks source add (file, dir, --url, --part-of), sha256 dedup, yaks promote from attachments'
status: shorn
type: feature
priority: 2
created: '2026-11-01T18:53:11Z'
updated: '2026-11-04T20:03:33Z'
parent: yaks-f301
depends_on:
- yaks-9af0
tags:
- cli
- doks
verify: cargo test --workspace
---

Implements the sources half of yaks-aae6. Port of Hearth's local-file and directory snapshots (`internal/sources`), minus fetching and extraction. `yaks promote <id>/<file> --source|--page PATH` moves an attachment and rewrites the yak's link.

---
▸ 2026-11-01T21:19:21Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T21:26:40Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: 738326c
---

Done on the `wiki` branch.

## What it does

- `yaks source add <path> [--title] [--url] [--part-of]` snapshots a file or small directory into `sources/<date>-<sha8>/`: the original, `extracted.md` for text files, and a `source.md` in Hearth's format, so the `joel` repo's `raw/sources/` can move over as is. Content already saved (same sha256, any day) is reported, not copied, and a path inside the farm is refused. Nothing is fetched or converted (the decision on yaks-aae6); `--url` names the page a local copy came from.
- `yaks source list [--json]`.
- `yaks promote <id>/<file> [--page wiki/<path>.md]` moves an attachment into a source (resource `<id>/<file>`) or a page (frontmatter added if missing), removes it, and rewrites every link to it. That includes `](<id>/my shot.png)`: `yaks attach` writes file names unencoded, so CommonMark never parsed those as links. Worth fixing in `attach` itself; noted as a follow-up below.
- `Fields` now writes a list of maps indented under its key (`sources:\n  - id: file`), the way OKF and Hearth do; scalar lists stay flush, yaks' style.
- New dependency: `sha2` (pure Rust, no default features), for Hearth's sha256 id and dedup rule.

## Evidence

- Scratch farm: `source add paper.md --url ...` then again gives `Saved` then `Already saved`.
- Promoting `finding.md` to `wiki/topics/finding.md` rewrote the attach link and the note's `#draft-finding` link.
- Promoting `Shot (1).png` saved `sources/.../Shot-1.png`, rewrote `![Shot (1)](t-…/Shot (1).png)` to the source, and removed the empty attachment dir. Doctor is clean.
- `tests/sources.rs` (2 tests through the binary) and `sources::tests` (file, directory, names). `cargo test --workspace`: 905 passed.

## Follow-up

`yaks attach` should percent-encode spaces in the link it writes (or wrap the destination in `<...>`), so attachments with spaces render as links.

---
▸ 2026-11-01T21:28:37Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T21:28:37Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
