---
id: yaks-dedc
title: Weird, duplicative structures in wiki sources
status: shorn
type: task
priority: 3
created: '2026-11-03T23:50:22Z'
updated: '2026-11-04T20:03:47Z'
tags:
- doks
---

---
▸ 2026-11-04T00:04:43Z [Joel Webber]

Example: In another repo, I imported a confluence doc, which the agent decided to fetch as HTML, then convert to markdown. The resulting sources came out like this:
```python
sources/[date-hash0]/  # "[name] (Confluence, draft v10)"
  source.md            # Markdown with frontmatter, ironically full of HTML garbage
  extracted.md         # *Only* HTML garbage; no frontmatter

sources/[date-hash0]/  # "[name] (Markdown conversion)"
  source.md            # Actual markdown with frontmatter, pointing back to the HTML.
  extracted.md         # The exact same markdown, without frontmatter
```

I'm not quite sure what in the tools and skills possessed the agent to do this, but it's massively redundant and pointless. In scenarios like this, it really should only keep a single source file with the markdown conversion and frontmatter with provenance information. The raw HTML is completely useless, as is the description "Snapshot of the local file /some/random/tempfile/location/...". It actually has a perfectly good title, but it decided to keep the useless temp-file provenance for some reason.

Let's look carefully at the tool/skill structures likely to have led to this behavior, and try to nudge them in the right direction. Consider the totality, rather than just trying to slap a "don't do this one weird thing" band-aid on it.

---
▸ 2026-11-04T17:23:10Z [source-fix]

## Diagnosis (source-fix, from the two sources in `joel`: `sources/2026-11-03-bea6d0de` and `sources/2026-11-03-14a63718`)

Both the skill and the tool led to this; the skill is the direct cause.

**Skill.** `yaks-research` ("Save before you rely on it") says: convert with your own tools, save the original with `--url`, "and save the text beside it with `--part-of <that source's source.md>` when it is a separate file". The agent did exactly that: the Confluence HTML became one source, its markdown conversion a second source `part_of` the first. Nothing says to stop at the form you read, that a raw HTML original is rarely worth keeping, or that `--part-of` is for parts (screenshots), not for conversions.

**Tool.**

- There is no way to give a source its own converted text. `extracted.md` is only ever a byte copy of a text original, so the skill points at a second `source add`, and every conversion becomes its own source.
- `html` is in `is_text_name`, so the HTML source's `extracted.md` is the raw HTML, byte for byte: "extracted" HTML garbage, not text.
- A markdown original gets an identical `extracted.md` (kept: search and the TUI read it, and it is Hearth's format; it is a duplicate only in the trivial case).
- `description` is always built from the absolute path: "Snapshot of the local file /private/var/folders/.../delta-scratch-.../x.md", even when `--title` and `--url` say everything. `resource` (no `--url`) or `local_copy` (with it) repeats the temp path. About 30 of the 39 sources in the `joel` farm say "Snapshot of the local file /tmp/...".
- The converted source got no `--url`, so its only link to the Confluence page is through `part_of`.

## Plan

Land before the doks rename (yaks-cc40): this is localized to `src/sources.rs`, the `source` help and docs, and the `yaks-research` skill, and cc40 then sweeps what this writes instead of conflicting with it.

Children:

- the tool: `source add --text`, no path-derived description, no temp paths as provenance, HTML not "extracted"
- the skill and docs: one source per thing, `--url` and `--title` always
- tidy the two example sources in `joel` into the one they should have been

Decision: a markdown original keeps its `extracted.md` (Joel agreed); a non-text original (HTML, PDF) gets one only when the saver supplies the text with `--text`.

---
▸ 2026-11-04T17:23:26Z [source-fix]

kind: move
from: hairy
to: shaving
child: yaks-d19c
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T17:29:24Z [source-fix]

All three children shorn; code and skill changes are on `yaks-dedc-sources` in the yaks checkout (two commits), awaiting Joel's go to land. After landing: `cargo install --path .` (per AGENTS.md), then yaks-cc40 can proceed. Existing sources in `joel` that say "Snapshot of the local file /tmp/..." (about 30) are left as written: sources are write-once.

---
▸ 2026-11-04T17:29:25Z [source-fix]

kind: move
from: shaving
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
