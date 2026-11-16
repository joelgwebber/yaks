---
type: topic
title: 'Note format: anchors that render on GitHub'
created: '2026-11-14T11:22:41Z'
description: Design for a note format whose notes have stable anchors on GitHub and other hosts; read before changing how notes are written or parsed
tags:
- links
- design
---

# Note format: anchors that render on GitHub

Design for [yaks-6c65](yaks-6c65), part of [yaks-1151](/doks/design/farm-layout-and-links.md). A yak's notes are the journal of the work, and today nothing in the file gives a note an anchor on a host that renders markdown. This dok picks a shape that does, keeps it readable and parseable, and lists what changes in the code.

## What is wrong today

A note is `---`, then `▸ <ts> [actor]`, then optional `key: value` lines closed by a second `---`. Rendered by GitHub ([evidence](yaks-6c65/render-evidence.md), `gh api markdown` plus the live page of [yaks-1151](yaks-1151) on the farm branch):

- A plain note header is an ordinary paragraph: no anchor at all.
- A note with meta lines is read by markdown as a setext heading (the line above a `---`), swallowing the meta into the heading text. Its anchor is `user-content--2026-11-11t211311z-sched-1kind-movefrom-hairyto-shaving`: it changes with the metadata and nobody could type it. 1,034 of 8,404 notes in the census are like this.
- A note link `[x](yaks-1a2b#2026-11-04T22:13:03Z)` is dropped entirely by GitHub: a `:` before any `/` reads as a URL scheme.
- The `▸` marker is yaks-specific, so no other tool shows a note as a unit.

## Candidates

Each rendered on GitHub (attached output):

| | Shape | Anchor | Renders as | Verdict |
|---|---|---|---|---|
| A | `#### 2026-11-04T22:13:03Z · actor` then the text | slug of the whole heading | a titled block | actor leaks into the slug |
| B | `#### 2026-11-04T22:13:03Z` then an italic line `actor · kind: move · from: hairy`, then the text | `2026-11-04t221303z`, `-1`, `-2` for the same second | a titled block, outline-friendly | **recommended** |
| C | `<a id="n-...">` then a bold timestamp line | explicit id, `user-content-` prefixed on GitHub | a bold line, no outline entry | GitLab strips the `id` |
| D | `### <ts>` then a metadata table | slug of the heading | a table per note | heavy: 8,404 tables, and a table cannot hold a value with a pipe or a newline |

### Why B

- **The anchor is the timestamp**, the stable identity a note already has. GitHub's slug of `2026-11-04T22:13:03Z` is `2026-11-04t221303z` (hyphens kept, colons dropped, lowercased), and its duplicate-heading suffix `-1`, `-2` is exactly the rule `links::note_anchor` already uses for notes written in one second. Hosts that slug the same way (GitLab, Gitea; Azure DevOps differs and is untested) all work.
- **The actor stays out of the slug**, so a link does not break if an actor is renamed and a person can type it.
- **The write side builds the link**: `yaks update --note` prints the note's link already (`/<herd>/yaks/<id>.md#2026-11-04t221303z`), so nobody types a slug.

### The metadata line

One italic line under the heading: `*actor · key: value · key: value*`. The keys are today's (`kind`, `from`, `to`, `commit`, `branch`, `thread`, `for`, any `--meta`). Values stay single-line scalars, quoted like frontmatter values when they contain ` · ` or start with a quote. The line is parseable (split on ` · `, then on the first `: `) and legible raw and rendered. A note with neither actor nor metadata has no line.

```markdown
#### 2026-11-04T22:13:03Z

*linkplan-lead · kind: move · from: hairy · to: shaving*

moved: hairy -> shaving
```

### Separators

The `---` before each note goes: the heading is the separator, and a stray `---` is what made the meta notes into setext headings. Heading level `####` sits under a description that uses `##` and `###`; notes read as the last section's children in an outline, which is acceptable. The alternative, a single `## Notes` heading over them, is the same cost and adds one more thing to keep.

## Parsing, and what the old shape needs

- **New parser:** a note starts at a line `#### <RFC 3339 timestamp>` (outside a code fence, which the old parser never had to care about) and runs to the next such line or the end. The following italic line, if it starts with `*` and ends with `*`, is actor and metadata.
- **Old shape stays readable** until `yaks migrate`, as every earlier schema: the reader accepts both, and a farm at schema 8 is only ever written in the old shape. Mixed notes in one body are not produced; the migration converts a whole yak at once.
- **Touch points** (from the audit): `store::parse_notes`, `append_note_meta`, `append_entry`, `note_head`, `split_note_head`, `parse_meta`; `mergedriver.rs` (its note-aware merge splits bodies by note); `tui/content.rs` (comment assembly and editing); `sheds.rs`; `links::note_anchor` and `find_note`; `markdown_lint.rs`; the docs and skills that describe the format.
- **A description that happens to hold a `#### <timestamp>` heading** is the one new ambiguity. It cannot be told from a note. Accept it: nothing in the farm has one (checked), and the lint can warn.
- **Rendering inside the TUI:** `tui/content.rs` already draws the header from parsed fields, so the TUI looks the same and only the file changes. A cleaner raw view is a bonus.

## Rejected

- **Keeping `▸`** and adding an id: no standard markdown construct gives the note an anchor without HTML, and GitLab strips the HTML.
- **A heading with the actor in it (A):** the slug then depends on the actor's name and its punctuation.
- **A table (D):** most notes have no metadata, so most would render as a bare heading and a few as tables, which is uneven, and a table cell cannot hold what a note value may.

## Open questions

- [ ] Heading level: `####` (lean) or `###`? A one-line change either way.
- [ ] Is the separator ` · ` acceptable in the italic line, or should it be a pipe-free ASCII form (` | `)? ` · ` reads better; a value can never contain it unquoted.
- [ ] Azure DevOps slugs headings differently and is untested: acceptable?
