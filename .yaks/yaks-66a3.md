---
id: yaks-66a3
title: 'Pinned header: a rule under it, and the status icon in the full header'
status: shorn
type: task
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:39:47Z'
parent: yaks-faf5
labels:
- tui
---

Two small fixes in `render_detail` / `pinned_header_line` (src/tui/render.rs) and `detail::build`:
- When the header collapses to the pinned one-line summary, give it a bottom border (a dim `─` rule row) so it is clearly separated from the content.
- The full (unscrolled) header shows no status emoji; the `Status:` field (or the `Task:` line) should carry the same icon the pinned header shows.
Before/after frames (pinned and unpinned) attached; update the `tui-detail*.svg` docshots.

---
▸ 2026-10-17T04:31:30Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:31:30Z [delta-lead]
Claim (yakherd delta-lead, 2026-10-17): owner **hdr-1**.
Scope: `render_detail` and `pinned_header_line` in src/tui/render.rs (a dim `─` rule row under the pinned one-line header; content still pushed down, nothing hidden; mouse hit area `hits.detail` must stay aligned with content rows), and the `Status:` field in `detail::build` (src/tui/detail.rs) carrying the status emoji, e.g. `Status:      🦬 hairy`. Only those functions in detail.rs: other workers own `scan_body_links` and `humanize_date`/`opt_date` there. Also check `App::detail_page` / scroll-into-view arithmetic accounts for the extra pinned row (see headless.rs on_resize and the live loop).
Evidence: tests that fail first (pinned header + rule occupy two rows; Status line contains the emoji); headless frames pinned and unpinned attached. Judge: coordinator at landing.
Do NOT regenerate docs/assets docshots (several workers would collide on the SVGs; the coordinator regenerates them after landing). Re-accept any insta snapshot your change moves (`INSTA_UPDATE=always cargo test`, or `cargo insta accept` if installed) and read every changed .snap diff before committing. `cargo fmt` reformats files nobody touched (src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs): run `cargo fmt -- <your files>` and `git checkout` those three if they changed. Docs/help parity per AGENTS.md (docs/tui.md, help_content in src/tui/render.rs) where behaviour a user sees changes.

---
▸ 2026-10-17T04:32:28Z [hdr-1]
Start: 2026-10-17T04:32:25Z; path: /home/joel/src/yaks/.delta/worktrees/vhcj0wne3pb5/yaks/.yaks/yaks-66a3.md

![pinned](yaks-66a3/pinned.txt)

![unpinned](yaks-66a3/unpinned.txt)

---
▸ 2026-10-17T04:39:12Z [hdr-1]
verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T04:39:47Z [hdr-1]
Decision: the rule is a second reserved row (`PINNED_ROWS = 2` in render.rs, shared with `App::detail_rows_shown`) drawn as a DarkGray `─` line; rejected a ratatui `Block` bottom border (would need a nested layout and would not share the row constant with the scroll arithmetic) and overlaying the first content row (hides content).
Decision: Status field is `🦬 hairy` (emoji from `Status::emoji()` + word) in the existing `Status:` field; rejected adding the emoji to the `Task:` section line. `edit_target_at` matches on the `Status:` prefix, so `E` is unaffected.
Decision: pin condition stays `detail_scroll > 0`; the viewport shrinks by 2 (`detail_rows_shown`), so `detail_max_scroll` / `scroll_line_into_view` / wheel clamp follow automatically. headless/runtime `detail_page` untouched.
Decision: moved mouse test `wheel_scrolls_detail_and_click_moves_line_cursor` to a 12-row frame / detail_page 9 (its 10-row frame no longer fits Labels: under a 2-row pin plus the longer layout); intent unchanged.
Summary: pinned header gets a dim rule row (content pushed down 2 rows, hits.detail aligned to content_area); Status: field carries the emoji; 5 insta snaps re-accepted (4 Status emoji lines, 1 find-overlay shows the rule and one fewer blank row); new tests pinned_header_has_a_rule_under_it_and_pushes_content_down_two_rows and full_detail_header_shows_the_status_emoji; scroll_into_view test updated (8 rows shown, not 9); docs/tui.md updated. No help_content change: no key changed. docshots not regenerated (per claim note). Frames attached: pinned.txt, unpinned.txt. Full `cargo test --release` and `cargo test --workspace` pass.

---
▸ 2026-10-17T04:39:47Z [hdr-1]
moved: shaving -> shorn
