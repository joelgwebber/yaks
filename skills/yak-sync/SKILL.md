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

## Plan phase

1. **Resolve the yak.** Read the yak file. If it has no `source:`, jump to *Creating a new upstream issue*. Otherwise parse the URL to determine the tracker.
2. **Fetch the upstream issue.** Use the appropriate MCP tool to pull title, description, status, priority, labels, comments (with their IDs and timestamps), attachments, and the issue's own `updated` timestamp. See *Tracker hints*.
3. **Compute the diff per field policy** (see *Field mapping rubric*). For each diverging field, choose initial `direction` and `resolution` per its policy:
   - `priority` always: `direction: upstream`, `resolution: auto`.
   - `labels` (synced bucket): `direction: upstream` for new upstream labels, `direction: local` for new yak-side labels, `resolution: auto` either way (label changes are usually safe).
   - `title` / `description` / `status`: `direction: pending`, `resolution: pending` — user must decide.
4. **Bucket comments.** Hash-match yak `### <iso>` blocks against upstream comments (normalize: strip headers, whitespace, lowercase). Anything matched is dropped. Yak-side leftovers go in `comments_up` with `resolution: pending`. Upstream-side leftovers go in `comments_down` with `resolution: auto` (external→yak ferry is a safe local write).
5. **Bucket attachments.** Match `(filename, size)` between local `.yaks/artifacts/<yak-id>/` and upstream attachments. Leftovers go in the appropriate bucket with `resolution: pending` (attachments are uneven across trackers — never auto-ferry).
6. **Write the sidecar.** Use the Write tool to put the YAML into `.yaks/.sync-pending/<yak-id>.yaml`. Include the `upstream_snapshot` so apply can detect drift later.
7. **Summarize for the user.** Tell them how many auto items, how many pending items, and how to review (`yak sync show <id>`) and apply (`/yaks:sync <id>`).

## Apply phase

1. **Load the sidecar** via `yak sync show <id>` (or read it directly).
2. **Re-fetch upstream** with the same fields used at plan time.
3. **Verify the snapshot.** Compare the current upstream against `upstream_snapshot` field-by-field. If anything changed: abort. Tell the user "upstream drifted since plan — discard and re-plan via `yak sync clear <id>` then re-run /yaks:sync." Do not apply anything.
4. **Apply each `auto` and `approve` item.**
   - Field with `direction: upstream` → update local via `yak.py update <id>` (or for description/comments, edit the body directly).
   - Field with `direction: local` → push upstream via the appropriate MCP write tool.
   - `comments_down` → append `### <iso> @<author> (from <tracker>:<key>)\n<body>` block to the yak body.
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

- **Jira (Atlassian MCP):** `getJiraIssue` returns fields, comments (with `created` and `updated` per comment), and attachment metadata in one call. `createJiraIssue` needs project key + issue type (use `getVisibleJiraProjects` + `getJiraProjectIssueTypesMetadata` to enumerate). `addCommentToJiraIssue` for ferrying notes up. **Attachments: read-metadata only** — the MCP exposes filename / mimeType / URL on read, but provides no upload tool and no content-download tool. Treat attachments as manual on this MCP.
- **Linear:** issue + comments are usually one GraphQL query, `updatedAt` per comment. Attachments are first-class — `attachmentCreate` and friends work.
- **GitHub Issues:** issue body + comments are separate list calls. No native "issue status" beyond open/closed; map `shorn` → closed, otherwise open. Labels are global to the repo. **Attachments: no public API** — image uploads in the GitHub web UI go through an undocumented IDP-protected endpoint, and the REST API only supports release-asset uploads, not issues. Treat attachments as manual.
- **Anything else:** fetch everything up front, diff locally, confirm every write. Verify the MCP's attachment surface before assuming you can ferry.

## CLI gestures

- `yak sync ls` — list yaks with pending sidecars.
- `yak sync show <id>` — print a sidecar's YAML for inspection.
- `yak sync clear <id>` — remove a sidecar (after a successful apply, or to discard a plan).

The plan and apply phases are skill-driven (they need MCP access); the CLI is bookkeeping only. In `yak list` and the TUI, yaks with a pending sidecar render with a leading `~` so you can spot them at a glance.

## Things this skill deliberately does *not* do

- No sweep mode ("sync everything"). One yak at a time. (See yak-bf54.2 for a future "which yaks might need syncing?" view that uses `last_synced` as the predicate.)
- No silent dedup without a provenance marker upstream. Given the silent-upstream rule, lightly-edited comments may duplicate on next sync; that is preferred to dropping them.
- No opinion on whether `.yaks/` is committed. This skill works equally well whether yaks are in-repo or gitignored.
- No interactive TUI for sidecar review yet. Use `yak sync show` and a text editor — sidecars are plain YAML by design.
