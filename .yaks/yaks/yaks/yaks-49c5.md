---
id: yaks-49c5
title: 'Sources: a readable slug in the path beside the date and hash?'
status: hairy
type: idea
priority: 4
created: '2026-11-14T17:52:03Z'
updated: '2026-11-14T17:59:07Z'
parent: yaks-1151
tags:
- links
- doks
needs: agent
---

From Joel's answer on yaks-5f70: while the farms are small, should paths carry a readable slug as well as the date and hash? Session notes already do (`sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`). Sources do not: `sources/<date>-<sha8>/` names a snapshot by date and content hash only, so a source link in prose or a GitHub tree listing says nothing about it.

## Options

- Leave as is: the id is stable and content-derived, and `source.md` carries the title.
- `sources/<date>-<sha8>-<slug>/`, the slug from `--title`. Readable on GitHub. Costs: the directory name is no longer derivable from the content, so the same bytes saved with a different title is a rename, `yaks source add` dedup by hash has to match on the prefix, `--refresh` keeps the id so the slug can go stale, and every link to a source (214 sources in this farm) changes in the migration.
- A slug in `source.md`'s title only (what is there now).

Lean: decide with the schema-9 migration, since it rewrites links anyway; if yes, do it in that same pass. Needs an ask when picked up.

---
▸ 2026-11-14T17:53:11Z [linkplan-lead]
kind: ask
needs: human
---

asked: needs human

Needs deciding before the schema-9 migration (yaks-acc0) starts, because it rewrites every link anyway. Sessions already carry a slug in their path; sources (`sources/<date>-<sha8>/`, 214 here) do not.

1. **Leave as is.** The id stays stable and content-derived; `source.md` carries the title. Lean: this. A source is cited from a dok's `sources:` list and a link, both written by yaks, so a reader on GitHub follows the link rather than reading the directory name.
2. **Add the slug**, `sources/<date>-<sha8>-<slug>/`, slug from `--title`. Readable in a GitHub tree listing. Costs: the same bytes saved under another title is a rename; `source add` dedup has to match on the `<date>-<sha8>` prefix; `--refresh` keeps the id so a retitled source's slug goes stale; every source link changes in the migration.

Answer "1" to leave it and I will shear this yak, or "2" and it joins the migration.

---
▸ 2026-11-14T17:59:07Z [sched-1]
kind: answer
needs: agent
---

Whoops, my mistake. As long as we can implement the de-duping stuff reliably, let's add the slug. I find it very difficult to scan sources quickly otherwise.
