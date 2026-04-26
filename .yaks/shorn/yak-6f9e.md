---
id: yak-6f9e
title: Web UI pointing at git repo
type: idea
priority: 2
created: '2026-04-14T14:24:05Z'
updated: '2026-04-18T00:46:10Z'
commit: b698f33
---

You could run it locally, or host it with access to a github repo. Similar feature set to the TUI, but in a way that makes it available to other users. If allowed to be mutative, it would just commit to the git repo.

Extra-cool: Allow branch picker, so you can see what others are working on, as long as they're pushing yak updates to a working branch.

## Plan

### Goal
Single self-contained HTML file that renders a read-only yak board for any public GitHub repo. No build step, no server, no CI. Open the file (or host on GH Pages) and point it at a repo.

### Data fetching strategy
1. **Tree API** (1 API call per page load): `GET /repos/{owner}/{repo}/git/trees/{branch}?recursive=1` — returns all file paths. Filter client-side to `.yaks/{hairy,shaving,shorn,dead}/*.md`.
2. **raw.githubusercontent.com** (CDN, not rate-limited): Fetch each task file's raw content. Parse YAML frontmatter + markdown body client-side.
3. Branch list via `GET /repos/{owner}/{repo}/branches` (1 call) for the branch picker. Cache aggressively.

Total API cost: ~2 calls per page load. 60/hr unauthenticated is plenty.

### UI sketch
- Repo input bar at top: `owner/repo` + branch dropdown
- Three-column kanban: Hairy | Shaving | Shorn (dead hidden by default, toggle-able)
- Each card: priority badge, type icon, ID, title, label chips
- Click to expand: full description (rendered markdown), deps, timestamps
- Parent/child grouping: children nested under parents
- Filter bar: search, type, priority, label (mirror TUI filter semantics)
- URL hash stores repo/branch/filters so links are shareable

### Dependencies (all CDN, no build)
- **js-yaml** (~30KB) — YAML frontmatter parsing
- **marked** or similar (~20KB) — markdown rendering for descriptions
- Vanilla JS + CSS, no framework. Keep it one file if feasible (inline or single-script with CDN imports).

### Implementation phases
1. **MVP**: Single HTML file. Hardcoded or URL-hash repo. Fetch tree + raw files, render three-column board with cards. No auth, public repos only.
2. **Polish**: Branch picker, filter bar, card expand/collapse, parent/child nesting, shareable URLs.
3. **Future (out of scope)**: Optional GitHub token for private repos. Pseudo-mutative interface via PR creation (create yaks / add comments by opening PRs against the repo).

### Open questions
- Hosting: GH Pages from the yaks repo itself? Or just a standalone file users download?
- Styling: minimal custom CSS vs. a tiny CSS framework (pico, water.css)?
- Do we want periodic auto-refresh, or manual only?

---
▸ 2026-04-17T01:50:16Z
MVP implemented in docs/index.html. Single self-contained HTML file using pico.css, js-yaml, and marked from CDN. Fetches via GitHub Trees API (1 call) + raw.githubusercontent.com (CDN). Three-column kanban with parent/child nesting, click-to-expand modal with rendered markdown, branch picker, shareable URL hash. Tested against joelgwebber/yaks — loads 99 tasks successfully. Polish ideas: sort shorn by updated date, add filter bar, dark mode toggle.

![V2 UI: shaving tab with light header, bison logo, refresh icon](artifacts/yak-6f9e/v2-shaving-tab.png)

![V2 UI: shorn tab with slide-in detail panel, human-readable dates, commit link](artifacts/yak-6f9e/v2-detail-panel.png)

![After polish: bumped text, consistent badges, better contrast](artifacts/yak-6f9e/v2-polish.png)
