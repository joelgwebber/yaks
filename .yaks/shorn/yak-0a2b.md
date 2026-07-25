---
id: yak-0a2b
title: 'Docs cleanup: repo description, README staleness, docs/ layout'
type: task
priority: 3
created: '2026-07-25T23:02:38Z'
updated: '2026-07-25T23:08:44Z'
---

Post-relaunch polish: refresh the GitHub repo description; scan README + docs for stale references (slash commands, old sync model, yak.py paths); clarify docs/design/sync.md naming (it's actually the current one-way projection design, not stale); confirm what docs/index.html is (GitHub Pages web UI) and whether its location is intentional.

---
▸ 2026-07-25T23:08:44Z
Done. (1) Repo description updated via gh (was 'Simple file-based task tracking for use by claude-code' -> cross-harness one-liner). (2) docs/design/sync.md is NOT stale — it's the current one-way projection design; renamed to projection.md to match its title + kill the 'we retired sync' confusion, updated the one yak-tracker skill ref. (3) docs/index.html = the 'Yak Board' web UI, live at joelgwebber.github.io/yaks via GitHub Pages (source main/docs) — intentional; added a 'Web board' section to the README documenting it. (4) README intro now names the PyPI dist (yakherder). Bumped 0.1.79->0.1.80 (skill ref + README are shipped payload). FLAGGED for user: the Claude/Codex plugin-install command syntax and the Zed '/yak' invocation in the README may be stale — worth verifying against current CLIs; repo homepageUrl is empty (could point to PyPI or the board).
