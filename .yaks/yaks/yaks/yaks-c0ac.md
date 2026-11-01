---
id: yaks-c0ac
title: 'Checks: stale pages, uncited sources, broken links, conflict markers (yaks check)'
status: shaving
type: feature
priority: 2
created: '2026-11-01T18:53:13Z'
updated: '2026-11-01T22:26:18Z'
parent: yaks-d26a
depends_on:
- yaks-101d
- yaks-2c42
labels:
- wiki
- cli
---

Hearth's nightly check minus what needs a model: `stale_after` past, sources no page or yak cites, links that lead nowhere, kinds of page that look alike.

---
▸ 2026-11-01T22:22:23Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:25:17Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `check-1`, on branch `wiki`, base a150387.

**Task.** `yaks check [--json]`: Hearth's nightly check minus anything that needs a model. It reports for a person or an agent to act on, and never edits. Exit 0 when clean, 1 when there are findings. Findings by kind:

- `stale`: a page past its `stale_after`. Use the page's own `stale_after`, else its schema's (`schemas::Schema::stale_after`, landed in a150387). The value is an ISO 8601 duration (`P1Y`, `P6M`, `P2W`, `P30D`, combinations) or a date (`2026-12-01`, stale once that day has passed). Count from the page's `verified:` date, else from the page file's last commit on the farm branch (`git log -1 --format=%cI -- <file>` in the farm worktree; skip the page if there is no git or no commit). Message like Hearth's: `wiki/x.md was verified 2026-01-02; its type asks for a re-check by 2026-07-02`.
- `uncited`: a source under `sources/<id>/` that no yak or page cites. A citation is a link or mention of `sources/<id>/` anywhere in a yak body or page (body or frontmatter `sources:`). A source and its `part_of` children count as one. A source whose `source.md` has `superseded_by:` or `uncited:` counts as cited. Port Hearth's `uncitedSources` (hearth repo `internal/check/scan.go`).
- `similar-kinds`: page kinds (types in use, plus schema names) that differ only by plural or hyphens. Port Hearth's `similarTypes` and `normalizeType`.
- `broken-link`, `schema-mismatch`, `merge-conflict`, `page-problem`: reuse `Farm::doctor` issues of those kinds as they are. Don't re-implement them.

**Scope.**

- New `src/check.rs`. Add a `Check` variant plus its dispatch arm in `src/main.rs` (only that region).
- Docs: `docs/cli.md` (command table plus a `### check` section), the `yaks` skill's command table, and `README.md` if its command table fits.
- Tests: `tests/check.rs` through the binary on a scratch farm.

**Out of scope.** `doctor` itself, schemas, sources and pages code (read their pub APIs). Ask before touching any other file.

**Evidence.** `tests/check.rs` fails before the change and passes after; the full gate passes. Paste a real `yaks check` run on a scratch farm with one finding of each kind into a note.

**Judge.** The coordinator, at landing.

---
▸ 2026-11-01T22:26:18Z [check-1]

Start Sun Nov  1 22:26:17 UTC 2026. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-c0ac.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/rc7whrpk3kdg/yaks
