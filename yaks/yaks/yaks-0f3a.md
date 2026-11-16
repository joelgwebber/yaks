---
id: yaks-0f3a
title: Shard the yaks directory for GitHub's 1000-entry listing cap?
status: hairy
type: idea
priority: 4
created: '2026-11-14T11:21:03Z'
updated: '2026-11-14T11:21:03Z'
parent: yaks-1151
tags:
- links
- git
---

Deferred from yaks-1151 (Joel, 2026-11-14): reconsider later.

GitHub's web listing truncates a directory at 1,000 entries ("4,185 entries were omitted" on nodejs/node). Direct links to files still work, and the farm branch has no code search to lose. This farm's `yaks/yaks/` holds 881 files.

Sharding by id prefix (`<herd>/yaks/<2 hex>/<id>.md`) keeps a yak's path stable for life, but since links store paths (the stored yak-link decision) it is another link rewrite, so weigh it before the next schema bump rather than after. Alternatives: a generated index page (clashes with "any index is derived"), or nothing.

Revisit when a farm passes about 800 yaks in one directory and someone browses it on the web.
