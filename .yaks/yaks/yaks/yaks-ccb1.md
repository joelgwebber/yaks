---
id: yaks-ccb1
title: 'Design: link grammar across farm, code and commits; yaks peek'
status: hairy
type: idea
priority: 1
created: '2026-11-01T18:52:37Z'
updated: '2026-11-01T18:53:39Z'
parent: yaks-7708
labels:
- wiki
- links
needs: human
---

One link grammar for yaks and pages, covering farm documents, the code repo, commit pins and anchors, plus `yaks peek` to read just the span a link points at.

## Today

- Another yak: its bare full id (`yaks-1a2b`), resolved by `refs::resolve`.
- An attachment: `<id>/<file>`, relative to the yak file.
- Code: `../src/foo.rs#L12-L40`, meaning "from the repo root". Pinned: `../blob/<sha>/src/foo.rs#L12` (yaks-da5e; resolved in the code checkout since yaks-6c4f). Existing farms use these: 14 such links in this farm, 18 in the `joel` farm.
- None of the code links work on GitHub while viewing the farm branch, so GitHub compatibility is already gone for them.

## Proposal: `<target>[@<rev>][#<anchor>]`

| Target | Form | Example |
|---|---|---|
| a yak | its full id, bare in prose | `yaks-1a2b` |
| a farm document | path from the farm root, leading `/` (OKF's recommended form) | `/wiki/topics/x.md`, `/sources/2026-11-01-ab12cd34/source.md` |
| a nearby farm document | plain relative path | `../topics/y.md`, `yaks-1a2b/log.txt` (attachments unchanged) |
| the code repo | `code:` then a path from the repo root | `code:src/foo.rs#L12-L40` |
| a URL | as is | `https://...` |

- `@<rev>` pins any path target to a commit (7-40 hex digits after the last `@`): `code:src/foo.rs@3f2a9c1#L12`, `/wiki/x.md@91be0d2#setup`. A farm path's rev is a farm-branch commit, a `code:` rev a code commit, so the target says which history the sha belongs to.
- Anchors: `#L12` and `#L12-L40` for lines (GitHub's form); `#<heading-slug>` (GitHub's slug rule) for a markdown section.
- **The old forms stay valid forever** (`../src/...`, `../blob/<sha>/...`); `yaks doctor --fix-links` can rewrite them into the new form, but nothing forces it.
- Wiki to yak is the bare id, so pages and yaks reference each other the same way. `yaks refs --incoming` works for a page path as for an id, and `doctor` reports links that lead nowhere.

## Peek

`yaks peek <link>` prints only what the link points at: the line range, or the section under the heading (up to the next heading of the same or higher level), or a page's frontmatter and first paragraph when there is no anchor, read through `git show` when pinned. `--context N` widens a range. This is cheap context for an agent, and the TUI uses the same resolver for a preview pane on the selected link.

## Rejected

- Keeping `../` as "the repo root" from pages: from `wiki/topics/x.md`, `../` means a farm directory; one prefix can't mean both.
- `../blob/<sha>/` for pinned farm paths: it can't say which history the sha is in.
- A query string (`?rev=`): no renderer does anything useful with it, and it breaks as a file path.

---
▸ 2026-11-01T18:53:39Z [delta:farm-wiki]
kind: ask
needs: human
---

asked: needs human

1. **Link forms:** `/wiki/x.md` (farm root, leading slash, as OKF recommends), relative paths for neighbours, `code:src/foo.rs#L12-L40` for the code repo, and `@<sha>` before the anchor to pin any path (`code:src/foo.rs@3f2a9c1#L12`, `/wiki/x.md@91be0d2#setup`). The existing `../src/...` and `../blob/<sha>/...` stay valid forever. Lean: this. The cost is that `code:` and `@sha` render as dead links on GitHub, but code links already don't work there on a farm branch.
2. **Heading anchors** use GitHub's slug rule. Lean: yes.
3. **`yaks peek` with no anchor** prints frontmatter plus the first paragraph (a page) or the description (a yak). Lean: yes.
