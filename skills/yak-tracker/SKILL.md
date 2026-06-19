---
name: yak-tracker
description: Relate yaks to external issue trackers (Jira, Linear, GitHub Issues) as a one-way projection. Use when seeding a yak from an external issue, finding which external issues a set of yaks rolls up to, or drafting/posting a status update to an external ticket from a bucket of yaks.
activation:
  - User wants to seed or import a yak (or group) from an external issue or URL
  - User wants to know which external issues a set of yaks maps to (rollup), or which keys to put in a PR
  - User wants to draft or post a status update to an external ticket from a set of yaks
  - User asks to "sync" yaks with an external tracker (route to projection — there is no bidirectional sync)
---

# Yak ↔ external tracker — one-way projection

Yaks are a **private, fine-grained, fast** layer; external trackers (Jira,
Linear, GitHub Issues) are a **shared, coarse, slow** layer. The relationship is
a **one-way projection, not synchronization**: many yaks roll up to few external
issues, the pointer runs **yak → external** only (each yak's `source:` URL), and
**the external tracker never knows yaks exist.** There is no shared state, no
field merge, nothing to reconcile.

See `docs/design/sync.md` for the full rationale. This skill covers the three
things that need judgment; the rollup itself is just a CLI command.

## Hard rules

- **Never write to the external tracker without explicit confirmation.** Drafting
  a comment or ticket edit is fine; *posting* it requires a yes, every time.
- **Never annotate upstream content with yak-specific markers** (`[yaks:…]`, yak
  IDs, etc.). The user may work in a shared tracker where yaks are private.
- **Never create an upstream issue automatically.** Ask where first
  (project/team/repo).
- **Stop and tell the user** if the tracker tool you'd need isn't connected.

## 1. Rollup (read-only) — which issues do these yaks touch?

This is a plain CLI command, no MCP needed:

```
yaks rollup [filters]        # group yaks by the external issue they point at
yaks rollup --keys [filters] # just the distinct keys, for pasting into a PR body
```

A yak with no own `source:` inherits the nearest ancestor's, so stamping one
`source:` on an umbrella yak covers its whole subtree. Scope with the usual
filters (`--label pr-369`, `--status shaving`, `--parent-of yak-abcd`).

**PR-key helper:** before opening a PR, run `yaks rollup --keys` over the shipping
set and paste the key(s) into the PR description. The forge (GitHub/Jira) makes
the PR↔issue link natively — we never write yak IDs anywhere.

## 2. Import-once — seed a yak from an external issue

When the user points at an external issue and wants a yak (or a group) for it:

1. Fetch the issue's title/description via whatever tracker tool is connected
   (see read hints below). Flatten to markdown if needed.
2. Create or update the yak(s), recording the issue URL as `source:`:
   `yaks create --title "…" --source <url>` (or `yaks update <id> --source <url>`).
   For a group, stamp `source:` on the umbrella yak; children inherit it.
3. **Stop tracking.** This is "import once, then diverge" — there is no
   watermark, no later drift check. Don't promise to keep them in sync.

## 3. Outbound draft — status update from a bucket of yaks

When a bucket of work is done and the user wants to update the external ticket:

1. Use `yaks rollup` (and the yak bodies) to gather what shipped under each key.
2. **Compose** a concise status update / suggested ticket edit, in plain prose —
   no yak IDs, no `[yaks:…]` markers.
3. Deliver one of two ways:
   - hand the text to the user to paste, **or**
   - if a tracker write tool is connected, offer to post it directly — and only
     post **after explicit confirmation**.

## Per-tracker read hints

Only the read path matters (import, drafting). The agent uses whatever is
connected; these are conveniences.

- **Jira (Atlassian MCP):** `getJiraIssue` returns fields + comments + attachment
  metadata in one call. Description is ADF — flatten to markdown when seeding.
  To draft back: `addCommentToJiraIssue` / `editJiraIssue` (confirmation-gated).
- **Linear (MCP):** `get_issue` + `list_comments` are separate calls.
- **GitHub Issues (`gh` CLI):** `gh issue view <N> --repo <o/r> --json title,body,comments`
  and `gh api repos/<o>/<r>/issues/<N>/comments`. To draft back:
  `gh issue comment` / `gh issue edit` (text only; confirmation-gated).
