---
id: yaks-a096
title: 'Skills: importing a repo''s docs and history into a farm'
status: hairy
type: task
priority: 2
created: '2026-11-06T21:37:12Z'
updated: '2026-11-06T21:37:12Z'
parent: yaks-3f35
tags:
- skills
- doks
---

From yaks-3f35, items 5 and 10. The schism import (schism-5b88) moved a repo's `design/`, `devlog/` and `screenshots/` into its farm and had to work out every step itself. The skills cover saving outside material as sources (`yaks-research`) and writing live session notes (`yaks-upkeep`), not moving a project's own documents in.

## What the guidance needs

- **Doks, not sources.** A repo's own docs that the farm will now maintain become doks (rewritten into the farm's kinds, then the originals removed from the repo). A source is a frozen outside snapshot, cited, never edited; don't import your own living docs as sources.
- **History.** A devlog becomes backdated session notes (yaks-0fa7), then review passes in date order, as if they had been written live.
- **Images.** Attach to the dok or note, per yaks-db66.
- **Repointing.** Yaks and code comments that link the moved repo files: in the yaks, a present-day note pointing at the dok (don't rewrite old notes, which recorded what was true then); in code, a comment pointing at the dok's farm path. Say how to find them: `yaks find <old path>`, `git grep`.
- **One commit for the code side**, separate from the farm (the farm never enters a code commit).
- **Afterwards, offer upkeep (item 10).** After a bulk import, offer to create the recurring "Review session notes" and "Tidy the doks" yaks (`yaks-upkeep`, "Setting it up"). The schism agent didn't, because nobody asked: the skill should say to offer, not wait.

Where it lives: lean a section in `yaks-research` ("Moving a project's docs into the farm") that points into `yaks-upkeep` for the history part. Folder layout and dated doks are yaks-c316; citing session notes is yaks-5f70. Cold-read it with a cheap model afterwards (AGENTS.md).
