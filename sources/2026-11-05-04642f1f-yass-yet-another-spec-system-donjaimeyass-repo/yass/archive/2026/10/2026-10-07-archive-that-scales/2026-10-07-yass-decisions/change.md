---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-into-months
created: 2026-10-07T17:34:55Z
---
# yass decisions

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass decisions lists dated decisions across active and archived changes, filtered and bounded, as text or JSON, and yass-log and yass-shape use it instead of reading archive/.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC19–AC28, AC30 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `decisions.go`: entries from every change's and piece's `## Decisions`, active and archived, every yass folder; who and date from the last parenthesis; filters, newest first, `--limit` with a count of what's left out; `--json`; read warnings to stderr
- [x] `--change` follows `follows:` both ways and takes in pieces; a piece alone brings what its change is linked to
- [x] Change templates, the `AGENTS.md` section and `yass-work` ask for `(<who>, <YYYY-MM-DD>)`; `README.md` shows it and lists the command
- [x] `yass-log` and `yass-shape` (and `docs/skills.md`) look decisions up with `yass decisions`
- [x] go test (`TestParseEntry`), e2e section 32 (two yass folders, a piece, an archived change, a follows chain, JSON, templates)
- [-] AC29: moved to `decisions-from-history` with a `--cites` search
- [x] AC30: in a scratch copy upgraded with a local 0.4.0 build, following the refreshed `yass-log`: `yass decisions --about "existing hooks"` finds the decisions, and `git log --follow` across the migration names #16

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Decisions are newest first by their own date, else their change's archived date, else its created date; ties keep file order - "why did we…" is usually about recent calls, and an undated entry is placed as well as the record allows (claude, 2026-10-07)
- `yass decisions` prints only warnings about files it couldn't read, to stderr - its stdout may be JSON, and the repo's other warnings belong to `yass status` (claude, 2026-10-07)
- The AC29 step is dropped here: Jaime chose a `--cites` search, built with reading evicted months in `decisions-from-history`, and AC29 moved there (claude, 2026-10-07)
- Text lines have the entry's own date first (blank when it has none), then the change, the decision, who, and the change's dates in brackets - one line per entry, as AC19 asks, with the date where scanning eyes look first (claude, 2026-10-07)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-07 (claude)
- Did: built and tested `yass decisions`, dated entries in the templates and playbooks, and switched `yass-log` and `yass-shape` to it. `go test`, `tests/e2e.sh` (450 ok), `tests/examples.sh` pass.
- Open (AC29): `yass-log` still finds a commit's decision by grepping every yass folder for `code:.*<sha>`, archive included. It's a needle search, so its output stays small, but it does walk `archive/`. Options: accept it and narrow AC29 to decision lookups (a plan revision), or add a citation search to the CLI (e.g. `yass decisions --cites <sha>`, searching boxes too). Jaime to choose.
- Next: Jaime's call on AC29; AC30 once the installed playbooks are refreshed (release, or `yass upgrade` with a local build).
### 2026-10-07 (claude)
- Did: a release dry run with local builds. A repo set up and archived with v0.3.0, upgraded with 0.4.0: files restamped, its flat archive migrated, both committed without hook warnings, a new archive filed by month, `yass decisions` reads it. A teammate still on 0.3.0 gets the "upgrade your binary" note and archives flat, which the next `yass upgrade` moves. A scratch copy of this repo upgraded cleanly too (12 files, nothing left to migrate), which gave AC30.
- Next: AC29, which Jaime is deciding (a `--cites` search in `decisions-from-history`, or narrowing AC29).
### 2026-10-07 (claude)
- Did: Jaime chose `--cites` for AC29; the plan revision moves it to `decisions-from-history`. Everything this piece delivers is done.
- Next: none here.
