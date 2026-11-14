---
id: yaks-f505
title: 'Link resolver and hosting layer: locate.rs, hosting.rs, yaks url'
status: hairy
type: feature
priority: 2
created: '2026-11-14T11:21:01Z'
updated: '2026-11-14T11:21:01Z'
parent: yaks-1151
tags:
- links
- cli
---

First piece of yaks-1151's plan ([/doks/design/farm-layout-and-links.md](/doks/design/farm-layout-and-links.md), section "The seam"). It does not change the on-disk layout or any stored link, so it ships on its own.

## Scope

- New `src/locate.rs`: a `Space` (farm root, the farm worktree's top and path prefix, the code checkout) and `resolve(Link) -> Place`: the one place for aliases and for the pinned-link fallback. From a `Place`: a local path (a pinned file gets the read-only temp copy `tui/source.rs` makes today), displayable content (what `peek` reads), and a web URL.
- New `src/hosting.rs`: a `Forge` (GitHub and GHES, GitLab, Gitea/Forgejo), detected from the remote via `store::normalize_remote_url`, with the URL templates in the dok. A custom domain declares its kind with `git config yaks.host`. Farm links use the farm repo's remote; `code:` links use the code repo's (they differ when `yaks.farm` shares another repo's farm). Move the "which remote" rule from `sync::remote_of` into `gitrepo.rs`.
- New `yaks url <link|id|path> [--pin]`: print the host URL. A farm file's permalink needs a farm commit the remote has; when the working farm is ahead of `origin/yaks`, fall back to the branch URL and say so on stderr. No recognised host: say why and name `yaks.host`.
- Shear the two real bugs in yaks-bede here (its note has the trace): `/sources/<id>` resolves to that source's `source.md` in `peek`, `doctor` and the TUI viewer (`follow.rs`), and `links::exists` stops skipping extensionless paths, so `doctor` reports a broken `/sources/<id>` or `/doks/sessions/...` link.
- Ship GitHub, GitLab and Gitea/Forgejo; Bitbucket and Azure DevOps are untested, so leave them out.

## Out of scope

The TUI menu (yaks-c3d0), any layout change, any change to what is stored. Do not write a new link form now.

## Evidence

Unit tests for every template and for remote-URL parsing (`git@host:o/r.git`, `ssh://`, `https`, GHES, GitLab subgroups, Azure DevOps ssh). A run of `yaks url` against this farm and a code file pinned at a sha, pasted in the shear note. `yaks peek /sources/<id>` working. `yaks doctor` flagging a broken `/sources/` link.

Docs in the same change: `docs/cli.md`, the clap help, the `yaks` skill's Links section, `yaks.host` in the config docs.
