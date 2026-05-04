# Yak ↔ external tracker sync — design

Status: working design. Source of truth for the implementation; will seed
end-user docs once the bidirectional work (yak-bf54.11) lands.

## Goals

- Keep one yak in sync with one upstream issue (Jira / Linear / GitHub Issues / …) **on demand**, with explicit per-field user resolution.
- Bidirectional: pull upstream changes into the yak; push local changes upstream.
- Plan / apply / discard model: every upstream write is preceded by a durable, user-reviewable artifact — never a single-shot prompt.
- Conservative defaults: when in doubt, do not sync. False sync is worse than no sync.
- Cross-tracker correctness: per-tracker push capabilities are first-class data, not folklore in the agent prompt.

## Non-goals

- Daemon mode / continuous background sync. The user picks when to plan and when to apply.
- N-way sync across multiple trackers. One yak, one upstream.
- Custom-field mapping beyond title / description / status / priority / labels / comments / attachments.
- Lossy markdown ↔ ADF conversion for Jira description push (manual handoff for v1).
- TUI process making MCP calls directly. The TUI is the resolution editor; the agent (skill) is the apply engine.

## Components

| Component | Role |
|---|---|
| `scripts/yaklib/sync.py` | Sidecar IO, sweep helpers, local-apply enforcement, capability matrix consumers. |
| `scripts/yaklib/sync_caps.py` *(new, Phase 0)* | Per-tracker capability matrix — what's pushable per field/bucket. |
| `scripts/yaktui/sync_review.py` | TUI dialog: review the sidecar, toggle resolutions and directions, edit merged values, partial-apply locally. |
| `skills/yak-sync/SKILL.md` | Agent procedure for plan + apply phases, including all upstream MCP writes. |
| `.yaks/.sync-pending/<id>.yaml` | The sidecar — durable plan artifact. |

The split is load-bearing: the TUI never talks to MCPs (curses process, offline-friendly); the skill never edits the sidecar by hand at apply time (it reads the user's resolutions verbatim).

## Lifecycle

```
                    /yaks:sync <id>
                         │
                         ▼
                ┌─── plan phase ────┐
                │ (skill, MCP-aware)│
                └────────┬──────────┘
                         │ writes .yaks/.sync-pending/<id>.yaml
                         ▼
              ┌──── user reviews ─────┐
              │  • TUI ~ dialog       │
              │  • yak sync show <id> │
              │  • hand-edit YAML     │
              └─────────┬─────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
  partial apply   /yaks:sync <id>   yak sync clear <id>
  (TUI A key,     (skill apply,     (discard plan)
   local-only)    full bidirectional)
```

## Sidecar schema

```yaml
yak_id: jira-301
source: https://example.atlassian.net/browse/SUBTEXT-301
generated: 2026-04-25T18:00:00Z

notes:                              # plan-time annotations the user should see
  - "priority: github source — local-only, not diffed"
  - "description: jira source — local→upstream push disabled (ADF lossy)"

upstream_snapshot:                  # what apply re-checks against to detect drift
  updated: 2026-04-25T18:00:00Z
  status: "In Progress"
  priority: "High"
  title: "..."
  description: "..."
  labels: ["bug", "auth"]
  comment_ids: ["458719", "458801"]
  attachment_ids: ["att-1", "att-2"]

fields:                             # one entry per field that differs
  - name: title
    local: "fix login race"
    upstream: "Login race on stale session"
    direction: pending              # local | upstream | pending
    resolution: pending             # auto | pending | approve | skip
    capability: ok                  # informational; from sync_caps
    merged_value: null              # when set, supersedes the direction's source
  - name: priority
    local: 3
    upstream: 1
    direction: upstream
    resolution: auto
    capability: ok

comments_up:                        # local notes not present upstream
  - body: "..."
    timestamp: 2026-04-25T17:30:00Z
    resolution: pending
    merged_body: null

comments_down:                      # upstream comments not present locally
  - author: "Clint Ayres"
    body: "..."
    timestamp: 2026-04-25T17:45:00Z
    upstream_id: "458801"
    resolution: auto

attachments_up:
  - filename: "diagram.png"
    size: 12345
    local_path: "/Users/joel/src/yaks/.yaks/artifacts/jira-301/diagram.png"
    resolution: pending

attachments_down:
  - filename: "screenshot.png"
    size: 23456
    upstream_url: "https://..."
    resolution: pending
```

### Field semantics

- **`direction`**: which side wins for this field.
  - `upstream` → overwrite local with upstream value (or `merged_value`).
  - `local` → push local value (or `merged_value`) to upstream.
  - `pending` → user hasn't decided.
- **`resolution`**: whether the row is ready to apply.
  - `auto` → silent default; apply without prompting (used for upstream-wins on priority/labels).
  - `pending` → needs explicit user decision.
  - `approve` → user has confirmed.
  - `skip` → user has declined.
- **`merged_value`** / **`merged_body`** *(Phase 2)*: optional user-authored override. When present, it is the authoritative value to write — supersedes both `local` and `upstream`. Cleared if `direction` changes.
- **`capability`**: informational copy of the per-tracker capability for this field; rendered in the TUI as a dim chip so the user sees constraints inline.

### Notes (`notes:`)

Plan-time annotations describing things about the *plan itself*. Emitted when:
- A field is excluded from diff because the tracker doesn't support it (e.g. priority on GitHub).
- A field's `direction: local` push is disabled by capability (e.g. Jira description).
- A bucket can't be ferried via MCP and requires manual handoff.
- The transport is degraded (MCP unreachable, snapshot-only).
- A normalizer was applied at diff time (e.g. Linear markdown).

Notes never write back to the yak. The TUI surfaces them above the field table.

## Cross-tracker capability matrix

Defined in `yaklib/sync_caps.py`. Both the TUI dialog and the skill apply phase consult this — it's the single source of truth for "what can we push, where."

| Field / bucket | Jira | Linear | GitHub |
|---|---|---|---|
| `title` | ok | ok | ok |
| `description` | **lossy** (md→ADF) | **normalizer** (silent rewrites — see Linear hint) | ok |
| `priority` | ok (1–5 identity) | ok (0=None ↔ yak 3 default ambiguity) | **n/a** (excluded from diff) |
| `labels` | ok | ok | ok |
| `status` | **transition** (workflow lookup; may reject) | ok (state lookup) | **binary** (only OPEN/CLOSED; "shaving" invisible) |
| `comments_up` | ok | ok | ok |
| `attachments_up` | **manual** (no upload API) | ok (`create_attachment` base64) | **manual** (no API) |

### Capability values

- **`ok`** — round-trip safe. Push freely.
- **`lossy`** — push possible but data may degrade (e.g. markdown→ADF). TUI warns; user can proceed.
- **`normalizer`** — upstream rewrites silently; we apply the same normalization at diff time so round-trip is neutralized.
- **`transition`** — push requires a multi-step lookup (e.g. Jira workflow transitions). Skill handles; if no path, surface to user, don't fake.
- **`binary`** — lossy compression of state space (e.g. GitHub OPEN/CLOSED). Push works but local intermediate states aren't preserved upstream.
- **`manual`** — no API. TUI shows but apply emits a hand-off list (paths/text) and stops.
- **`n/a`** — concept doesn't exist upstream. Excluded from diff; never appears in `fields`.

## Plan phase (skill, agent-driven)

1. Resolve yak. If no `source:` and the user wants to push, jump to **Creating a new upstream issue**.
2. Fetch upstream via the appropriate MCP / CLI shell-out.
3. Compute per-field diff. For each diverging field, set initial `direction` and `resolution` per the field-mapping rubric.
4. Bucket comments. Yak comments live as `---\n▸ <iso8601> [@author] [(from <tracker>:<key>)]\n<body>` blocks in the description. Hash-match bodies (normalize: strip headers, whitespace, lowercase). Unmatched yak-side → `comments_up: pending`. Unmatched upstream-side → `comments_down: auto` (safe local write).
5. Bucket attachments. Local attachments live as `![alt](artifacts/<yak-id>/<filename>)` lines plus files under `.yaks/artifacts/<yak-id>/`. **Parse those lines out of the description body before computing the description-diff** or you'll get phantom drift forever. Match `(filename, size)`. Leftovers → appropriate bucket, always `pending`.
6. Emit `notes:` for capability-driven exclusions and manual handoffs (see capability matrix above).
7. Write the sidecar including `upstream_snapshot`.
8. Summarize for the user: N auto items, N pending items, "review with `yak sync show` or `~`, apply with `/yaks:sync <id>`."

**Fast path:** if every item resolves to `auto` (purely silent), apply immediately and clear the sidecar in the same invocation.

## Apply phase (skill, agent-driven)

1. Load the sidecar.
2. Re-fetch upstream. Compare against `upstream_snapshot` field-by-field. **If anything changed: abort.** Tell the user "upstream drifted — `yak sync clear <id>` and re-plan." Do not apply anything.
3. Process each `auto` and `approve` item per `direction`:
   - **Field, `direction: upstream`** → write `merged_value` (if set) or `upstream` to the yak via `yak.py update <id>`.
   - **Field, `direction: local`** → check `capability`; if `ok`/`lossy`/`transition`/`binary`, push `merged_value` or `local` via the tracker's MCP write. If `manual`, surface a hand-off line and skip the push.
   - **`comments_down`** → append `---\n▸ <iso> @<author> (from <tracker>:<key>)\n<body>` to yak body.
   - **`comments_up`** → push `merged_body` or `body` via add-comment MCP. **After successful push, rewrite the local comment header to include `(synced <tracker>:<comment-id>)`** so next plan's hash-match dedupes.
   - **`attachments_down`** → download bytes, write to `.yaks/artifacts/<yak-id>/`, append `![alt](artifacts/...)` line to body. If MCP can't fetch bytes, surface the URL and stop (don't silently skip).
   - **`attachments_up`** → if tracker capability is `ok` (Linear), upload via MCP. Otherwise (Jira, GitHub), surface the local path in the apply summary for manual upload.
4. Resolve `last_synced` (the watermark for "yak and upstream agreed at this point"):
   - All items `auto` or `approve` → stamp `last_synced=now` automatically.
   - Any `skip` or unresolved `pending` → ask: *"Suppress remaining drift until upstream changes? [Y/n]"*. Default Y. Y stamps now; N leaves it alone (denied drift resurfaces next sync).
5. Clear the sidecar.
6. Brief summary: applied locally / applied upstream / skipped / manual handoff list.

## TUI dialog (Phase 1 of bidirectional work)

Opens via `~` over a yak with a pending sidecar. Today it's read-only for anything beyond `direction: upstream` field rows on `{title, description, priority, labels}`. Bidirectional turns it into a **full sidecar resolution editor**.

### New affordances

| Key | Action |
|---|---|
| `↑ ↓ j k` | Navigate field/bucket rows. |
| `space` / `enter` | Cycle resolution (approve → skip → pending). |
| `s` | Set resolution `skip`. |
| `p` | Set resolution `pending`. |
| `d` *(new)* | Cycle direction (upstream → local → pending). Refuses for `n/a` capability. |
| `e` *(new, Phase 2)* | Open `$EDITOR` to author `merged_value`/`merged_body`. Two-up buffer if direction is `pending`. Pre-warns for `lossy`; refuses for `manual`. |
| `[` `]` *(new)* | Cycle within a multi-item bucket (comments, attachments). |
| `A` | Apply local-direction-resolved items now; persist the rest in the sidecar. |
| `D` | Discard the sidecar (with confirmation). |
| `q` / `Esc` | Save sidecar resolutions and exit. |

### Capability rendering

Each row shows a small dim chip with its `capability`. `lossy`, `manual`, `transition`, `binary` are visible inline so the user understands constraints without having to leave the dialog.

### Partial apply

`A` continues to apply only **local-direction, resolved, TUI-appliable** field rows (still gated by `_TUI_APPLIABLE_FIELDS = {title, description, priority, labels}`). Status, comments, attachments, and `direction: local` rows persist in the sidecar with the user's resolutions intact. Footer reports e.g. `"3 applied locally; 4 items need /yaks:sync to push upstream."`

The user then runs `/yaks:sync <id>` once at the end; the skill picks up resolutions verbatim and executes the upstream writes.

## Mutation gating (Phase 5 — warn-and-re-plan)

A pending sidecar represents a held conversation between the yak and upstream. Mutating the yak between plan and apply silently corrupts the sidecar:

- `direction: upstream` row → apply overwrites your intervening edit with upstream. **Lost.**
- `direction: local` row → apply pushes the **plan-time** local value upstream, not your fresh edit. **Silently dropped.**
- `pending` row → dialog shows stale local values. Confusing.

### The rule

When any mutation is attempted on a yak with a pending sidecar, prompt:

```
yak yak-bf54 has a pending sync plan (3 unresolved items).
Editing will invalidate the plan and discard it.
Continue? [y/N]:
```

- **Default N** — one keystroke aborts, sidecar untouched.
- **Y discards the sidecar** (with `.bak` left at `.yaks/.sync-pending/<id>.yaml.bak` for safety) and the edit proceeds.
- CLI escape: `--force-discard-pending` for scripting.

### Carve-outs (no prompt)

- **`slaughter`** auto-clears the sidecar silently. The yak is dying; the plan is moot.
- **`/yaks:sync` itself** — re-planning *is* the resolution path.

### Implementation

A single helper `confirm_discard_pending(root, yak_id) -> bool` called from each mutation entry point (`yak update`, `yak attach`, `yak shave`, `yak shorn`, `yak regrow`, `yak reparent`, plus TUI mutate paths). Returns True if the caller may proceed.

## Sweep / drift check

`/yaks:sync-check` and natural-language "which of my yaks might need syncing?" — **detection only, never auto-plans.**

1. Enumerate via `yak sync check --json`.
2. Skip yaks with `has_pending: true` (already in the pipeline).
3. Per-tracker batch query upstream `updated` (one JQL for Jira; one GraphQL for Linear; iterate per repo for GitHub).
4. Classify per yak: `upstream-newer`, `local-newer`, `both`, `none`.
5. Concise grouped report. User invokes `/yaks:sync <id>` per yak they want to plan.

## Per-tracker hints (compact reference)

Full hints live in `skills/yak-sync/SKILL.md`. Summary:

### Jira (Atlassian MCP)

- `getJiraIssue` returns fields + comments + attachment metadata in one call.
- Description is ADF — flatten to markdown for diff/storage. **Push back disabled (lossy).**
- Priority maps 1↔1 (Highest=1 … Lowest=5).
- Status push: `getTransitionsForJiraIssue` → `transitionJiraIssue`. Workflow may reject; surface to user.
- Comments: `addCommentToJiraIssue`.
- Attachments: read metadata only. No upload, no download bytes. Manual handoff.

### Linear

- `get_issue` + `list_comments` are separate calls.
- Use normalized `statusType`: `backlog`/`unstarted` → hairy, `started` → shaving, `completed` → shorn, `canceled` → dead.
- Priority maps 0–4 → 1–5; 0 (None) → 3 (yak default).
- **Markdown normalizer** at diff time: convert leading `-` bullets to `*`, `_x_` italics to `*x*`, collapse trailing whitespace. Apply to both sides; leave the local file untouched.
- **Sweep gotcha:** `issue.updatedAt` does not bump on attachment-only changes. `/yaks:sync-check` won't surface attachment-only drift; only per-yak sync's apply-time snapshot diff catches it.
- Attachments are first-class: `create_attachment` (base64) / `get_attachment` / `delete_attachment`.

### GitHub Issues (no MCP — `gh` CLI)

- Read: `gh issue view <N> --repo <owner/repo> --json ...` and `gh api repos/<o>/<r>/issues/<N>/comments`.
- Write: `gh issue edit`, `gh issue comment`, `gh issue close`, `gh issue reopen`, `gh issue create`.
- Status is binary OPEN/CLOSED. `shaving` maps to OPEN; this is invisible upstream — emit a `notes:` line.
- **No priority concept.** Excluded from diff; emit `"priority: github source — local-only, not diffed"`.
- Labels are repo-scoped objects — extract `.name` before namespacing as `gh-<name>`.
- **No attachments API.** Manual handoff only — emit `"attachments: github source — manual ferry only"`.

## Hard rules (carried from SKILL.md)

- Never touch upstream without confirming. Every upstream write is preceded by an explicit user-approved sidecar resolution.
- When in doubt, do not sync. Conservative duplication beats data loss.
- Never silently drop a local note or remote comment.
- Never annotate upstream content with yak-specific markers (`[yaks:…]` etc.). Comments ferried yak→external post as plain content.
- Never create an upstream issue automatically. Ask where (project, team, repo).
- Stop and tell the user if the required MCP isn't connected.

## Out of scope (v1)

- Markdown→ADF for Jira description push.
- TUI making MCP calls directly.
- Three-way diff3 viewer (`merged_value` covers practical merging).
- Custom-field mapping.
- Multi-tracker N-way sync.
- Daemon / continuous sync.
- Sweep-mode enhancement to surface Linear attachment-only drift (would need separate per-yak `attachment_ids` snapshot file).

## Implementation order (yak-bf54.11)

| Phase | Scope | Notes |
|---|---|---|
| 0 | Capability matrix (`sync_caps.py`) | Foundation. Both TUI and skill consume it. |
| 4 | Plan-time `notes:` for capability gaps | Smallest user-visible win; ships independently. |
| 1 | TUI: direction toggle + bucket navigation + capability rendering | Sidecar becomes fully editable in TUI. |
| 2 | TUI: manual edit (`merged_value` / `merged_body`) | Adds `e` hotkey + sidecar fields. |
| 3 | Skill: bidirectional apply respecting `merged_*` + capability matrix + comment-up provenance stamp | Closes the loop. |
| 5 | Mutation gating (warn-and-re-plan + `slaughter` carve-out) | Independent; can ship any time after Phase 0. |
