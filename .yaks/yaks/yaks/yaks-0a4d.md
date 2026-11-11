---
id: yaks-0a4d
title: 'Census: link forms and layout in real farms, and what a migration would touch'
status: hairy
type: task
priority: 2
created: '2026-11-11T21:53:57Z'
updated: '2026-11-11T22:07:11Z'
parent: yaks-1151
tags:
- links
- doks
---

Part of yaks-1151. Count the link forms actually present in the farms on this machine, find anomalies (like the `/doks/sessions/...` links in yaks-bede), and size a rewrite.

---
▸ 2026-11-11T22:07:11Z [triage-1]

Census done (read-only). Full per-farm report: [report.md](yaks-0a4d/report.md); script: [linkcensus.py](yaks-0a4d/linkcensus.py).

**Covered (12 farms):** yaks, joel, surveyor, canon-b4dde6c5, schism, hearth-1b94c9bb, haks, potemkin, memcraft, j15r, gnusto (schema 5, flat, 0 links), pg (schema 7 `wiki/`, broken `.git` file, so no remote). **Skipped:** canon-1e2efb11 (same commit d2700db as canon-b4dde6c5), hearth-3980235f (an older ancestor of hearth-1b94c9bb), repo-10b2880d (synthetic: 670 "Yak number N" yaks created within 10 minutes, no remote), and the tmp/test farms.

**Totals:** 5,283 links in 3,556 md files; 147 broken. `YAKS_DIR=… yaks doctor --json` agrees where it overlaps (yaks 1, surveyor 9, hearth 46 broken-link). Doctor does not scan captured sources; my script does, so it reports more. The working trees were unchanged after doctor ran.

| form (all farms) | n | broken |
|---|---|---|
| URL | 1,327 (236 github blob/tree, **0** to a farm's own repo) | 0 |
| farm-abs other first segment | 1,261 | 33 |
| attachment `<id>/<f>` | 762 | 2 |
| `/doks/…` | 355 | 4 |
| relative-down | 348 | 21 |
| yak-id / yak-id#note | 262 / 191 | 0 / 1 |
| sibling (in an attachment) | 226 | 20 |
| `/sources/…` / `/sessions/…` | 134 / 83 | 3 / 1 |
| `code:` / legacy `../src` | 107 / 39 | – |
| fragment-only | 100 | 0 |
| `../` that climbs out | 59 | 59 |
| `/doks/sessions\|sources` (yaks-bede) | **3** (yaks 2, joel 1) | 1 |

`@sha` pins: 207 (176 of them in haks). `#L` anchors: 170.

**Anomalies.** None of the 1,261 "other first segment" links were authored as farm links. joel has 1,151 in captured `sources/*/extracted.md` (site-relative links copied from web pages: `/docs` 955, `/install` 30, …). hearth has 110 in hearth-edaf's captured-doc attachments, plus a fixture `/joel/doks/…` (no `joel` herd exists in hearth). potemkin's 17 `/potemkin/…` links are herd-absolute and valid. Of the 59 upward links, 31 are pg `wiki/` `../topics/…` and the rest are in captured sources.

**Layout.** The largest directory is `yaks/yaks` with 885 entries. **No directory exceeds 1000** on this machine, so the "~10000" farm is not here. gnusto holds 530 files flat at its root. 40 filenames need URL-encoding, all in joel `sightmap-1add/compare 2/…`, an attachment *subdirectory* with a space. Attachment dirs: yaks 139, surveyor 36, hearth 30, haks 29, memcraft 22. Paths run up to 155 chars (yaks `sources/…/files/yass/archive/…`).

**Remotes.** All on github.com: joelgwebber ×4, cowpaths, fullstorydev ×2, rocketsurgery-games ×3, subtextdev. Each farm is the `yaks` branch of its code repo. No GitLab or enterprise host.

**Frontmatter and notes.** 3,370 of 3,556 md files have frontmatter (yak shape: `id,title,status,type,priority,created,updated,tags[,parent,depends_on]`). The markdown API (`gh api markdown`, gfm) renders it as `<hr>` plus a paragraph of the keys. There are 8,404 notes in total. A plain note header `▸ <ts> [actor]` becomes a `<p>`, which gives **no anchor**. A note with meta lines (`kind: move`…`---`) becomes a **setext `<h2>`** that swallows the meta lines, so its slug is not the timestamp; 1,034 notes are like this. GitHub **drops the href** from `[x](#2026-…T21:14:55Z)` and `[x](yaks-1a2b#2026-…:55Z)` (the colon reads as a scheme); `%3A`-encoded fragments survive.

**Migration sizing: authored docs only** (2,388 internal links; captured sources excluded):

| farm | GitHub resolves today | (ii) drop `.yaks` | (iii) also bede | yak-id/note | (i) doc-relative rewrites | unresolvable |
|---|---|---|---|---|---|---|
| yaks | 51% | 74% | 74% | 126 | 252/519 | 1 |
| joel | 45% | 92% | 92% | 19 | 152/277 | 1 |
| schism | 5% | 50% | 50% | 260 | 491/519 | 0 |
| hearth | 34% | 48% | 48% | 48 | 192/360 | 46 |
| potemkin | 44% | 100% | 100% | 0 | 22/39 | 0 |
| surveyor, canon, haks, memcraft, j15r, pg | 90–100% (attachments only, already doc-relative) | same | same | 0 | 0 | 9 / 31 (pg) |
| **all** | **1,210 (51%)** | **1,764 (74%)** | +1 | **453** | **1,109** | 88 |

- (ii) rewrites nothing and fixes every farm-abs link. All of the remaining gap is the 453 yak-id/note links, which fail on GitHub in **any** layout (no `.md`, no herd dir, no note anchor). Only a stored-form change (e.g. `/<herd>/yaks/<id>.md#…`) or a non-GitHub viewer fixes them.
- (iii) is trivial: 3 links in total, 2 of them mechanical.
- (i) rewrites every farm-abs and every yak-id link. Each of those is mechanical whenever the target resolves today. The 88 unresolvable ones need a person: mostly hearth fixtures, pg `../`, and surveyor siblings that are missing.

Caveats: links were extracted with a regex approximation of CommonMark (fences and code spans stripped), not pulldown-cmark. "GitHub resolves" is computed from the file tree, not checked against live GitHub. I did not check how the blob view renders frontmatter.
