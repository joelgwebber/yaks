---
id: yaks-ccb9
title: 'Commits phase 3: a derived Code: row in the yak detail pane'
status: shorn
type: feature
priority: 2
created: '2026-11-18T03:02:46Z'
updated: '2026-11-18T15:51:13Z'
parent: yaks-3515
depends_on:
- yaks-3f90
tags:
- tui
- git
---

Phase 3 of yaks-3515; needs phase 2.

A `Code:` row in the detail pane, built on read and never stored. It shows when a yak's notes hold commit links, `commit:` fields or checkpoint `branch:` fields, for example `Code: 2 landed, 1 branch`. Enter opens the list, newest first; each entry opens its commit view or names its branch. This is the TUI side of `yaks commits <id>`, so share its collection code rather than copying it.

#### 2026-11-18T03:03:14Z

*sources-triage*

Phase 3 depends on phase 2 (yaks-3f90): the rows open commit views.

#### 2026-11-18T15:36:47Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T15:51:12Z

*sources-triage · commit: e9f48aedb18f1f4aa0128fb278f67e2b472cd817*

Landed on main (`yaks update --commit` records the sha in this note's metadata).

- `src/yakcommits.rs` is the one collection: a note's `commit:` field plus `commit:` links (and, given `Space::read_url`, host commit URLs) in its text, once each; and the checkpoint branches (`branchlog::gather` now calls it). `Farm::commits` (`yaks commits`) and the TUI's `Code:` rows both read it.
- Detail pane: `Code:` rows below the fields, newest first, `kind short · when`, each a commit link that opens the commit view; then `branch remote/name`. No git call while rendering, so a host URL in prose counts in `yaks commits` but not in the row (it still opens when followed).
- Evidence: gate green (`cargo fmt --check`, `cargo test -p yaks --no-fail-fast`); new tests `yakcommits::tests::*`, `the_code_rows_list_a_yaks_commits_newest_first_and_its_branches`; headless on the real farm, yaks-ab81 shows `Code: landed 65fa57e · 54m ago` and Tab+Enter opens its commit view: [frame](yaks-ccb9/code-row-to-commit.txt).
- Docs: `docs/tui.md` (detail pane, Commits), `docs/cli.md` and `--help` for `commits`.

#### 2026-11-18T15:51:13Z

*sources-triage · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
