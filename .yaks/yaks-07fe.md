---
id: yaks-07fe
title: 'yaks sheds is slow (~1.2 s for 4 sheds): profile it and make the common path cheap'
status: hairy
type: task
priority: 2
created: '2026-10-12T16:04:57Z'
updated: '2026-10-12T16:04:57Z'
parent: yaks-38dc
labels:
- cli
- delta
---

`yaks sheds` takes ~1.2 s on this machine (4 live sheds plus 51 orphaned Delta pins, release build, measured 2026-10-12); `yaks inbox --sheds` about the same. Joel: worth finding out why before the TUI (yaks-38dc, approach B) builds on it with a background refresh.
Profile first, then fix. Suspects, not measured: one `git` process per question per shed (rev-parse, symbolic-ref, merge-base, two rev-list counts, status --porcelain, reflog read, ls-tree + cat-file for the base farm); resolving every pin group in the host, including the 51 gone ones, on every call; reading each shed's whole farm plus a base farm materialized into a temp dir; the reference farm read once more for `landed`.
Goal: know where the time goes (per-shed and per-step timings), then make the common path cheap: e.g. skip or cache gone pins, batch git calls, skip the base-farm read when a shed's farm and HEAD are unchanged since the last call (a derived, per-user cache only: the AGENTS.md rule for any index), and separate what can be event-driven (farm mtime) from what needs git.
Evidence: before/after timings on this machine and on ~/src/j15r.
