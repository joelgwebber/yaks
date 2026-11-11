---
type: topic
title: 'Farm layout and links: making the farm browsable on GitHub'
created: '2026-11-11T22:39:16Z'
description: Plan for dropping the .yaks root and changing stored link forms so yaks, doks, sessions and sources work on GitHub and similar hosts; read before changing the layout or the link grammar
tags:
- links
- design
---

# Farm layout and links: making the farm browsable on GitHub

Design for [yaks-1151](yaks-1151). Findings come from four read-only scouts: host behaviour ([yaks-7d13](yaks-7d13)), a code audit ([yaks-9714](yaks-9714)), a census of the farms on one machine ([yaks-0a4d](yaks-0a4d)) and a triage of the neighbouring yaks ([yaks-20dd](yaks-20dd)). Every number below is from those notes; the attached reports have the evidence.

## The finding in one paragraph

Dropping the `.yaks/` root fixes most of it, and costs no link rewrites: every farm-root link (`/doks/x.md`) then resolves on GitHub, GitLab and Gitea/Forgejo, because those hosts read a leading `/` as the repository root at the ref being viewed. By the census that takes authored internal links that work on GitHub from 51% to 74%. What is left is almost all **yak-id links** (453 of them, bare `yaks-1a2b` and `yaks-1a2b#note`): they are extensionless and relative, so they 404 on every host. `code:` links and note anchors are the other two forms GitHub cannot follow. Those three need a decision about the stored form, and since that means a schema bump anyway, the layout change and the link rewrite should ship as **one** migration.

## What each link form does on GitHub

| Form | Today | After dropping `.yaks/` | Fix needed |
|---|---|---|---|
| `/doks/x.md#heading`, `/sessions/...`, `/sources/<id>/source.md` | 404 (`.yaks` missing) | works | none |
| `notes/x.md`, `<id>/shot.png` inside that yak, bare sibling in an attachment | works | works | none |
| `<id>/file` from a dok or another herd | wrong directory | wrong directory | write `/<herd>/yaks/<id>/file` |
| `yaks-1a2b` | 404 | 404 | write `/<herd>/yaks/<id>.md` |
| `yaks-1a2b#2026-...T22:13:03Z` | not a link at all (a `:` before any `/` reads as a URL scheme) | same | path form, plus an anchor (see below) |
| `code:src/foo.rs@sha#L12` | plain text | plain text | host URL |
| `../src/foo.rs` (legacy, 39) | 404 | 404 | host URL, or leave |
| `/doks/sessions/...`, `/doks/sources/...` | 404 | 404 | wrong path, see below |
| `#L12-L40` on a rendered `.md` | no effect | no effect | needs `?plain=1`; ignore |

Evidence: [linkhost-evidence](yaks-7d13/linkhost-evidence.md) (`gh api markdown` with `context=joelgwebber/yaks`, live pages, and GitLab and Gitea renderer source). Bitbucket and Azure DevOps are documentation only.

## Decisions to make

Each has a lean; the asks are on [yaks-1151](yaks-1151).

### 1. Drop the `.yaks/` root

Lean: yes. The branch root becomes the farm root:

```text
README.md            static: explains the layout, written once by init/migrate
config.yaml  schema  schemas/  doks/  sessions/  sources/
<herd>/yaks/<id>.md  <herd>/yaks/<id>/  <herd>/doks/
.gitattributes  .gitignore
.yaks/schema         a stub holding the new schema number, see Compatibility
```

A static README is not a derived index, so it does not break "any index is derived". GitHub shows a `README.md` as the landing page of `tree/yaks`, which today shows only `.gitattributes` and `.yaks/`.

### 2. Stored form of a yak link

Lean: **`/<herd>/yaks/<id>.md`** (and `/<herd>/yaks/<id>/<file>` for an attachment from outside the yak), canonical on disk. A yak's herd is its id prefix, so the path is derivable. yaks keeps *accepting* a bare id on input and in the CLI, and the commands that write note and description text (`update --note`, `create`, the TUI editors, `attach`) rewrite a bare-id link to the canonical form as they save. The reader on GitHub gets a working link; the writer, human or agent, still types `[x](yaks-1a2b)`.

- Rejected: leaving bare ids. It is the friendliest to write and is the form 453 links use, but it is the reason the farm would stay half-dead on GitHub.
- Rejected: document-relative `../../yaks/yaks/yaks-1a2b.md`. It rewrites 1,109 links, breaks whenever a file moves, and reverses the no-`../` decision in [yaks-ccb1](yaks-ccb1), which the host research says to keep: GitHub lets `../` climb out of the repository into the URL, GitLab and Gitea stop at the root, and Bitbucket is buggy.
- Cost to accept: the stored text now encodes the layout. A future layout change is another migration. That is how schemas 4 to 8 already worked, and `yaks mv` already rewrites links.

### 3. Note anchors

Lean: defer, and file a yak. A note header `▸ <ts> [actor]` renders as a plain paragraph on GitHub, so no note has an anchor there, and the 1,034 notes with meta lines accidentally become a heading whose slug is not the timestamp. The two fixes both change the on-disk note format (a visible `<a id>` that GitLab strips, or a heading per note that Azure DevOps slugs differently), which touches the note parser and the merge driver. Until then a note link on GitHub lands on its yak, which is the right file; yaks itself still jumps to the note.

### 4. Code links

Lean: a **pinned** `code:path@sha#L12` is written as the host URL (`https://github.com/o/r/blob/<sha>/path#L12`) when the code repo's remote is a recognised host; yaks parses its own host's URLs back into a code link, so the TUI still opens them locally. An unpinned `code:` stays as is, since it means "the current file" and a URL can only say `blob/HEAD`.

- Why pinned only: the pin makes the URL stable, and it is exactly the case where the reader on GitHub cannot be pointed at a local checkout.
- Cost: the stored text names the host, so a repo rename or host move needs a rewrite.
- Not chosen: `/../<sha>/path` (works on GitHub only) and translating on display (no display on GitHub).

### 5. Sessions and sources: the `/doks/sessions/...` bug (yaks-bede)

Lean: **no alias**. Sessions and sources are siblings of `doks/` at the farm root, which is the layout the census found everywhere and the one a root-absolute link needs. [yaks-bede](yaks-bede)'s links are simply wrong paths (three links in two farms). Two real fixes:

- `links::exists` skips paths without a file extension, so `yaks doctor` never reports a broken `/sources/<id>` link; it should, and offer the nearest real path.
- `/sources/<id>` names a directory; the TUI viewer and `yaks peek` should open its `source.md`. On GitHub the same link opens the folder listing, which is a fine target.

The migration corrects the three links it can.

### 6. Large directories

Lean: do not shard. GitHub cuts a directory listing at 1,000 entries ("4,185 entries were omitted"), but every file stays reachable by URL and the farm branch has no code search to lose. This farm's `yaks/yaks/` holds 881 entries and will pass 1,000 soon; the largest elsewhere on this machine is smaller. Sharding (`yaks/<hex>/<id>.md`) keeps a yak's path stable for life, but because decision 2 stores paths, it would be a second link rewrite later. If a shard is wanted, now is the cheap moment; this is the one place the lean is least settled.

## The seam: one resolver, one host layer

The audit found three copies of the follow/open dispatch in the TUI (`detail_nav.rs`, two sites in `handlers.rs`) and no single place that maps a link to a place. The proposal:

- `links.rs` stays the pure grammar (parse, `Link`, anchors).
- New `locate.rs`: a `Space` (farm root, the farm worktree's top and path prefix, the code checkout). `resolve(Link) -> Place` is the one place for aliases, legacy `.yaks/` forms and the fallback for pinned links into pre-migration history. From a `Place`: a local path (a pinned file gets a read-only temp copy), displayable content (what `yaks peek` and the viewer of [yaks-c3d0](yaks-c3d0) show), and a web URL.
- New `hosting.rs`: a `Forge` (GitHub, GitLab, Gitea/Forgejo), detected from the remote with `store::normalize_remote_url`, and the URL templates below. The "which remote" rule moves from `sync::remote_of` into `gitrepo.rs`. A custom domain declares its kind with `git config yaks.host`. When `yaks.farm` shares another repo's farm, farm links use that repo's remote and code links use the code repo's.
- CLI: `yaks url <link|id|path>` prints the host URL (`--pin` for the commit form).
- TUI: one `follow(Place)`, and for [yaks-c3d0](yaks-c3d0) an `O` menu over any yak, dok, session, source or code link: `e` editor, `o` system open, `p` absolute path, `r` farm or repo path, `g` host link. Keys follow the case-tier rules, none of these write a yak. For a pinned `code:` file, "absolute path" means the working-tree path when the file is unchanged since the pin, else the read-only temp copy (the rule opening already uses).

### URL templates

`ref` is a branch or a commit.

| Host | File | Lines |
|---|---|---|
| GitHub, GHES | `https://{host}/{o}/{r}/blob/{ref}/{path}` | `#L{a}-L{b}` (for `.md`, `?plain=1` first) |
| GitLab | `https://{host}/{group/path}/{r}/-/blob/{ref}/{path}` | `#L{a}-{b}` |
| Gitea, Forgejo | `https://{host}/{o}/{r}/src/branch/{b}/{path}`, `src/commit/{sha}/{path}` | `#L{a}-L{b}` |
| Bitbucket | `https://bitbucket.org/{ws}/{r}/src/{ref}/{path}` | `#lines-{a}:{b}` |
| Azure DevOps | `https://dev.azure.com/{org}/{proj}/_git/{repo}?path=/{path}&version=GB{branch}` (`GC{sha}`) | `&line={a}&lineEnd={b}` |

Ship GitHub, GitLab and Gitea/Forgejo first (verified from their sources); Bitbucket and Azure DevOps only when someone can test them. A farm file's permalink uses a farm commit that the remote already has; if the working farm is ahead of `origin/yaks`, `yaks url` falls back to the branch URL and says so.

## Compatibility and the shared branch

- **Old yaks on a new branch.** An older binary never reaches its schema check: discovery fails first with "restore .yaks/". `min_version` cannot help, since it lives in the config the old binary no longer finds. A stub `.yaks/schema` holding the new number gives old binaries the existing "newer schema, upgrade" refusal. One file, removable after a few releases.
- **Concurrent writers.** The migration is about 1,240 file renames. Rebasing old-layout commits onto it (what `yaks sync` does) keeps edits to renamed files, but a new yak in a renamed directory conflicts, and a new file in a directory that did not exist at the base is left behind in `.yaks/`. So: everyone syncs first, the migration moves strays, and the human picks the moment, as the `yaks-coordinating` skill's schema-bump order says. File history survives (`git log --follow` crossed the schema-6 move); check `git show -M` on a rehearsal clone, since 1,240 renames is over git's default rename limit.
- **Pinned `@sha` farm links** into pre-migration commits need a `.yaks/` fallback in `links::read_at`, next to the existing pre-doks fallback.
- **Per-machine TUI state** (saved views, the starred working set, drafts) is keyed by the farm root's path, which changes from `<worktree>/.yaks` to `<worktree>`. Without a move, every user loses their stars on upgrade. The first run on the new layout copies the old state across.
- **`yaks check`** staleness restarts for every dok, since the last-commit date of a renamed file is the move.

## The work

```text
A  resolver + hosting seam, `yaks url`, doctor and peek fixes     M   no layout change, lands first
B  TUI view-first and `O` menu (yaks-c3d0)                          L   after A
C  layout: discovery and farm_git prefix (store.rs, init.rs)        M   after the decisions, lands first of the layout
D  layout: path classifiers, land guard, sync/status/weight users   M   after C, parallel with E and F
E  schema 9 migration: moves, strays, .gitattributes and LFS,       L   starts beside C, after the decisions
   link canonicalisation, stub schema, README, saved-state move
F  docs, bundled skills and clap help                               M   last
```

A and B deliver the "open in GitHub" and read-only view without touching the layout, and are useful on their own. C to F are one release, one schema bump.

The audit's size and file scopes are in [the audit](yaks-9714/audit.md); the touch-point inventory (`farm_git`'s `root.parent()`, `preflight::yak_id_of`, `weight::kind_of`, the land guard, the `-- .yaks` pathspec in `sync.rs`, the `.yaks/**` rules in `.gitattributes`) is the checklist for C to E.

## Open questions

- [ ] Do notes become headings or `<a id>` targets (decision 3)?
- [ ] Bitbucket and Azure DevOps behaviour is untested.
- [ ] How do Obsidian and VS Code resolve a leading `/`? It is the vault or folder root only when the farm worktree is the folder opened.
- [ ] Does GitHub match `#fragment` to anchor ids case-sensitively in the browser?
