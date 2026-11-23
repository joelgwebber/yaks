---
id: yaks-ca59
title: 'Writers store links the host can follow: canonicalise on save, pinned farm links as host URLs'
status: shaving
type: feature
priority: 2
created: '2026-11-23T00:18:30Z'
updated: '2026-11-23T01:25:55Z'
parent: yaks-e387
tags:
- links
- cli
---

Under yaks-e387 (TUI and GitHub resolve links the same way).

Two gaps between what yaks reads and what GitHub can follow, both in text written today:

1. **Bare yak ids are not canonicalised on save.** Decision 2 of [yaks-1151](/yaks/yaks/yaks-1151.md) (Joel, 2026-11-14): "Stored yak links become `/<herd>/yaks/<id>.md`; bare ids stay accepted and are canonicalised on save." [yaks-acc0](/yaks/yaks/yaks-acc0.md) did the one-time migration (`migrate::canonical_dest`) but no writer calls it: 17 `](yaks-1a2b)` / `](yaks-1a2b#note)` links have been written since, and every one 404s on GitHub.
2. **A pinned farm link (`path@<sha>`) is a yaks-only form.** GitHub reads `@f4211aaa` as part of the filename: the v0.0.41 release dok's hero image `<img src>` is `raw/yaks/doks/releases/v0.0.41/archive-browser.png@f4211aaa`, a 404 (curl; the unpinned URL is 200 image/png). Decision 4 of the same plan already settled this for pinned `code:` links: stored as the host URL, read back by `locate::Space::read_url`.

Want:

- One canonicaliser, applied by every farm writer as it saves. Move the yak and attachment rules of `canonical_dest` out of `migrate.rs` into the link layer, so the migration and the writers share it. Writers: `create`/`update` description and notes, `dok new`/`dok set`, the TUI editors (through the same `Farm` methods), `session note`, `attach --caption`. Idempotent; code spans and fences untouched (`links::rewrite`).
- A pinned farm link is stored as the farm remote's host URL at that commit: `raw/<sha>/<path>` for an image (what an `<img>` loads; `hosting::Repo::raw_url`), `blob/<sha>/<path>[#anchor]` otherwise. Only when the farm branch's remote is a recognised host and the farm is not private; else left as written. A pinned `code:` link likewise (decision 4; check whether its write side shipped).
- `Space::read_url` reads this farm's own host URLs back (blob and raw, at a sha) as pinned farm links, as it does for code. The farm and the code repo can share one GitHub repo (this one does): tell them apart by which repository has the commit, or by the farm remote when `yaks.farm` points elsewhere.
- `@sha` stays readable forever (old text).
- Docs: the yaks skill's link section and "Notes and descriptions", `docs/cli.md`, the notes line in `src/brief.rs`.

#### 2026-11-23T00:19:08Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-23T00:19:35Z

*herd-files*

Claim (herd-files): worker `hostlinks-1` (claude-sonnet-5-5), base `7c1452bd`, in parallel with `inlineimg-1` on yaks-bc4a.

**Scope:** the description's four points: one shared canonicaliser that every farm writer applies on save; pinned farm links stored as host URLs; `Space::read_url` reads the farm's own host URLs back; docs.

**Files you own:** `src/migrate.rs` (move the rules out), `src/links.rs`, `src/locate.rs` (`read_url` and its helpers), `src/hosting.rs`, the writer paths in `src/farm.rs` / `src/main.rs` / `src/session*.rs`, `docs/cli.md`, `docs/README.md`, the `yaks` skill's link sections, `src/brief.rs`'s notes line.

**Not yours:** anything under `src/tui/` (the TUI goes through `Farm` methods, so it gets canonicalisation for free) and `docs/tui.md`. Those belong to yaks-bc4a.

**Judge:** a reviewer can write `[b](yaks-1a2b)` and `![s](doks/x/s.png@<sha>)` through `yaks update --note`, `yaks dok set` and `yaks create`, then:

- read the stored form in the file: `/<herd>/yaks/<id>.md`, and the `raw/<sha>/` host URL when the farm has a GitHub remote;
- see `yaks peek` / `yaks url` still resolve both;
- confirm that a farm without a remote, or a private farm, keeps `@sha` as written.

**Evidence:**

- integration tests through the real binary for each writer;
- a `gh api markdown -f mode=gfm -f context=<o>/<r>` render of a canonicalised text, showing GitHub would follow the links. Attach the output; `gh` is authenticated on this machine, and the call has no side effects;
- the unit tests for `canonical_dest` keep passing.

**Decisions to record, not ask:** how to tell a farm URL from a code URL when both share one repo, and which writers you covered. **Ask (`yaks ask`) before** rewriting existing farm text in bulk: that is not in scope.

#### 2026-11-23T00:20:07Z

*hostlinks-1*

Start: Mon Nov 23 00:20:07 UTC 2026. Farm file: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-ca59.md`. Checkout: `/Users/joel/src/yaks/.delta/worktrees/rp3x6aw18wgt/yaks`.

[hl-gh-render](yaks-ca59/hl-gh-render.txt)

#### 2026-11-23T01:25:53Z

*hostlinks-1*

Decision: a farm URL and a code URL in one shared repository (this one) are told apart by which history has the commit: `Space::read_url` reads a `blob/` or `raw/` URL as a farm link when the commit is an ancestor of the farm branch (`git merge-base --is-ancestor`, memoised in `Space`), else as the code's. Rejected: telling them apart by path prefix (a farm path and a code path can coincide), and by the farm remote alone (it is the same remote when they share a repo). With a repository of its own the farm needs no test.

#### 2026-11-23T01:25:53Z

*hostlinks-1*

Decision: writers covered, all through one `Farm::stored` call (`locate::stored_text`) on only the text handed in, never the rest of the farm:

- `create` description, `update` description and note, `ask`/`answer`/`request`/`pickup` text, the `shorn --change` note;
- `dok new` body, `dok set` body and `--new`, `dok row set` cells, `session note` body.

Not covered, said so here: `attach --caption` (a markdown title, not a destination, so no link to canonicalise); the TUI's direct file editor (`tui/handlers.rs` `commit_attachment_edit`, inside yaks-bc4a's files; it writes through `doks::save`/`atomic_write`, not a `Farm` method), `release_dok_save`, `source_set`, `promote`, `rename_attachment`, which rewrite generated or existing text.

#### 2026-11-23T01:25:54Z

*hostlinks-1*

Decision: a pinned `code:` link is stored as a host URL too (decision 4 of yaks-1151; its write side had not shipped, only the reader). Same `host_form` path, on the code checkout's remote. Rejected: leaving `code:` pins as written, since they 404 on GitHub the same way and `read_url` already reads them.

#### 2026-11-23T01:25:55Z

*hostlinks-1*

Decision: one canonicaliser `links::canonical_dest` (yak and attachment rules moved out of `migrate.rs`); the migration keeps only its own namespace and source-slug steps around it, so its tests pass unchanged. Host form lives in `Space::host_form`, not `links` (it needs git facts and the memo). The writers call `locate::stored_text`, which does both rules in one `links::rewrite_kind` pass (a new variant that tells the callback whether the link is an image).

#### 2026-11-23T01:25:55Z

*hostlinks-1*

Decision: an image is stored as `raw/<sha>/...` and anything else as `blob/<sha>/...[#anchor]`, judged by the markdown syntax (`![..](..)`), not the extension. A pinned link to a directory, an unpinned link, a commit the checkout lacks, no remote, a local-path or unknown host, and a private farm all stay as written (never an error: the writer must not refuse a note for it).
