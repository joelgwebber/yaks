---
id: yaks-390f
title: 'Read a host URL for the code repo as a code link, then store pinned code: links as URLs'
status: hairy
type: feature
priority: 3
created: '2026-11-15T05:31:38Z'
updated: '2026-11-18T01:15:37Z'
parent: yaks-1151
tags:
- links
- cli
---

Split out of [yaks-acc0](/yaks/yaks/yaks-acc0.md) (the schema-9 migration), as Joel allowed on [yaks-6cf2](/yaks/yaks/yaks-6cf2.md) ("if we do need to handle that parsing case, defer it and leave a comment about unifying the resolution").

Decision 4 of yaks-1151 was: a pinned `code:path@sha#L12` is stored as the host URL `https://github.com/o/r/blob/<sha>/path#L12`, which yaks parses back into a code link. The first half is cheap (`hosting::Repo::file_url`, landed). The second half does not exist: `links::parse` is pure and does not know the code repo's remote, so a host URL parses as `Link::Url`. If the migration rewrote pinned code links before that exists, then:

- the TUI would open them in the browser, not in the read-only code viewer (`mdrender::link_target` maps `Link::Url` to the system opener);
- `yaks check`'s `code-moved` finding (`check::code_refs`) would stop seeing them;
- `doctor`, `refs` and the take-out menu would treat them as foreign URLs.

## Scope

A single place that turns a URL into a `Link::Code` when it names a file of the code repo's own host: given the remote (`gitrepo::hosting_repo` on the code checkout), `https://{host}/{o}/{r}/blob/{sha-or-branch}/{path}[#L..]` and the GitLab and Gitea forms in `hosting.rs` become `Link::Code { path, rev, anchor }`. Then every reader that matches `Link::Code` goes through it: `mdrender::link_target`, `check::code_refs`, `locate::Space::resolve`, `links::edges` (refs, doctor), `peek`. The rewrite step of the migration comes last, once all of them read both forms.

Open questions for whoever picks it up: where the remote lives for a pure caller (a `Space` field, set once, is the lean: `Space` already carries the code checkout); a branch ref (`blob/main/...`) is not a pin and should stay a URL; a URL for another repo stays a URL.

## Not in the schema-9 migration

Until this lands, the migration leaves `code:` links, pinned or not, exactly as written, and the legacy `../src/x` and `../blob/<sha>/x` forms as they are. They stay plain text on GitHub, as they are today; nothing in yaks changes for them.
