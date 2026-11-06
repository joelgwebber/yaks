---
id: yaks-c316
title: 'Dok conventions: dated doks, a created date, and folder layout'
status: hairy
type: task
priority: 3
created: '2026-11-06T21:37:12Z'
updated: '2026-11-06T21:48:51Z'
parent: yaks-3f35
tags:
- doks
- design
needs: agent
---

From yaks-3f35, items 6 and 7.

## Dates (item 6)

- **Dated artifacts.** Screenshots, meeting notes and the like are about a day. The schism import invented a schema (`schemas/screenshot.md`: `date`, `commit`, `yaks`) and a `YYYY-MM-DD-slug.md` file name. Nothing suggests either, so each farm will invent its own.
- **No created date.** A dok's frontmatter says nothing about when it was written; a reader runs `git log` on the farm branch. Yaks have `created:`/`updated:`.

To decide: should `yaks dok new` stamp `created:` (and an edit through yaks `updated:`)? A dok format change, so ask. Lean: stamp `created:` only; `updated:` goes stale under hand edits, and `verified:` already means "facts re-checked". Separately, a skill rule for dated doks: a `date:` field (typed, so `dok query` sorts by it) and a date-prefixed file name.

## Folders (item 7)

The `yaks-research` skill says "`doks/topics/`, `doks/projects/` and `doks/people/` are a good start; follow whatever the farm already uses". The importer couldn't tell whether screenshots belong under `doks/topics/` or `doks/screenshots/`. A rule to state: folders follow kinds, not subjects. `doks/topics/` holds `topic` doks about any subject, a typed kind gets its own folder (`doks/screenshots/` for `type: screenshot`), and a subject never gets a folder of its own (use tags).

## Evidence

The skill text (`yaks`, `yaks-research`) and, if `created:` is stamped, a test and `docs/`.

---
▸ 2026-11-06T21:37:50Z [schism-feedback]
kind: ask
needs: human
---

asked: needs human

Two dok-format questions, my lean on each.

1. **`created:` on doks.** Have `yaks dok new` stamp `created:` (not `updated:`, which hand edits would leave stale). Alternative: leave dates to git history.
2. **Folders follow kinds, not subjects.** `doks/topics/` holds `topic` doks on any subject; a typed kind gets its own folder (`doks/screenshots/` for `type: screenshot`); subjects are tags, never folders. Alternative: leave layout to each farm, as the skill says now.

---
▸ 2026-11-06T21:48:51Z [sched-1]
kind: answer
needs: agent
---

1. Agreed.
2. That also works for me.
