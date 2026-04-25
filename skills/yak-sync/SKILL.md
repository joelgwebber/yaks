---
activation:
  - User asks to sync a yak with its external issue tracker
  - User invokes /yaks:sync
---

# Yak ↔ external tracker sync

This skill performs a **single-yak, bidirectional, merge-with-prompts** sync
between a yak and its external issue (Jira, Linear, GitHub Issues, etc.).
It is intentionally manual. Automating bidirectional sync across arbitrary
trackers is a fool's errand — the goal here is to make it *easy to keep one
yak in sync when you want to*, not to run a daemon.

## Hard rules

- **Never touch the external tracker without confirming with the user first.** Every upstream write (field update, comment post, attachment upload, issue creation) is preceded by a prompt that shows the exact change.
- **When in doubt, do not sync.** A field, comment, or label whose mapping isn't clearly 1:1 should be left alone rather than guessed. False sync is worse than no sync — data loss is hard to reverse, missed drift is easy.
- **Never silently drop a local note or a remote comment.** If in doubt whether two items are the same, keep both — conservative duplication beats data loss.
- **Never annotate upstream content with yak-specific markers.** The user may be working in a shared tracker where yaks are a private tool. Comments ferried yak→external post as plain content, no `[yaks:…]` prefix. Only the local yak body may carry provenance annotations.
- **Never create an upstream issue automatically.** If a yak has no `source:`, ask the user whether to create one and *where* (project + issue type for Jira, team for Linear, repo for GitHub, etc.) before doing anything.
- **You must have the necessary MCP tools available.** If the yak's source points at a tracker whose MCP is not connected, stop and tell the user.

## Workflow

1. **Resolve the yak.** Read the yak file. If it has no `source:` field, jump to *Creating a new upstream issue* below. Otherwise, parse the URL to determine the tracker.

2. **Fetch the upstream issue.** Use the appropriate MCP tool to pull: title, description, status, priority, labels, comments, attachments, and timestamps. See *Tracker hints* below for efficient per-tool invocations.

3. **Diff structured fields.** Each field has its own policy — see the *Field mapping rubric* below for the full table. In short:
   - **title, description, status** — show a short local-vs-upstream view and ask the user which side wins. Status mapping is lossy across trackers; never auto-resolve.
   - **priority** — upstream-wins, no prompt. PMs/Engs re-tune priority frequently; the yak's priority is more "how I plan my queue" than shared truth.
   - **labels** — namespaced. Split each side's label list into "synced" (matching `<tracker>-*`) and "local-only" (everything else). Diff only the synced bucket; local-only labels never ferry. Strip the prefix on yak→external, add it on external→yak.

4. **Diff notes against comments.** Treat each `### <iso>` block in the yak's markdown body as one note. Normalize both sides (strip leading `### <iso>` headers on yak notes, strip whitespace, lowercase) and hash. Then:
   - **Identical hashes** → same item, skip.
   - **Close-but-not-identical** (one is a prefix of the other, or differs only by trivial edits) → prompt the user: merge / duplicate / skip.
   - **No match** → ferry across, confirming direction per item.

   When ferrying **yak → external**, post the note body as-is, with no provenance marker. When ferrying **external → yak**, append a block to the description body in the form:

   ```
   ### <iso> @<author> (from <tracker>:<issue-key>)
   <comment body>
   ```

   The `(from …)` footer only lives locally; it makes the origin obvious to future readers and lets subsequent syncs recognize re-imported comments.

5. **Diff artifacts (best-effort).** Match `(filename, size)` between local `.yaks/artifacts/<yak-id>/` and upstream attachments. Matches are skipped. Non-matches are ferried with confirmation. On name collisions with different content, keep both: local keeps its name, imported file gets a numeric suffix (`foo.png` → `foo.1.png`).

   **Reality check before you try:** attachment APIs are uneven across trackers and most MCPs don't expose them at all. Do not assume the operation is possible. Concretely (as of writing): the Atlassian MCP has no attachment upload or content-download tool; GitHub Issues has no public REST endpoint for image attachments; Linear has clean attachment mutations and is the only one of the three where ferry actually works programmatically. When the MCP can't do what's needed, **list the affected files explicitly** (filename + local path or upstream URL) and tell the user to handle the transfer manually — never silently skip an attachment, that hides a real divergence.

6. **Apply agreed changes.** Update the local yak via `yak.py update` (or by editing the file directly for body changes). Update the upstream via its MCP tool. Batch where you can; confirm any batch that exceeds ~3 changes.

7. **Resolve `last_synced`.** This is the watermark for "yak and upstream agreed at this point" — future syncs use it to short-circuit when `upstream.updated <= last_synced`.

   - **All proposed changes accepted (or none proposed)** — stamp via `yak.py update <id> --last-synced now`. No prompt; there's nothing residual to discuss.
   - **Any proposed change was denied** — ask: *"Suppress remaining drift until upstream changes? [Y/n]"*. Default Y because the user just reviewed everything explicitly.
     - **Y / suppress** — stamp `last_synced` to now. The fast-path predicate will short-circuit until upstream actually changes again.
     - **N / re-prompt next time** — leave `last_synced` alone. The denied drift surfaces again on the next sync.

   Always pass the literal `now` to `--last-synced` — that same `update` call also bumps `updated`, so `last_synced` and `updated` end up aligned and the predicate `local.updated > last_synced` doesn't fire spuriously on the next sync.

8. **Report.** End with a short summary: what changed locally, what changed upstream, what the user declined.

## Creating a new upstream issue

If the yak has no `source:` and the user wants to push it upstream:

1. Ask *where*. This varies per tracker and there is no default:
   - **Jira:** project key + issue type (Bug, Task, Story, …).
   - **Linear:** team.
   - **GitHub Issues:** owner/repo.
   - **Others:** ask for whatever identifier the MCP tool requires.
2. Show the payload (title, description, type-mapped, priority-mapped) and confirm.
3. Create the issue. Capture the returned URL.
4. Stamp it into the yak: `python3 ${CLAUDE_PLUGIN_ROOT}/scripts/yak.py update <id> --source <URL>`.
5. On subsequent syncs the yak has a source, so the normal workflow applies.

## Field mapping rubric

Don't treat any of these as hard rules — every tracker names things differently, and the agent is expected to use judgment. These are the patterns we've seen:

| Field | Usually easy? | Notes |
|-------|---------------|-------|
| title | yes | Direct copy. |
| description | yes | Markdown / rich text; trackers vary, usually close enough. |
| status (hairy/shaving/shorn/dead) | lossy | Map per-tracker: shaving ≈ In Progress, shorn ≈ Done, dead ≈ Won't Do / Closed-not-planned. Confirm with user before applying. |
| priority | upstream-wins | Priorities are subjective and frequently re-tuned by PMs/Engs upstream — never propose pushing the yak's priority. On a sync, accept the upstream value. If the user wants a stable local priority, that's a future "local-only field" feature. |
| labels | namespaced | Synced labels are stored locally as `<tracker>-<name>` (e.g. upstream `bug-fix` → yak `jira-bug-fix`). Strip the prefix when ferrying yak→external; add it on external→yak. Bare labels (no `<tracker>-` prefix) are local-only — they never travel upstream. This sidesteps schema conflicts (some trackers require pre-defined label sets) and keeps user-chosen taxonomies separate. |
| depends_on | hard | Most trackers have issue-link semantics but the link types vary. Best-effort; confirm per link. |
| comments / notes | per-item | See step 4 above. |
| attachments | per-item | See step 5 above. |

## Tracker hints

Keep this short and tracker-agnostic — the agent should use whatever MCP is connected. These are *hints* for efficient calls, not specifications.

- **Jira (Atlassian MCP):** `getJiraIssue` returns fields, comments (with `created` and `updated` per comment), and attachment metadata in one call. `createJiraIssue` needs project key + issue type (use `getVisibleJiraProjects` + `getJiraProjectIssueTypesMetadata` to enumerate). `addCommentToJiraIssue` for ferrying notes up. **Attachments: read-metadata only** — the MCP exposes filename / mimeType / URL on read, but provides no upload tool and no content-download tool. Treat attachments as manual on this MCP.
- **Linear:** issue + comments are usually one GraphQL query, `updatedAt` per comment. Attachments are first-class — `attachmentCreate` and friends work.
- **GitHub Issues:** issue body + comments are separate list calls. No native "issue status" beyond open/closed; map `shorn` → closed, otherwise open. Labels are global to the repo. **Attachments: no public API** — image uploads in the GitHub web UI go through an undocumented IDP-protected endpoint, and the REST API only supports release-asset uploads, not issues. Treat attachments as manual.
- **Anything else:** fetch everything up front, diff locally, confirm every write. Verify the MCP's attachment surface before assuming you can ferry.

## Things this skill deliberately does *not* do

- No sweep mode ("sync everything"). One yak at a time. (See yak-bf54.2 for a future "which yaks might need syncing?" view that uses `last_synced` as the predicate.)
- No silent dedup without a provenance marker upstream. Given the silent-upstream rule, lightly-edited comments may duplicate on next sync; that is preferred to dropping them. If this becomes painful in practice, revisit.
- No opinion on whether `.yaks/` is committed. This skill works equally well whether yaks are in-repo or gitignored.
