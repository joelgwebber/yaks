---
id: yaks-d695
title: Add pull-request(s) field to frontmatter
status: shorn
type: feature
priority: 3
created: '2026-08-26T12:58:19Z'
updated: '2026-10-31T17:24:17Z'
parent: yaks-039b
tags:
- schema
---

Alongside 'source' for upstream issues, we need a place to track active PR(s) as well.

I think this needs to handle multiple PRs, as that's a pretty common pattern.

Source can be singular, because we want to discourage a single yak from representing multiple issues (that's what herds are for).

---
▸ 2026-10-31T15:56:26Z [delta:Cloud Runners]

Adopted into `yaks-039b` (2026-10-31) so the PRs field stays consistent with the other "where does this yak's code live" fields (`yaks-bdf9` landed shas, `yaks-a3fc` worker branches). Where it lives (frontmatter vs note metadata) follows `yaks-6a11`.

---
▸ 2026-10-31T16:54:08Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:54:09Z [yakherd]

Decision (yakherd, 2026-10-31, within the `yaks-039b` direction): the PR list is current state, so it lives in frontmatter as `prs:` (a list of URLs), beside `source:`. Checkpoint and landed events live in note metadata.

Claim: owner worker `prs-1`. Owns: `model::Task` (new `prs` field), `store` parse/render of `prs:`, the merge driver's list merge for it, `yaks update --pr <url>` / `--remove-pr <url>` (and `create --pr`), the `prs` lines in `yaks show` text and JSON, the TUI detail header, `--json` list output if `source` appears there. Evidence: round-trip and merge tests, CLI tests, a TUI frame. Judge: yakherd.

---
▸ 2026-10-31T16:54:51Z [prs-1]

Started 2026-10-31T16:54:47Z (date -u). Yak file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-d695.md

[yaks__tui__tests__detail_header_lists_prs_as_links](yaks-d695/yaks__tui__tests__detail_header_lists_prs_as_links.snap)

---
▸ 2026-10-31T16:59:47Z [prs-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-31T17:00:02Z [prs-1]

Decision: `prs` is current state in frontmatter, a block list written right after `source:`, omitted when empty (per yakherd Decision); parsed as block or inline `[a, b]` like `labels`. Rejected: rendering inline (labels are block too).

Decision: `--pr` / `--remove-pr` take ONE url per flag, repeated (`--pr A --pr B`), not `num_args = 1..`: URLs are not split on commas/whitespace like labels, and a greedy flag would swallow the positional id. Rejected: reusing the label normalizer (a URL can contain commas).

Decision: PR URLs are trimmed, empties dropped and duplicates removed by one `farm::dedup_prs` (used by create and update); add of an existing URL / remove of a missing one leaves the file unmodified (no `updated:` bump), checked in tests/prs.rs.

Decision: TUI detail shows `PRs:` rows (first row labelled, one URL per row) right under `Source:`, URLs as links via `Target::Url`, read-only. Not added to the edit form (`E` on a PR row does nothing) and `create` form passes no PRs; editing is CLI-only. Rejected: a form row (touches create.rs/handlers/snapshots beyond scope; list editing needs its own design). Follow-up if wanted.

Decision: `store::edited` (used by `yaks status`) counts a `prs` change as an edit, like `source`. `list --json` does not include `source` today (task_value is shared by show/create/list JSON, so `prs` appears wherever `source` does, only when non-empty).

Summary: model/store/json/mergedriver/farm/main/tui detail + docs (cli.md, tui.md) done. Tests: store round trips (block, inline, unknown-field preservation), merge-driver both-sides-add and add/remove, tests/prs.rs CLI (update/--remove-pr/idempotent re-add/create/show text+json absent-when-empty), TUI detail test + snapshot `detail_header_lists_prs_as_links` (attached). Gate `cargo test --workspace` green (759 unit + integration). README lists no fields so unchanged.

For the yakherd, skills text (not edited by me): `yaks` skill task-format block should list `prs:` after `source:` ("prs: a list of pull-request URLs, the yak's current PRs; block list, omitted when empty"); the `update` row should add `--pr <url>` / `--remove-pr <url>` (repeatable, re-adding is a no-op) and `create` row `--pr <url>`; `show`/`--json` carry `prs` only when non-empty. yaks-tracker/yaks-coordinating may mention `--pr` where PRs are recorded.

---
▸ 2026-10-31T17:00:02Z [prs-1]

moved: shaving -> shorn

---
▸ 2026-10-31T17:00:48Z [delta:Cloud Runners]

kind: landed
commit: 601fa0731ce628555b5a8004d3a03933ea99e271
source: 5bddef7988c8688bfae719132c31e4267363e8c6
branch: main
---

landed: 601fa07 yaks-d695: add prs: pull-request list to frontmatter, update --pr/--remove-pr, create --pr, show, TUI detail

---
▸ 2026-10-31T17:02:25Z [yakherd]

Yakherd: landed as `601fa07` (recorded automatically by `yaks land`). Gate green; reproduced on a scratch farm: `create --pr`, `update --pr` de-duplicates, `--remove-pr`, and `show --json` lists `prs`. Skills lines added in 6cad621.

---
▸ 2026-10-31T17:24:16Z [yakherd]

kind: landed
commit: 7c749040037cdf390a62682799617113c862fd53
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): 7c74904 yaks-d695: skills document prs: (--pr/--remove-pr), and update --meta in the command map (was `6cad621`)

---
▸ 2026-10-31T17:24:17Z [yakherd]

kind: landed
commit: d3910587d4b48ab8190ab8fed62b348f51a25320
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): d391058 yaks-d695: add prs: pull-request list to frontmatter, update --pr/--remove-pr, create --pr (was `601fa07`)
