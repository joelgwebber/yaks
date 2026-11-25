---
id: yaks-e0cd
title: 'Skills: say a running yaks tui syncs every farm commit, so a farm write is published'
status: hairy
type: task
priority: 2
created: '2026-11-25T04:22:19Z'
updated: '2026-11-25T04:22:19Z'
tags:
- skills
source: 'feedback-to-yaks:fullstorydev/surveyor#2026-11-24: commits that reach origin in the background look like someone else''s push'
---

From fullstorydev/surveyor's feedback dok (2026-11-24 entry). An agent told the person a dok would stay local until reviewed; the person's open `yaks tui` synced it to `origin/yaks` first.

Checked: the `yaks` skill, "Where the farm lives", says "Nothing syncs on its own". That is wrong: `yaks tui` syncs every `sync_interval` (default `5m`; `src/tui/farmsync.rs`, `docs/cli.md` "The CLI never syncs by itself (only `yaks tui` does ...)").

## Fix

Replace the sentence: the CLI never syncs, but a running `yaks tui` on the machine syncs every few minutes, whoever made the commit. Treat any farm write as published; never promise to hold one back; keep a draft as a yak attachment or in scratch space. Check `docs/` and `yaks-working`/`yaks-coordinating` for the same claim.
