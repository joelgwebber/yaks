---
id: yaks-c316
title: 'Dok conventions: dated doks, a created date, and folder layout'
status: shaving
type: task
priority: 3
created: '2026-11-06T21:37:12Z'
updated: '2026-11-07T17:38:40Z'
parent: yaks-3f35
tags:
- doks
- design
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

---
▸ 2026-11-07T17:04:28Z [herd-stars]
kind: pickup
---

picked up

herd-stars: doing it myself: dok new stamps created:, and the folders-follow-kinds and dated-dok rules in the skills.

---
▸ 2026-11-07T17:04:44Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:38:40Z [herd-stars]

Done by herd-stars in `2206248e`, per Joel's two answers.

- `yaks dok new` (and the TUI's new-dok) stamps `created:` with the time, UTC, as a yak's; a `--set created=...` wins. Nothing stamps `updated:`. `created` joins the schema META keys, so a typed kind's schema need not list it ([Farm::dok_new](code:src/farm.rs@2206248e#L2248-L2275)).
- Skills: `yaks-research` states "folders follow kinds, not subjects" and the dated-dok rule (`date:` field typed `date`, date-prefixed name); the `yaks` skill's Doks line says both briefly.
- `docs/cli.md` and `dok new --help` updated.
- Evidence: `tests/doks.rs` (the caller's `created` wins), the TUI new-dok test (a parseable RFC 3339 stamp), and a scratch farm: `created: '2026-11-07T17:38:25Z'` on a new dok, `doctor` clean with a typed schema. Full gate green.
