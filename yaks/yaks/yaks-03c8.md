---
id: yaks-03c8
title: 'Schema 10: update existing tests to the one-file source layout'
status: hairy
type: task
priority: 2
created: '2026-11-17T23:31:25Z'
updated: '2026-11-17T23:31:25Z'
parent: yaks-e83f
tags:
- doks
- cli
---

Part of yaks-e83f. The schema-10 code is on `main` in the commit marked "wip": a source is `sources/<name>.md` (frontmatter is the provenance, body is the text), with its original or a directory's tree in the sibling `sources/<name>/`. `cargo test -p yaks` has 36 failures. All of them are existing tests that build or assert the old layout (`source.md`, `extracted.md`, `files/`, the `files:` list), plus the schema-gate test (now 10).

Update each test to the new layout. Keep what it was testing, not its old assertions. Where a test's purpose no longer exists (the joined directory text), replace it with the new behaviour's test:

- a directory's tree sits directly in the sibling directory, with `file_count:` and no `files:`;
- `--text` on a directory becomes the body;
- a JSON or code original is kept, with a fenced body.

Snapshots: review each `.snap.new` and accept it only when the change is the layout's.

Do **not** change non-test code. If a test shows a real bug in the new code, `yaks ask` with the failing case.
