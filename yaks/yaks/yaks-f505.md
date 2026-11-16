---
id: yaks-f505
title: 'Link resolver and hosting layer: locate.rs, hosting.rs, yaks url'
status: shorn
type: feature
priority: 2
created: '2026-11-14T11:21:01Z'
updated: '2026-11-14T11:57:55Z'
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

---
▸ 2026-11-14T11:21:30Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T11:21:30Z [linkplan-lead]

Claim: worker `locate-1` (Sonnet tier, per AGENTS.md's standing delegation; the scope is one outcome with tests that fail first). Owner of the new `src/locate.rs`, `src/hosting.rs`, the `yaks url` command in `src/main.rs` (its own subcommand region only), `gitrepo.rs` (the remote rule), `links.rs` (`exists`, the `sources/<id>` resolution) and `src/tui/follow.rs` (the viewer's `sources/<id>` case). Out of scope: layout, stored link forms, the TUI menu, `store.rs`. Judge: the coordinator re-runs the evidence on its own build; the worker shears nothing.

---
▸ 2026-11-14T11:22:00Z [locate-1]

Start Sat Nov 14 11:22:00 UTC 2026. yaks path yaks-f505: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f505.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/tqdpn9meejzv/yaks

---
▸ 2026-11-14T11:44:53Z [locate-1]

Milestone: hosting.rs (Forge/Repo/templates + tests), locate.rs (Space, Place, farm_file alias, pinned_file moved from tui/source.rs), gitrepo::remote_of (sync::remote_of is a thin caller), `yaks url` subcommand, links::exists/peek and follow.rs use the `sources/<id>` alias. Builds. Next: unit + integration tests, docs.

Decision: a private farm (`yaks.private`) refuses `yaks url` for farm files (it never leaves the machine, so no host link exists); alternative was to link anyway.

Decision: farm links use the remote of the farm worktree's branch (`gitrepo::remote_of`: `branch.<b>.remote`, else `origin`); a missing remote or an unrecognised host refuses with a message naming `yaks.host`. No guessing among several remotes.

Decision: without `--pin` a link names the branch (stable); `--pin` names HEAD when a remote branch contains it, else falls back to the branch URL and says so on stderr. A file not in the remote ref gets a stderr note rather than silence (link would 404).

[yaks-f505-url-evidence](yaks-f505/yaks-f505-url-evidence.txt "yaks url against this farm: a yak, an attachment, a dok, a pinned code: path")

---
▸ 2026-11-14T11:57:13Z [locate-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-14T11:57:20Z [locate-1]

Shear summary.

- `src/hosting.rs`: `Forge` (GitHub/GHES, GitLab, Gitea/Forgejo) and `Repo::from_remote` (via `store::normalize_remote_url`; `git config yaks.host` declares a custom domain; Bitbucket/Azure/local paths refused with a reason), URL templates, unit tests for every template and remote spelling (scp, `ssh://`, https, GHES, GitLab subgroups, Azure ssh).
- `src/locate.rs`: `Space`, `Place`, `farm_file` (the `sources/<id>` alias, one place), `pinned_file` (moved from `tui/source.rs`), `web_url`, `parse_target`; `content` and `local_path` are in place for the TUI follow (yaks-c3d0).
- `gitrepo::remote_of` holds the "which remote" rule; `sync::remote_of` is a thin caller.
- `yaks url <link|id|path> [--pin]` (own clap variant and dispatch arm).
- Bugs from yaks-bede: `links::exists` now checks extensionless paths in the `sources/`, `sessions/`, `schemas/` namespaces (and `doks/<ns>/`), `links::peek` and the TUI viewer (`follow.rs`, `read_doc` too) read `/sources/<id>` as `source.md`.
- Docs: `docs/cli.md`, clap help, `yaks` skill Links section, README and cli config note for `yaks.host`.

Evidence: `cargo test -p yaks` PASS (verify note above), `cargo fmt --check` clean, `tests/url.rs` (url flow, refusals, peek `/sources/<id>`, doctor flags `/sources/nope` and `/doks/sessions/...`), attached url output. Scratch-farm doctor run: `doks/t.md:L3: (/sources/nope) no such file in the farm`. Local-only: nothing pushed, nothing under `.yaks/` staged.

---
▸ 2026-11-14T11:57:21Z [locate-1]
kind: move
from: shaving
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAGEKLZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-14T11:57:55Z [linkplan-lead]
kind: landed
commit: 298b91ac7f0774fbc3a248c0f044d8fc96c81eee
source: b20c4a925485e6dc3141ef941f94560108e85d97
branch: main
---

landed: 298b91ac yaks-f505: locate.rs and hosting.rs, yaks url, /sources/<id> resolves in peek, doctor and the viewer
