---
id: yaks-274c
title: 'Commits phase 1: Link::Commit and the host-URL reader (shared with code links)'
status: shaving
type: feature
priority: 2
created: '2026-11-18T03:02:45Z'
updated: '2026-11-18T14:40:13Z'
parent: yaks-3515
tags:
- links
- git
---

Phase 1 of yaks-3515; read its decision note first.

- `links`: a `Link::Commit { sha }`. Written as the code repo's host commit URL (`https://github.com/o/r/commit/<sha>`, plus the GitLab and Gitea forms in `hosting.rs`), or as `commit:<sha>` (7 to 40 hex). `hosting::Repo` gets a `commit_url`.
- **One reader** turns a URL on the code repo's own host into a yaks link: `/commit/<sha>` becomes `Link::Commit`, and `/blob/<sha>/<path>#L..` becomes `Link::Code`, which is also yaks-390f. It lives where the code checkout's remote is known (a `locate::Space` field is the lean), and every `Link::Code`/`Link::Commit` reader goes through it.
- `peek` on a commit link prints the subject, author, date and changed files. `doctor` treats a commit missing locally as unknown, not broken. `yaks url` takes a commit.
- Tests: parse each forge's URL form, a foreign repo's URL stays a URL, and a branch blob URL stays a URL.

#### 2026-11-18T14:26:23Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T14:40:13Z

*sources-triage · kind: landed · commit: b0a6d579b17168590399b8aad5193821946e6660 · branch: main*

Landed b0a6d579b17168590399b8aad5193821946e6660 on main: commit:<sha> link form (Link::Commit, code commits only); locate::Space::read_url reads the code repo's own host commit URLs as commit links and pinned blob URLs as pinned code: links (GitHub, GitLab, Gitea). Wired into peek, url (commit:<sha> -> host commit page, with the pushed check) and check's cited-code refs. peek commit: prints hash/author/date/message/files (links::commit_text, gitrepo::commit_info), reused by phase 2. Gate green; checked against this repo: peek commit:1091a53, peek/url of github commit and blob URLs, unknown sha exits 1 with a reason. TUI still shows commit links as artifacts: phase 2 (yaks-3f90).
