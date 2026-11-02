---
id: yaks-aae6
title: 'Design: kinds of farm document, layout and addressing'
status: shorn
type: idea
priority: 1
created: '2026-11-01T18:52:36Z'
updated: '2026-11-01T20:09:15Z'
parent: yaks-7708
tags:
- wiki
- design
---

Kinds of document in a farm, where each lives, and how each is addressed. Parent: the farm wiki umbrella.

## Kinds

| Kind | Addressed by | Lives at | Notes |
|---|---|---|---|
| yak | id (`yaks-1a2b`) | `<herd>/yaks/<id>.md`, attachments in `<herd>/yaks/<id>/` | unchanged: status lifecycle, notes log |
| page | farm-relative path (`wiki/topics/x.md`) | `wiki/**.md`, or `<herd>/wiki/**.md` for one herd's pages | OKF frontmatter plus a markdown body; no notes log |
| source | id `<YYYY-MM-DD>-<8 hex of sha256>` | `sources/<id>/`: `source.md`, the original, `extracted.md` | written once; deduplicated by sha256 across the farm |
| schema | type name | `schemas/<type>.md` | describes the fields of a typed page (and extra yak fields, see the frontmatter design yak) |

Proposed layout (schema 7):

```text
.yaks/
  config.yaml  schema  .gitignore
  <herd>/
    yaks/<id>.md, yaks/<id>/      yaks, unchanged
    wiki/**.md                    optional: pages owned by one herd
  wiki/**.md                      farm-wide pages
  sources/<id>/                   saved outside material
  schemas/<type>.md               page types
```

`wiki`, `sources` and `schemas` become reserved herd names. The schema-6 reservation of `docs` was never built; it stays reserved so no herd can take it, but nothing lives there.

## Pages

- Frontmatter follows OKF v0.2 (the spec Hearth adopted): `type` (defaults to `note` when missing), `title`, `description`, `tags`, `sources`, `status` (`draft | stable | deprecated`), `stale_after`, plus anything else, kept verbatim. Hearth's untyped kinds carry over: `note`, `topic`, `howto`, `summary`.
- Pages are files: an agent may edit one with its own editor tools. `yaks` validates (`doctor`), commits with attribution, and moves (`yaks mv` rewrites every link to it, as `rename` does for yaks).
- CLI: `yaks page list [PREFIX]`, `page show`, `page new PATH --type --title`, `page set PATH key=value`, `page query --type T [--where ...]`, `mv`. JSON on every query, for agents and later Hearth.
- A page outlives the yak that wrote it, but stays in context: the yak links it, `yaks refs --incoming` on the page lists the yaks that mention it, and a superseded page is marked `status: deprecated` with `superseded_by:` instead of deleted.

## Sources and promotion

- `yaks source add PATH|DIR [--url URL] [--part-of SOURCE]` snapshots a local file or small directory (Hearth's rules: sha256 dedup, size caps, `source.md` with where and when). Fetching URLs, HTML conversion and PDF text stay outside yaks: an agent or Hearth fetches, then hands yaks the file with `--url`.
- **Promotion**, the step research yaks lack today: `yaks promote <id>/<file> --source` moves an attachment into `sources/`, and `--page wiki/x.md` moves a markdown attachment into the wiki with frontmatter. Either way the yak's link is rewritten to the new place.

## Two branches, always

Every repo with a farm has a code branch and the farm branch, including a knowledge-only repo, where `main` may hold nothing but `AGENTS.md` and scaffolding. In the `joel` repo, `wiki/`, `raw/sources/`, `schemas/` and the notes move into the farm; the experiment code under `projects/` stays on `main` and pages link to it with code links (see the link grammar yak). No in-tree `.yaks/` comes back (`yaks-a99f`).

## Out of scope here

Hearth's `core/`, `instructions/`, `views/`, `schedules/` and protected paths are agent-layer concepts and stay in Hearth. Session notes (`raw/sessions/`) wait until the workflow layer.

---
▸ 2026-11-01T18:53:39Z [delta:farm-wiki]

kind: ask
needs: human
---

asked: needs human

Three choices in the layout; I'm building on the leans unless you say otherwise.

1. **Where pages live:** farm-wide `wiki/` plus optional `<herd>/wiki/`, with `wiki`, `sources`, `schemas` reserved herd names. Lean: yes. The alternative was `docs/` (already reserved), but "docs" is the name we just decided means product docs on `main`, so reusing it would blur the line.
2. **Sources at the farm root** (`sources/<id>/`, shared by all herds, deduplicated farm-wide) rather than per herd. Lean: farm root. A paper cited by two herds is one source.
3. **Two branches even for a knowledge-only repo** (`main` holds `AGENTS.md` and scaffolding, or experiment code as in `joel`). Lean: yes; one layout everywhere, and `joel` already wants code on `main`.

---
▸ 2026-11-01T20:02:20Z [Joel Webber]

kind: answer
needs: agent
---

1. Yup, I dig that. Wiki's a well-known concept, and we have a pretty good link syntax for covering all the cases, I think.
2. Yes, exactly.
3. Agreed; the degenerate case costs ~nothing, so let's keep it consistent.

---
▸ 2026-11-01T20:09:14Z [delta:farm-wiki]

kind: pickup
---

picked up

Decision (Joel, 2026-11-01): all three leans accepted.

- Pages live in `wiki/` and optional `<herd>/wiki/`; `wiki`, `sources` and `schemas` are reserved herd names (`docs` stays reserved, unused).
- Sources sit at the farm root, `sources/<id>/`, deduplicated farm-wide.
- Every repo with a farm has two branches, including knowledge-only repos.

---
▸ 2026-11-01T20:09:15Z [delta:farm-wiki]

kind: move
from: hairy
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
