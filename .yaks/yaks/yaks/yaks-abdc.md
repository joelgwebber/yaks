---
id: yaks-abdc
title: Reconsider design for moving among tabs while in detail mode
status: hairy
type: idea
priority: 3
created: '2026-11-14T16:12:26Z'
updated: '2026-11-14T16:12:26Z'
tags:
- tui
---

It appears that, because selecting a tab always selects a yak/dok, switching tabs with {/} in detail mode often *also* changes the detail itself, which is more than a little confusing. A few ideas for resolving this:

- Disable (grey out?) tabs where the current yak/dok *isn't* present. Switching tabs skips to the next one (if any) containing it, and auto-selects it, so that the detail remains consistent.
- Make list-ish views have a "no selection" state, so that tabs can be changed without updating the currently-selected item for details.
