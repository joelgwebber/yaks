---
id: yaks-66a3
title: 'Pinned header: a rule under it, and the status icon in the full header'
status: shaving
type: task
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T04:31:30Z'
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
