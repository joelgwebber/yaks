---
id: yaks-c0ac
title: 'Checks: stale pages, uncited sources, broken links, conflict markers (yaks check)'
status: shorn
type: feature
priority: 2
created: '2026-11-01T18:53:13Z'
updated: '2026-11-01T22:51:31Z'
parent: yaks-d26a
depends_on:
- yaks-101d
- yaks-2c42
labels:
- wiki
- cli
verify: cargo test --workspace
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

---
▸ 2026-11-01T22:34:38Z [check-1]

Progress: src/check.rs written (stale, uncited, similar-kinds + doctor's four kinds reused), Check variant + dispatch added in main.rs, cargo build OK. Next: tests/check.rs, docs, scratch run.

---
▸ 2026-11-01T22:39:38Z [check-1]

Evidence: real run on a scratch farm (git init + ./target/debug/yaks init --no-skills, XDG_DATA_HOME in a temp dir), one finding of each kind, exit=1:

Found 7 findings:
Page problem: wiki/clash.md: no title:
Broken link: wiki/soup.md:L1: (/wiki/missing.md) no such file in the farm
Merge conflict: wiki/clash.md: conflict markers at L5
Stale page: wiki/p2.md was due to be checked by 2020-01-01
Stale page: wiki/soup.md was verified 2020-03-01; its type asks for a re-check by 2020-09-01
Uncited source: sources/2026-11-01-4cd893a3 is cited by no page or yak
Similar kinds: these look like the same kind of page: `project` (1 page), `projects` (1 page)

(--json gives [{kind, message, ids}], same shape as doctor --json.) tests/check.rs: 4 tests, fail before (no `check` command), pass after. cargo test --workspace: 27 "test result" lines, 933 passed, 0 failed, no error/FAILED lines.

Decision: findings use doctor's {kind, message, ids} shape and exit-1 rule, with its four kinds reused via Farm::doctor(false) (filtered to those kinds) rather than a new JSON shape or re-scanning. Alternative: a Hearth-like grouped object {stale: [...], ...}; rejected because agents already parse doctor's array.
Decision: no `Finding` for yak-only doctor kinds (invalid-status, dangling parent...): check is about pages and sources, per the spec's list. Alternative: pass all doctor issues through.
Decision: stale from a duration needs `verified:` or a farm-branch commit; with neither, or with an unreadable value, the page is skipped silently (spec). A `verified:` that is an RFC 3339 timestamp is also accepted (date part). Durations must be in Y, M, W, D order (as Hearth's regex); months are added with chrono Months (clamps day-of-month), days then added.
Decision: date `stale_after` means stale once the day has passed (today > date), as Hearth's now > date+1day, compared by calendar day in UTC.
Decision: uncited citations are read from the whole text of every page and yak (frontmatter, body, notes), found by scanning for `sources/<id>/` at a word edge instead of a regex (no regex dependency; Cargo.toml is off limits). The `resources/x/` look-alike is rejected. Source files under sources/ are not read, so a source citing itself does not count. Hearth also skipped dot dirs and its raw/; here only pages and yaks are read, so sources/ and schemas/ docs do not cite.
Decision: the similar-kinds scan excludes schemas/yak.md (it extends yaks, not a page kind); normalization also lowercases, as Hearth. Alternative: include it.
Decision: `uncited` ids are `sources/<id>` (the directory), `similar-kinds` ids are the kind names, `stale` ids the page path.
Decision: `check` prints no skills advisory (doctor's render_skill_advisory is environment, not farm state).

---
▸ 2026-11-01T22:42:28Z [check-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T22:42:32Z [check-1]

Summary: yaks check landed in src/check.rs (+Check in main.rs), tests/check.rs, docs/cli.md, README.md, yaks skill table. verify PASS (cargo test --workspace).

---
▸ 2026-11-01T22:42:33Z [check-1]
kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD0oa5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-01T22:43:03Z [delta:farm-wiki]
kind: landed
commit: 206f23dda065c2a7e19a5cf0fb25dcac76cd6bad
source: 47954199f5a8d471abb24888a1902ab56d589340
branch: wiki
---

landed: 206f23d yaks-c0ac: yaks check (stale pages, uncited sources, similar kinds, doctor's page findings)

---
▸ 2026-11-01T22:51:31Z [delta:farm-wiki]

Landed as 206f23d (yakherd). Re-ran the gate: 941 passed with yaks-f0db landed too. Reproduced on a fresh scratch farm: one page with a past `stale_after` date and a broken link, a `project`/`projects` pair, and an uncited source gave all four findings, exit 1.
