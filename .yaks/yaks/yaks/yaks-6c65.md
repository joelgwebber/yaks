---
id: yaks-6c65
title: 'Design: new note format with anchors that render on GitHub'
status: hairy
type: idea
priority: 2
created: '2026-11-14T11:21:01Z'
updated: '2026-11-14T11:21:01Z'
parent: yaks-1151
tags:
- links
- markdown
---

Decision (yaks-1151, Joel, 2026-11-14): reconsider the note format now and ship it in the schema-9 migration.

Today a note is `---`, then `▸ <ts> [actor]`, then optional `key: value` meta lines closed by `---`. On GitHub the header is a plain paragraph (no anchor), and a note with meta lines becomes a level-2 heading that swallows the meta, so its anchor is not the timestamp (census: 8,404 notes, 1,034 with meta; see yaks-0a4d). Nothing in the file gives a note a stable anchor, and GitHub drops `[x](yaks-1a2b#...T22:13:03Z)` entirely, because a `:` before any `/` reads as a URL scheme.

## What to settle

- A header that is a real heading, so the anchor exists on every host and the note renders as a titled block. Candidate: `#### 2026-11-04T22:13:03Z · actor` (the host research found `#### <ts>` slugs to `20261104t221303z` on GitHub, GitLab and Gitea, not on Azure DevOps). The other candidate is a visible `<a id>` target, which GitLab strips.
- Where the metadata (`kind`, `commit`, `branch`, `thread`, ...) goes so it renders legibly and stays parseable: a small table, a definition line, a fenced block, or plain lines.
- The link form for a note once there is an anchor, including the `-1` / `-2` suffix for notes in one second, and how a note link is written so GitHub keeps it as a link (path form plus the slug).
- How every reader copes: `store::parse_notes`, `append_note_meta`, the merge driver's note-aware merge (`mergedriver.rs`), `tui/content.rs` (comments), `sheds.rs`, `links::note_anchor`, the markdown lint. The old shape must stay readable by new yaks until `yaks migrate`, as every earlier schema did.
- Whether the file keeps the `---` separators (GitHub also renders a leading `---` block as the frontmatter table) or uses a plain rule.

## Deliverable

A short design dok (`doks/design/note-format.md`) with two or three candidate shapes rendered on GitHub with `gh api markdown` (attach the output), a recommendation, the parser and merge-driver impact, and a `yaks ask` on this yak for Joel. Do not implement.
