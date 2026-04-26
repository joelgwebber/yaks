---
activation:
  - User asks to sync a yak with its external issue tracker
  - User invokes /yaks:sync
---

# Yak ↔ external tracker sync

This skill performs a **single-yak, bidirectional, plan-then-apply** sync between a yak and its external issue (Jira, Linear, GitHub Issues, etc.). It is intentionally manual. Automating bidirectional sync across arbitrary trackers is a fool's errand — the goal here is to make it *easy to keep one yak in sync when you want to*, not to run a daemon.

## Hard rules

- **Never touch the external tracker without confirming with the user first.** Every upstream write (field update, comment post, attachment upload, issue creation) is preceded by an explicit user-approved resolution in the sidecar.
- **When in doubt, do not sync.** A field, comment, or label whose mapping isn't clearly 1:1 should be left alone rather than guessed. False sync is worse than no sync — data loss is hard to reverse, missed drift is easy.
- **Never silently drop a local note or a remote comment.** If in doubt whether two items are the same, keep both — conservative duplication beats data loss.
- **Never annotate upstream content with yak-specific markers.** The user may be working in a shared tracker where yaks are a private tool. Comments ferried yak→external post as plain content, no `[yaks:…]` prefix. Only the local yak body may carry provenance annotations.
- **Never create an upstream issue automatically.** If a yak has no `source:`, ask the user whether to create one and *where* (project + issue type for Jira, team for Linear, repo for GitHub, etc.) before doing anything.
- **You must have the necessary MCP tools available.** If the yak's source points at a tracker whose MCP is not connected, stop and tell the user.

## The plan / apply / discard model

A sync is split into two phases separated by a durable artifact, the **sidecar** at `.yaks/.sync-pending/<yak-id>.yaml`:

- **Plan phase** — fetch upstream, compute the per-field diff, write a sidecar capturing every proposed change as a YAML structure with explicit `resolution` fields. **No mutations.** The user can inspect the sidecar (`yak sync show <id>`), hand-edit it (it's just YAML), or discard it (`yak sync clear <id>`).
- **Apply phase** — re-fetch upstream, verify the snapshot stored in the sidecar still matches, then process each item according to its resolution. On full success the sidecar is cleared; on partial failure it stays so the user can re-resolve.

This model exists because:
- Single-shot sync prompts strand the user mid-flow with each decision; they can't review the full picture before acting.
- A durable sidecar lets the user step away, hand-edit decisions, or queue many syncs and review in batch.
- Snapshot + re-fetch on apply gives atomicity: if upstream changed between plan and apply, we abort cleanly instead of overwriting fresh remote work.

When the user invokes `/yaks:sync TASK_ID`:

- **No sidecar exists** → run the plan phase. After writing the sidecar, summarize the proposed changes and tell the user to review (`yak sync show`) and then apply (`/yaks:sync TASK_ID` again).
  - **Exception**: if the plan contains zero `pending` items (i.e., everything is `auto` — purely silent application), you may apply immediately and clear the sidecar in the same invocation. Tell the user what was applied.
- **Sidecar exists** → run the apply phase. If snapshot drifted, abort and offer to re-plan.

## Sidecar schema

```yaml
yak_id: jira-301                   # local yak ID
source: https://...                # upstream URL
generated: 2026-04-25T18:00:00Z    # ISO8601 timestamp of plan phase

notes:                             # plan-output annotations the user should see
  - "priority: github source — local-only, not diffed"
  - "attachments: github source — manual ferry only"

upstream_snapshot:                 # all diffed fields as observed at plan time
  updated: <iso>
  status: <upstream status name>
  priority: <upstream priority name>
  title: <string>
  description: <string>
  labels: [<list of upstream labels, sans prefix>]
  comment_ids: [<list of upstream comment IDs that existed at plan time>]
  attachment_ids: [<list of upstream attachment IDs that existed at plan time>]

fields:                            # one entry per field that differs
  - name: priority
    local: 2
    upstream: 1
    direction: upstream            # which side wins for this field
    resolution: auto               # auto | pending | approve | skip
  - name: title
    local: "...local edit..."
    upstream: "...original..."
    direction: pending             # user must pick local or upstream
    resolution: pending

comments_up:                       # local notes not present upstream
  - body: "..."
    timestamp: <iso>
    resolution: pending            # default pending; user accepts or skips

comments_down:                     # upstream comments not present locally
  - author: "Clint Ayres"
    body: "..."
    timestamp: <iso>
    upstream_id: "458719"
    resolution: auto               # default auto; ferry is safe (no upstream write)

attachments_up:
  - filename: "..."
    size: 12345
    local_path: "..."              # absolute path the user can hand-upload
    resolution: pending

attachments_down:
  - filename: "..."
    size: 12345
    upstream_url: "..."
    resolution: pending            # default pending — many MCPs can't ferry these
```

`resolution` values: `auto` (silent default — apply without prompting); `pending` (needs explicit user decision before apply will touch it); `approve` (user has confirmed); `skip` (user has declined). When the user hand-edits the sidecar, they typically change `pending` → `approve` or `skip` per item.

`direction` (on `fields` items only) indicates which side wins: `local` ferries the yak value upstream; `upstream` overwrites the yak with the upstream value; `pending` means the user hasn't decided yet.

`notes` is an optional list of plain-text annotations describing things the user should know about the *plan itself* — not field-level decisions. The skill emits a note when:

- A field has been excluded from the diff because the tracker doesn't support it (e.g. priority on GitHub).
- A bucket can't be ferried via MCP and requires a manual hand-off (e.g. attachments on Jira/GitHub).
- The transport is degraded or cached (e.g. MCP unreachable; plan generated from snapshot).
- A normalizer was applied during diff (e.g. Linear markdown canonicalization).

Notes live only in the sidecar — they do not write back to the yak. The TUI surfaces them above the field table.

## Plan phase

1. **Resolve the yak.** Read the yak file. If it has no `source:`, jump to *Creating a new upstream issue*. Otherwise parse the URL to determine the tracker.
2. **Fetch the upstream issue.** Use the appropriate MCP tool to pull title, description, status, priority, labels, comments (with their IDs and timestamps), attachments, and the issue's own `updated` timestamp. See *Tracker hints*.
3. **Compute the diff per field policy** (see *Field mapping rubric*). For each diverging field, choose initial `direction` and `resolution` per its policy:
   - `priority` always: `direction: upstream`, `resolution: auto`.
   - `labels` (synced bucket): `direction: upstream` for new upstream labels, `direction: local` for new yak-side labels, `resolution: auto` either way (label changes are usually safe).
   - `title` / `description` / `status`: `direction: pending`, `resolution: pending` — user must decide.
4. **Bucket comments.** Yak comments live as blocks in the description body, fenced by a thematic break and a sigil header:

   ```markdown
   ---
   ▸ <iso8601> [@<author>] [(from <tracker>:<key>)]
   <body>
   ```

   The `▸` sigil at column 0 immediately after a `---` line is the unambiguous parse marker. A block extends from the `▸` line until the next `---\n▸` block or end of file. Hash-match yak comment bodies against upstream comments (normalize: strip headers, whitespace, lowercase). Anything matched is dropped. Yak-side leftovers go in `comments_up` with `resolution: pending`. Upstream-side leftovers go in `comments_down` with `resolution: auto` (external→yak ferry is a safe local write).
5. **Bucket attachments.** Local attachments live as `![alt](artifacts/<yak-id>/<filename>)` lines in the yak description body and as files under `.yaks/artifacts/<yak-id>/`. **Parse these `![...](artifacts/...)` lines out of the description before computing the description-diff** — otherwise every yak with a local attachment shows phantom description drift. Match `(filename, size)` between local artifacts and upstream attachments. Leftovers go in the appropriate bucket with `resolution: pending` (attachments are uneven across trackers — never auto-ferry).
6. **Write the sidecar.** Use the Write tool to put the YAML into `.yaks/.sync-pending/<yak-id>.yaml`. Include the `upstream_snapshot` so apply can detect drift later.
7. **Summarize for the user.** Tell them how many auto items, how many pending items, and how to review (`yak sync show <id>`) and apply (`/yaks:sync <id>`).

## Apply phase

1. **Load the sidecar** via `yak sync show <id>` (or read it directly).
2. **Re-fetch upstream** with the same fields used at plan time.
3. **Verify the snapshot.** Compare the current upstream against `upstream_snapshot` field-by-field. If anything changed: abort. Tell the user "upstream drifted since plan — discard and re-plan via `yak sync clear <id>` then re-run /yaks:sync." Do not apply anything.
4. **Apply each `auto` and `approve` item.**
   - Field with `direction: upstream` → update local via `yak.py update <id>` (or for description/comments, edit the body directly).
   - Field with `direction: local` → push upstream via the appropriate MCP write tool.
   - `comments_down` → append `---\n▸ <iso> @<author> (from <tracker>:<key>)\n<body>` block to the yak body.
   - `comments_up` → post via the tracker's add-comment MCP tool, content as-is, no provenance prefix.
   - `attachments_down` → download and place in `.yaks/artifacts/<yak-id>/`, append a body link. If the MCP can't fetch bytes, list the URL and stop — don't silently skip.
   - `attachments_up` → upload via the tracker's MCP tool. If the MCP can't upload (Atlassian, GitHub Issues), list the local path and stop — don't silently skip.
5. **Resolve `last_synced`.** This is the watermark for "yak and upstream agreed at this point" — future syncs use it to short-circuit when `upstream.updated <= last_synced`.
   - **Every item was `auto` or `approve`** — stamp via `yak.py update <id> --last-synced now`. No prompt.
   - **Any item was `skip` or `pending` (still unresolved)** — ask: *"Suppress remaining drift until upstream changes? [Y/n]"*. Default Y because the user just reviewed everything.
     - **Y / suppress** — stamp `last_synced` to now. The fast-path predicate short-circuits until upstream actually changes again.
     - **N / re-prompt next time** — leave `last_synced` alone. The denied drift will resurface next sync.
   - Always pass the literal `now` (not an explicit timestamp) so `last_synced` and `updated` end up aligned.
6. **Clear the sidecar** via `yak sync clear <id>`.
7. **Report.** Brief summary: applied locally / applied upstream / skipped.

## Discard phase

If the user wants to throw away the plan: `yak sync clear <id>`. Sidecar is gone, yak is unchanged, `last_synced` is unchanged.

## Creating a new upstream issue

If the yak has no `source:` and the user wants to push it upstream:

1. Ask *where*. This varies per tracker and there is no default:
   - **Jira:** project key + issue type (Bug, Task, Story, …).
   - **Linear:** team.
   - **GitHub Issues:** owner/repo.
   - **Others:** ask for whatever identifier the MCP tool requires.
2. Show the payload (title, description, type-mapped, priority-mapped) and confirm.
3. Create the issue. Capture the returned URL.
4. Stamp it into the yak: `yak.py update <id> --source <URL>`.
5. On subsequent syncs the yak has a source, so the normal plan/apply flow applies.

## Field mapping rubric

Don't treat any of these as hard rules — every tracker names things differently, and the agent is expected to use judgment. These are the patterns we've seen:

| Field | Default policy | Notes |
|-------|---------------|-------|
| title | direction: pending | Direct copy when accepted; user picks side. |
| description | direction: pending | Markdown / rich text; trackers vary, usually close enough. |
| status (hairy/shaving/shorn/dead) | direction: pending | Lossy: shaving ≈ In Progress, shorn ≈ Done, dead ≈ Won't Do / Closed-not-planned. Never auto-resolve. |
| priority | direction: upstream, resolution: auto | Priorities are subjective and frequently re-tuned by PMs/Engs upstream — never propose pushing the yak's priority. Accept upstream silently. |
| labels | resolution: auto for synced bucket | Synced labels are stored locally as `<tracker>-<name>` (e.g. upstream `bug-fix` → yak `jira-bug-fix`). Strip the prefix when ferrying yak→external; add it on external→yak. Bare labels (no `<tracker>-` prefix) are local-only and never ferry. Sidesteps schema conflicts (some trackers require pre-defined label sets) and keeps user-chosen taxonomies separate. |
| depends_on | direction: pending | Most trackers have issue-link semantics but link types vary. Best-effort; confirm per link. |
| comments / notes | comments_down: auto, comments_up: pending | External→yak ferry is a safe local write. Yak→external ferry must be confirmed. |
| attachments | always pending | Many MCPs can't ferry binary content; never auto-apply. Surface paths/URLs to the user when blocked. |

## Tracker hints

These are *hints* for efficient calls, not specifications. The agent should use whatever MCP is connected.

- **Jira (Atlassian MCP):** `getJiraIssue` returns fields, comments (with `created` and `updated` per comment), and attachment metadata in one call. `createJiraIssue` needs project key + issue type (use `getVisibleJiraProjects` + `getJiraProjectIssueTypesMetadata` to enumerate). `addCommentToJiraIssue` for ferrying notes up. Description is ADF — flatten to markdown for diffing and yak storage. Priority maps identity 1↔1: Highest → 1, High → 2, Medium → 3, Low → 4, Lowest → 5. **Attachments: read-metadata only** — the MCP exposes filename / mimeType / URL on read, but provides no upload tool and no content-download tool. Treat attachments as manual on this MCP.

- **Linear:** issue + comments are separate calls (`get_issue` and `list_comments`). Use the normalized `statusType` (not display name) when mapping status: `backlog` and `unstarted` → hairy, `started` → shaving, `completed` → shorn, `canceled` → dead. Priority maps Linear 0–4 → yak 1–5: `1` (Urgent) → 1, `2` (High) → 2, `3` (Normal) → 3, `4` (Low) → 4, `0` (None) → 3 (yak default — "no upstream priority set" is treated as medium). Auto-resolve upstream-wins per the rubric.

  Attachments are first-class — `create_attachment` (base64-encoded binary upload) / `get_attachment` (binary download) / `delete_attachment` all work, so on this tracker `approve` on an `attachments_*` row is safe. **Sweep gotcha:** Linear does *not* bump `issue.updatedAt` when an attachment is added or removed. `/yaks:sync-check` won't surface attachment-only drift; apply-time snapshot diff (which tracks `attachment_ids`) catches it on a per-yak sync. If a user reports "I added an image to a Linear issue, why isn't it surfacing in sync-check?" — explain this gap and ask them to run `/yaks:sync <id>` directly.

  **Markdown normalization (Linear-only).** Linear silently rewrites markdown on save: `_italics_` → `*italics*`, `- bullet` → `* bullet`, and probably more. Without normalization every sync against a Linear-sourced yak will surface phantom description drift. **Apply a post-read normalizer to *both* sides of the description (and comment bodies) at diff time only — leave the local yak file untouched.** Known rules: convert leading `-` bullets to `*`, convert `_x_` italic spans to `*x*`, collapse trailing whitespace, collapse repeated blank lines. If diff still flags drift after normalization, *show the user the diff*; never silently drop content.

- **GitHub Issues (no MCP available — shell out to `gh`):** there is no GitHub MCP yet, so the skill drives the `gh` CLI directly. Read shape: `gh issue view <N> --repo <owner/repo> --json number,title,body,state,labels,assignees,createdAt,updatedAt` and `gh api repos/<owner>/<repo>/issues/<N>/comments`. Write shape: `gh issue edit`, `gh issue comment`, `gh issue close`, `gh issue reopen`, `gh issue create`. Status is binary: per Linear's own docs, `unstarted/started → OPEN`, `completed → CLOSED`. We adopt the same: hairy/shaving → OPEN, shorn/dead → CLOSED. `shaving` is invisible upstream — call this out in plan output ("shaving is local-only on GitHub"). **Priority is not a GitHub concept** — exclude the priority field from the diff entirely when source is GitHub, and surface a one-line note in plan output: `priority: github source — local-only, not diffed`. Labels are repo-scoped *objects* `{name, color, description, ...}` — extract `.name` before namespacing as `gh-<name>`. **Attachments: no public API and no `gh` CLI surface** — image uploads in the GitHub web UI go through an undocumented IDP-protected endpoint, and the REST API only supports release-asset uploads, not issues. `gh issue comment` and `gh issue edit` only accept body text — no `--attach` flag exists. Treat attachments as manual: emit a `notes:` line `"attachments: github source — manual ferry only"`, leave any `attachments_*` row at `pending` (which the user can only resolve by `skip`), and surface local paths in the apply summary so the user can hand-upload via the web UI.

- **Anything else:** fetch everything up front, diff locally, confirm every write. Verify the MCP's attachment surface before assuming you can ferry.

## Sweep / drift check

Triggered by `/yaks:sync-check` or natural-language asks like "which of my yaks might need syncing?". This is **detection only** — never auto-plan, never auto-apply. The whole point is to surface candidates so the user can pick what to sync.

1. **Enumerate** source-linked yaks:

   ```
   yak sync check --json
   ```

   Returns a list of `{id, status, tracker, key, source, last_synced, local_updated, has_pending}`. Already classified by tracker; already aware of pending sidecars.

2. **Bucket by tracker.** Skip yaks where `has_pending: true` — they're already in the pipeline; surfacing them again as "might need syncing" is noise.

3. **Per-tracker batch query** for upstream `updated`:
   - **Jira** — one JQL call: `issuekey IN (KEY1, KEY2, ..., KEYN) AND updated > "<oldest_last_synced>"`. The `updated >` clause filters to yaks that *might* be drifted; client-side, compare each result's `updated` against that yak's specific `last_synced`. Fields: `["updated"]` is enough.
   - **Linear** — single GraphQL `issues(filter: { id: { in: [...] }, updatedAt: { gt: <iso> } })`.
   - **GitHub** — no batch endpoint for issues across multiple repos; iterate per repo.
4. **Classify each yak's drift:**
   - `upstream-newer` — `upstream.updated > yak.last_synced`. The tracker has changes you haven't pulled.
   - `local-newer` — `yak.updated > yak.last_synced`. You've edited the yak and haven't pushed.
   - `both` — both predicates fire.
   - `none` — fast-path: `upstream.updated <= last_synced` and `yak.updated <= last_synced`. No action needed.

5. **Report** as a concise table grouped by drift kind. Don't write sidecars; just tell the user what's worth their attention. They invoke `/yaks:sync <id>` per yak they want to plan.

If a yak's `last_synced` is missing (never synced), treat it as `upstream-newer` for reporting — first sync needs to happen.

## CLI gestures

- `yak sync ls` — list yaks with pending sidecars.
- `yak sync show <id>` — print a sidecar's YAML for inspection.
- `yak sync clear <id>` — remove a sidecar (after a successful apply, or to discard a plan).
- `yak sync check [--tracker X] [--json]` — enumerate source-linked yaks; input for the sweep flow above.

The plan, apply, and upstream-drift query are skill-driven (they need MCP access); the CLI is bookkeeping only. In `yak list` and the TUI, yaks with a pending sidecar render with a leading `~` so you can spot them at a glance.

In the TUI, `~` on a yak with a pending sidecar opens a review dialog where the user can cycle each field's resolution (approve / skip / pending) and apply locally. The dialog deliberately handles only the no-brainers: title / description / priority / labels with `direction: upstream`. Anything that needs an MCP write (`direction: local`, comments, attachments) or a lossy mapping (`status`) is shown read-only and the user is told to apply via `/yaks:sync`.

## Things this skill deliberately does *not* do

- No automation: even with sweep, the goal is to *identify* what may need syncing, not to act on it. The user always picks per-yak whether to plan, apply, or ignore.
- No silent dedup without a provenance marker upstream. Given the silent-upstream rule, lightly-edited comments may duplicate on next sync; that is preferred to dropping them.
- No opinion on whether `.yaks/` is committed. This skill works equally well whether yaks are in-repo or gitignored.
- No comment / attachment apply from the TUI sidecar dialog. Those buckets show up as counts only — the agent-driven `/yaks:sync` flow is what actually ferries them.
