---
id: yaks-8fb0
title: 'Document substrate: Task as a typed view over Fields; one parser and writer'
status: hairy
type: task
priority: 2
created: '2026-11-27T02:38:22Z'
updated: '2026-11-27T02:43:23Z'
parent: yaks-7e32
depends_on:
- yaks-3b08
- yaks-e61e
tags:
- doks
---

Section 6 of [the design](/doks/design/one-document-model.md#6-fields-schemas-and-tags): a `Document` (path, kind, `Fields`, body) as the one representation, with `Task` a typed view over its `Fields`. One parser, one writer, one quoting policy (`store::write::needs_quotes` vs `fields::plain_safe` today), one key order. The merge driver, JSON and filter read through it.

Gates: load and save every file of a copy of this farm with **zero diff** (no byte changes until schema 11); TUI startup bench at 10k yaks no slower (yaks-e61e). Builds on yaks-8384's model. Needs a design pass by the coordinator before delegating: the `Task` API surface and the fast path.

#### 2026-11-27T02:43:23Z

*herd-unify*

Design pass (yakherd `herd-unify`, 2026-11-27), before delegating. Waits for yaks-3b08 to land: both rewrite large parts of `farm.rs`.

## Shape

- `Task` keeps its typed fields as a parsed cache (the hot path reads `id`, `status`, `title`, `parent`, `tags` per row, and parsing on demand per access would cost the startup bench), and replaces `extra: Fields` with `fields: Fields` holding **every** frontmatter key in file order. `Task::parse` = `split_frontmatter` + `Fields::parse` + reading the built-ins out of it; `store::parse_task`'s own line loop, `unquote` and `parse_inline_list` go.
- Writes go through setters (`set_status`, `set_title`, `add_tag`, ...) that update the cache **and** `fields` in place. `write::render` becomes `"---\n" + fields.render() + "---\n" + body`. An edit then rewrites only the keys it set, as doks already do.
- A **new** yak is built in canonical key order (today's `write::frontmatter` order), so new files look as they do now.
- One quoting rule, already implied by `fields::render_scalar`: a `Value::Str` is quoted when YAML would misread it (`store::write::scalar`), a `Value::Plain` stays plain when it reads back as the same text. Built-in yak fields are strings, so they keep today's quoting. Document it in one place (fields.rs) and delete the second description.
- Body: kept as read, not trimmed, matching doks; a writer that changes the body normalises only what it wrote.
- `Document { path, kind, fields, body }` is the shared base; `Task` wraps one plus the cache. `Dok` becomes the same `Document` (it already is, minus the name).
- The merge driver's per-field `scalar!` macro over typed fields goes: merge `fields` with the existing `merge_fields`, then re-read the cache.

## Gates

- **Zero diff:** load and save every file of a copy of this farm through the new code with no edit; `git diff --stat` empty. Also: an edit of one field changes exactly that key's lines.
- **Startup:** TUI first frame on a generated 10k-yak farm no slower than before (yaks-e61e's numbers; regenerate the farm, the scout's `/var/tmp/yakscale` farms are gone).
- `cargo test -p yaks`, insta snapshots unchanged.

## Risks

- `labels:` is still read as `tags:` (schema 6). Keep reading it; schema 11 can drop it.
- `Task { .. }` literals: about 60 sites, mostly tests; a `Task::new(id, title)` builder keeps them short.
