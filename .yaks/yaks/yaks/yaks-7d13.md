---
id: yaks-7d13
title: 'Research: how GitHub and other hosts resolve and render farm markdown links'
status: hairy
type: task
priority: 2
created: '2026-11-11T21:53:55Z'
updated: '2026-11-11T22:10:12Z'
parent: yaks-1151
tags:
- links
- doks
---

Part of yaks-1151. Establish, with evidence, which link forms render and navigate correctly when a farm branch is browsed on GitHub, GitLab, Gitea/Forgejo, Bitbucket and Azure DevOps, so the link-form decision rests on facts.

[linkhost-evidence](yaks-7d13/linkhost-evidence.md)

---
▸ 2026-11-11T22:10:12Z [triage-1]

## Findings: how hosts resolve farm links (linkhost-1)

Raw output: [linkhost-evidence.md](yaks-7d13/linkhost-evidence.md). GitHub results come from experiments: `POST /markdown` with `context=joelgwebber/yaks`, the real blob views of `yaks` branch files, and curl probes. GitLab and Gitea/Forgejo come from their renderer source (`lib/banzai/filter/repository_link_filter.rb`, `sanitization_filter.rb`; gitea `modules/markup/render_link.go`, `renderhelper/repo_file.go`, `web_src/js/markup/anchors.ts`) plus live Codeberg pages. Bitbucket and Azure DevOps come from docs only.

| Behaviour | GitHub | GitLab | Gitea/Forgejo | Bitbucket Cloud | Azure DevOps |
|---|---|---|---|---|---|
| `/doks/x.md` | repo root at the **current ref** (also at a sha: `blob/<sha>/doks/...`) | repo root at the current ref (`path[1..]`) | repo root at the current ref | unverified | repo root (documented) |
| `x.md`, `sub/x.md` | relative to the file's directory | same | same | same (old bugs) | same |
| `../` inside the repo | works | works | works | buggy (BCLOUD-6589) | `./` documented, `../` unclear |
| `../` past the root | climbs URL segments (`/../main/x` lands on branch `main`, documented) | clamps at the root | clamps (`path.Join`) | ? | ? |
| `#heading` | works (`user-content-` id, `#slug` href) | works | works | toc ext | works, own slug rule (`:` becomes `-`) |
| `#L12-L40` on a `.md` | **no** highlight in the rendered view; needs `?plain=1#L12` | `?plain=1` (unverified) | `?display=source` (unverified) | — | — |
| link to a directory | `blob/dir` 301s to `tree/dir` | type lookup gives `tree` | `src` path | ? | ? |
| `%20`, `<a b.md>` | kept; a raw space breaks the link | unescaped, then re-escaped | kept | ? | ? |
| extensionless `yaks-1a2b` | `blob/.../yaks-1a2b`: **404**, or a 301 to the attachments dir when one exists | 404 | 404 | 404 | 404 |
| destination with `:` before any `/` (`yaks-1a2b#…:03Z`, `x.md#a:b`, `code:src/x.rs`) | **link dropped, shown as plain text** (read as a URL scheme). `./`, `/`, `sub/` or `%3A` keeps it | ? | allowed schemes only | ? | ? |
| images, relative or `/` | rewritten to `raw/<ref>/...` | `-/raw/` | `raw`/`media` | ? | relative and absolute documented |
| frontmatter | rendered as a table | shown as-is in a code box | `<details>` table by default (configurable) | ? | ? |
| explicit `<a id>` / `<a name>` | kept with a `user-content-` prefix; gfm mode lowercases the id; JS maps `#x` | `a` ids **stripped** (footnotes only) | kept with a `user-content-` prefix (backend for `id`, JS for `name`) | HTML not supported | limited |
| directory listing | truncated at **1,000** with a message (nodejs/node `test/parallel`: 4,185 entries omitted). Contents API capped at 1,000 | ? | ? | ? | ? |
| code search | **default branch only**: the farm branch is never indexed (repo search for a yak title returned 0) | ? | ? | ? | ? |

### Specific answers

- **The note heading slug.** `## ▸ 2026-11-04T22:13:03Z [actor]` gives `-2026-11-04t221303z-actor`. Today notes are paragraphs, not headings, so they have **no anchor on any host**. A stable target would be `<a id="n-20261104t221303z"></a>` (lowercase, no colons). It works on GitHub and Gitea, but GitLab strips it. A heading such as `#### 2026-11-04T22:13:03Z` slugs to `20261104t221303z` on GitHub, GitLab and Gitea, but ADO gives a different slug, and it changes the on-disk format.
- **Ref preservation.** On GitHub, a `/x` link viewed at `blob/<sha>/...` stays at that sha (verified on yaks-0079). Root-absolute links therefore pin correctly when someone reads a pinned farm commit.
- **A README at the farm root.** GitHub renders `README.md` in any directory listing (verified: `tree/main/docs` carries a `readme` payload). Today `tree/yaks` shows only `.gitattributes` and `.yaks/`, so nothing useful appears. A `README.md` at the branch root would be the landing page for `tree/yaks`.
- **Large directories.** This farm has 881 files, which is fine. A farm with about 10k yaks gets a truncated listing, though each file still opens by URL. With no code search on that branch, readers can only find yaks through an index page. A committed generated index conflicts with the "any index is derived" invariant, so it needs a decision. Fanning out by status breaks the one-stable-path rule. An id-prefix fan-out (`yaks/1a/…`) keeps paths stable but changes the link form.

### Recommendation

**Drop the `.yaks/` root and use farm-root-absolute links (`/doks/x.md`, `/<herd>/yaks/<id>.md`) for everything outside the linking file's own subtree.** With that change, ccb1's rule (absolute from the root, relative only downward) already matches GitHub, GitLab, Gitea and ADO, which all read `/` as the repo root at the current ref. Root-absolute links survive a move of the linking file, and with no `../` there is no clamp-versus-climb divergence. I see **no reason to revisit ccb1 and allow `../`**: it adds breakage when files move, and past the root it behaves differently on GitHub than on GitLab and Gitea.

What still fails under that recommendation:

1. **Bare yak ids** (`yaks-1a2b`) 404 on every host. They must be stored, or migrated, as `/<herd>/yaks/<id>.md`. yaks can keep accepting bare ids as input.
2. **Note anchors** with colons are dropped as links on GitHub unless the path has a `/`. They need a path form plus an anchor target the file actually emits (above). GitLab will never support an explicit target.
3. **`#L` on `.md` targets** needs `?plain=1` on GitHub, so line anchors into doks don't work in a plain link.
4. **Local viewers.** `/` means the workspace or vault root. That is correct only when the farm worktree itself is opened (VS Code, Obsidian: unverified).
5. **Bitbucket** relative-link behaviour is unverified and historically buggy.

**`code:` links.** GitHub drops the `code:` scheme entirely: the text renders but there is no link. The GitHub-only `/../<sha>/src/x.rs` climb trick clamps on GitLab and Gitea, so it is not portable. Recommended: store a **host URL** with a pinned sha, `https://github.com/o/r/blob/<sha>/src/foo.rs#L12-L40`, and have yaks parse its own origin's URLs back into `Code{path,rev,anchor}`. A reader without yaks gets a working, pinned link. Unpinned links use `blob/HEAD/path` (verified 200) or the default branch. The cost is host-bound text: a repo rename or host move needs a `yaks` rewrite, as `yaks mv` does now. Keep `code:` as input sugar that yaks expands on write.

### URL templates (`ref` = branch or sha; GitHub accepts short shas and `HEAD`)

- **GitHub / GHE:** `https://{host}/{o}/{r}/blob/{ref}/{path}#L{a}-L{b}`. For `.md` add `?plain=1` before `#L`. Directory: `/tree/{ref}/{dir}`. Raw: `/raw/{ref}/{path}`. Farm file: `/blob/yaks/{path}`.
- **GitLab:** `https://{host}/{group/sub…}/{r}/-/blob/{ref}/{path}#L{a}-{b}`.
- **Gitea/Forgejo:** `https://{host}/{o}/{r}/src/branch/{branch}/{path}#L{a}-L{b}`, or `/src/commit/{sha}/{path}`.
- **Bitbucket:** `https://bitbucket.org/{ws}/{r}/src/{ref}/{path}#lines-{a}:{b}`.
- **Azure DevOps:** `https://dev.azure.com/{org}/{proj}/_git/{repo}?path=/{path}&version=GB{branch}` or `GC{sha}`, then `&line={a}&lineEnd={b}&lineStartColumn=1&lineEndColumn=1&lineStyle=plain`.

Parsing the remote: handle `git@{host}:{path}.git`, `ssh://[user@]{host}[:port]/{path}`, and `https://{host}/{path}[.git]`. For ADO ssh, `git@ssh.dev.azure.com:v3/{org}/{proj}/{repo}`. GitLab paths can contain subgroups, so keep the full path. Infer the host kind from known hostnames; enterprise domains need a setting (e.g. git config `yaks.host`). For `yaks.farm`, take the farm repo's remote for farm links and the code repo's remote for code links.

### Open questions

- Should note headers become headings, or should yaks emit `<a id>`? Either is an on-disk format change.
- Is a committed root `README.md` or index allowed under "any index is derived"?
- How should yaks configure the host kind for custom domains?

### Not verified

- Bitbucket and ADO behaviour (docs only).
- GitLab `?plain=1` and Gitea `?display=source` line anchors.
- Whether the GitHub client-side anchor match is case-sensitive.
- Obsidian and VS Code handling of `/` links.
- Listing limits on hosts other than GitHub.
