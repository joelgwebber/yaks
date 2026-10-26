---
id: yaks-ef3b
title: 'Run-on filter: yaks whose markdown has lines that join into one paragraph'
status: shorn
type: feature
priority: 2
created: '2026-10-26T23:07:10Z'
updated: '2026-10-26T23:19:10Z'
parent: yaks-db07
labels:
- cli
- markdown
---

---
▸ 2026-10-26T23:07:33Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T23:07:33Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `runon-1`. Plan: the latest note on yaks-db07.

**Goal:** a shared filter flag `--run-on` that keeps yaks whose description or any note contains a paragraph with a soft line break, i.e. adjacent lines that CommonMark joins into one paragraph.

- **Detection:** split the body with the existing comment splitter (`content::parse` or whatever the TUI renderer uses, so note headers aren't counted). Parse each block with `pulldown-cmark` using the same options as the TUI renderer, and flag a `SoftBreak` event inside a paragraph (not inside a code block or table, which have no soft breaks anyway). Put it in a small pure function (e.g. `markdown_lint::has_run_on(body) -> bool`, or in `store`/`content`, wherever the renderer's parser setup can be reused without the TUI) with unit tests.
- **Filter:** add `run_on: bool` to `FilterSpec` (`src/filter.rs`; fix every construction site, most use `..Default::default()`) and match it in `FilterSpec::matches`. Add `--run-on` to `FilterFlags` and `build_spec` in `src/main.rs`, so `list`, `next`, `path`, `log`, `bulk`, `inbox` and the rest all get it.
- **Docs:** `docs/cli.md` "Filter flags (shared)"; the `--help` doc comment.

**Out of scope:** the TUI filter drawer (don't add a chip), `request`/`bulk` code (`kinds-1` owns the `Request` and `Bulk` variants and arms in `src/main.rs`, and `describe_bulk_mutation`), the skills (delta-lead).

**Evidence:**
- Unit tests that fail first:
  - flagged: a two-line paragraph; a run-on paragraph inside a note but not the description; `a\nb` in a list item's paragraph.
  - not flagged: blank-line paragraphs, lists, fenced code, a hard break (two trailing spaces or a trailing backslash), a table.
- An integration test running `list --run-on`.
- Paste in a note `yaks list --run-on | wc -l` against the live farm, using your OWN build read-only (`./target/debug/yaks list --run-on`), plus 3 example ids. That shows it finds real candidates; the 2026-10-17 survey estimated ~440 run-on lines.

**Gate:** `cargo test -p yaks`. **Judge:** delta-lead at landing.

---
▸ 2026-10-26T23:08:05Z [runon-1]

Start (runon-1): `date -u` = Mon Oct 26 23:08:00 UTC 2026.

`yaks path yaks-ef3b`: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-ef3b.md`

---
▸ 2026-10-26T23:13:18Z [runon-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-26T23:14:54Z [runon-1]

Decision: the detector lives in a new `src/markdown_lint.rs` (`has_run_on(body)`), and the pulldown-cmark options moved there as `markdown_lint::options()`, which `tui::mdrender::render` now calls. Rejected: putting it in `tui::content`/`tui::mdrender` (the filter is CLI-side and `filter.rs` should not depend on the TUI), and copying the five option flags (two sources of truth for "what joins").

Decision: blocks are split with `store::description` + `store::parse_notes`, not `tui::content::parse`. Same delimiter rule (`---` then a `▸` line); `store` is the non-TUI copy. Rejected: making `tui::content` reachable from `filter.rs`.

Decision: any `SoftBreak` event counts, without tracking whether it is inside a paragraph. pulldown-cmark only emits `SoftBreak` in inline content (paragraphs, tight list items, multi-line headings/table-less text), never in code, tables or HTML, so a paragraph check adds state and changes nothing. A two-line setext heading or a lazy list continuation is flagged, which is right: it renders joined.

Decision: `run_on` is CLI-only. The TUI drawer and saved views (`spec_from_json`) set `run_on: false` explicitly; no chip, no persistence, per the claim's out-of-scope. It does count in `content_active()` and `FilterFlags::any_set()` (so `bulk --run-on` alone is a valid selector).

---
▸ 2026-10-26T23:14:54Z [runon-1]

Evidence (runon-1).

Fail first, detector stubbed to `false` (`cargo test -p yaks --no-fail-fast -- run_on markdown_lint`):

```text
test markdown_lint::tests::note_headers_do_not_count ... ok
test markdown_lint::tests::leaves_well_formed_markdown_alone ... ok
test markdown_lint::tests::flags_lines_that_join ... FAILED
test filter::tests::run_on_keeps_bodies_with_joined_lines ... FAILED
test list_run_on_keeps_only_joined_lines ... FAILED
```

Stubbed to `true` (`cargo test -p yaks --bin yaks -- markdown_lint`), the negatives bite:

```text
test markdown_lint::tests::flags_lines_that_join ... ok
test markdown_lint::tests::note_headers_do_not_count ... FAILED
test markdown_lint::tests::leaves_well_formed_markdown_alone ... FAILED
```

Restored: all 5 pass. Full gate `cargo test -p yaks`: 871 passed, 0 failed; `yaks verify` PASS.

Live farm, own build, read-only (`./target/debug/yaks`):

- `list --run-on | wc -l`: 124 (of 474 non-dead); `list --run-on --all`: 129.
- Examples, each checked by eye: `yaks-00c4` (three description lines meant as paragraphs), `yaks-2c16` (prose line followed directly by a list... and two prose lines joined), `yaks-640a` (two description sentences on adjacent lines).

`list --help` shows: "Keep only yaks whose description or any note has run-on lines: adjacent lines that markdown joins into one paragraph (a soft line break). Candidates for a tidy".

---
▸ 2026-10-26T23:15:03Z [runon-1]

Correction to the evidence note: in `yaks-2c16` the run-on is the two prose lines "…and so forth." / "We'll need to think…". A bullet list right under a paragraph line is not a soft break (a list may interrupt a paragraph), so that part is not what flags it.

Summary (runon-1): new shared filter flag `--run-on`. `markdown_lint::has_run_on(body)` splits the body into description + notes (`store::description`, `store::parse_notes`) and flags any pulldown-cmark `SoftBreak`, parsed with `markdown_lint::options()`, now the single options source the TUI renderer also uses. `FilterSpec.run_on` is matched last in `FilterSpec::matches`; `FilterFlags.run_on` + `build_spec` give it to every filter-taking command. Docs: `--help`, `docs/cli.md` "Filter flags (shared)", README bulk selector list. Evidence: 5 tests, failing under both a `false` and a `true` stub; gate 871/0, verify PASS; 124 live candidates.

---
▸ 2026-10-26T23:15:03Z [runon-1]

moved: shaving -> shorn

---
▸ 2026-10-26T23:19:10Z [delta-lead]

Landed (delta-lead, yakherd) and pushed in ab5e667. Gate on the combined tree: 873 passed, 0 failed. Reproduced on a throwaway farm with my own build: list --run-on found the run-on yaks, bulk --run-on --request tidy dry-ran (skip marked), --commit queued one and skipped the yak awaiting a human, the tidy instruction is in the note, a bare request exits 2. Live farm: 124 yaks match --run-on.
