# Yak → external tracker projection — design

Status: **active design.** Supersedes the bidirectional sync design (yak-bf54,
slaughtered) and absorbs the field notes from the retired `sync-rethink.md`.

## The one idea

Yaks and external trackers are **two layers at different granularities**, linked by
a **one-way projection**:

```
   yaks (private, fine-grained, fast)
     │  many
     │  source: ──────────────►  external issues (shared, coarse, slow)
     ▼                            (no knowledge of yaks)
   PR  ── ticket key in body ──►  (GitHub/Jira native PR↔issue link)
```

Many yaks roll up to few external issues. The pointer runs **one way only —
yak → external**. The external side stays oblivious that yaks exist. There is no
shared state, so there is nothing to reconcile.

This is the honest relationship. Everything below follows from it.

## Why not bidirectional (the argument)

The previous design tried to keep one yak in sync with one upstream issue,
bidirectionally. That baked in two assumptions that don't survive real use:

1. **1:1 cardinality** — one yak ↔ one issue. In practice N:1 *is* the case
   (a real session mapped 12 active yaks → 2 Jira keys). A fine-grained local
   tracker and a coarse shared tracker are *supposed* to differ in granularity;
   that is the entire point of having both.
2. **Bidirectional truth** — both ends mirror each other. But status models don't
   align (hairy/shaving/shorn/dead vs. a Jira workflow vs. GitHub's binary
   OPEN/CLOSED), priority scales differ, Jira descriptions are lossy ADF, labels
   need namespacing, hierarchies have different shapes and rules (Jira sub-tasks
   can't nest; yaks nest arbitrarily), and attachments can't round-trip on Jira or
   GitHub. Every one of these is a patch over "two sources of truth for one datum."

A decisive structural fact: **provenance can't live upstream.** A hard rule
forbids writing `[yaks:…]` markers into a shared tracker, so the external issue
*cannot* hold the back-pointer. Bidirectional identity is therefore inherently
unanchored on one end. The one-way model isn't a compromise — it's the only side
that can hold a durable link.

The bidirectional engine that was built to manage all this (sidecars, a
plan/apply/discard lifecycle, a capability matrix, comment hash-matching with
provenance stamps, attachment ferrying, snapshot drift detection, mutation gating,
a TUI resolution editor — ~2,400 lines) was completed and then never used. Drop the
two assumptions and that entire surface evaporates.

## What this design deletes

Against the one idea, all of the following are unnecessary and are removed:

- Capability matrix — gone (we never push fields).
- Sidecars, plan/apply/discard, snapshot/drift, mutation gating, `last_synced` —
  gone (no shared state to protect).
- Status / priority / description / label field mapping + md↔ADF — gone (we don't
  mirror).
- Comment hash-matching + provenance rewrites — gone (we don't ferry comments to
  dedupe).
- Attachment round-trip — gone (nothing ferried by default).
- Hierarchy mirroring + the sub-task-can't-nest wall — irrelevant (we don't
  replicate structure).

The smell test for anything proposed in this area: **if a feature only exists
because two ends must agree, it's in the wrong design.** Keep only what survives
"the external tracker never knows yaks exist."

## The model

- **`source:`** is the yak→external pointer (already part of the frontmatter). It
  records the yak's *conceptual home* — the external issue this work belongs to.
- **Effective source via inheritance.** A yak with no own `source:` inherits the
  `source:` of its nearest ancestor (`parent.N` → `parent` → …). One stamp on an
  umbrella yak keeps a whole subtree on the rails. Inheritance is resolved at query
  time; nothing is written into descendants.
- **Many → one, and flat.** A bucket of yaks rolls up to one external issue. We do
  **not** map yak-tree levels to external-tree levels (umbrella↔story,
  bucket↔sub-task) — that quietly reintroduces structure-mirroring and the
  hierarchy mismatch. The mapping stays many→one and flat.
- **No field merge, ever.** Humans (or the agent on explicit request) author the
  external side at ticket granularity. Yaks are never the source of truth for
  external fields, nor vice-versa.
- **Two relationships, kept separate:**
  - *yak → conceptual home* (where the work belongs): `source:`.
  - *PR → shipping ticket* (where the work landed): the key in the PR body, native
    to the forge. We don't conflate them. A yak conceptually belonging to one
    ticket may ship in a PR filed under another; that's the PR's business, not the
    yak's. The yak records only its conceptual home.

## The surface (what we build vs. assist)

Everything here is **read-or-assist, never authoritative-write.** The only thing
worth real code is the rollup; the rest are skill prompts that lean on whatever
tracker tools happen to be connected.

### 1. Rollup report — `yaks rollup [filter]` (code)

A pure local query, no MCP, no network. Groups yaks by **effective source** and
lists which yaks sit under each external target. Answers "what external issues does
this set of yaks touch, and which yaks point at each?"

- Honors the standard `FilterSpec` flags (`--label`, `--status`, `--parent-of`, …).
- `--json` for scripting.
- `--keys` mode: print just the distinct external keys (e.g. `SUBTEXT-369`) for the
  selected set — the **PR-key helper**. Use before opening a PR to know which keys
  go in the body so the forge makes the PR↔issue link natively. We never write yak
  IDs anywhere.

### 2. Import-once — skill (no plumbing)

Optionally fetch an external issue's title/description to *seed* a yak or group,
then **explicitly stop tracking**. "Import once, then diverge." No watermark, no
drift. The agent fetches via whatever MCP is connected and calls the existing
`yaks create --source` / `yaks update --source`.

### 3. Outbound draft — skill (no plumbing)

When a bucket of work is done, *compose* a status update / suggested ticket edit
from the rollup output and the yak bodies. Delivery has two modes:

- the human pastes it, **or**
- the agent drops it directly on the external issue **if it has the tools** —
  always gated behind explicit user confirmation.

This is the only yak→external *content* path, and it is always a deliberate,
confirmed act — never automatic, never a mirror.

## Surviving hard rules

Most of the old rule list was about protecting shared state that no longer exists.
What remains:

- **Never touch the external tracker without explicit confirmation.** Every
  upstream write (a drafted comment, a ticket edit, a new issue) requires a yes.
- **Never annotate upstream content with yak-specific markers** (`[yaks:…]` etc.).
  The user may be working in a shared tracker where yaks are a private tool.
- **Never create an upstream issue automatically.** Ask where (project/team/repo).
- **Stop and tell the user if the required tracker tool isn't connected.**

## Per-tracker read hints (compact reference)

Only the **read** path matters now (import-once, drafting). Push/diff mechanics are
gone. The agent uses whatever is connected; these are conveniences.

- **Jira (Atlassian MCP):** `getJiraIssue` returns fields + comments + attachment
  metadata in one call. Description is ADF — flatten to markdown when seeding.
- **Linear (MCP):** `get_issue` + `list_comments` are separate calls. Status type
  and priority are readable but we don't map them.
- **GitHub Issues (`gh` CLI):** `gh issue view <N> --repo <o/r> --json ...` and
  `gh api repos/<o>/<r>/issues/<N>/comments`. For drafting back:
  `gh issue comment` / `gh issue edit` (text only; confirmation-gated).

## Open questions (deferred, not decided)

- **`label → source` registry.** This design uses per-yak `source:` + ancestor
  inheritance, which covers the observed N:1 ergonomics. If buckets that cut
  *across* the yak tree (e.g. `pr-*` labels) need their own home, a
  `label → source` map in config could be added later. Resist until a real case
  demands it — it's new config surface.
- **Recording the "shipped-in" ticket on the yak.** Currently no: the yak points
  only at its conceptual home; PR↔ticket is the forge's job. Revisit only if the
  forge link proves insufficient in practice.
