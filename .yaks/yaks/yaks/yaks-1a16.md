---
id: yaks-1a16
title: Keyboard tweaks for tag auto-completion
status: hairy
type: task
priority: 3
created: '2026-11-11T20:23:06Z'
updated: '2026-11-11T20:26:33Z'
tags:
- tui
---

Currently, the tag autocomplete affordance eats `<tab>` right after you select a tag. Looks like this is because it leaves the cursor right after the tag just added, so `<tab>` pushes you back into the completion overlay.

"Fixing" this may be as simple as just adding a space after the selected tag, so that you can tab to the next field.
