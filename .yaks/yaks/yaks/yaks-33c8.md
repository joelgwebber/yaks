---
id: yaks-33c8
title: 'Idea: model-driven reformat of legacy yak bodies for CommonMark'
status: shorn
type: idea
priority: 4
created: '2026-10-17T04:25:58Z'
updated: '2026-10-26T22:06:22Z'
labels:
- markdown
---

With CommonMark soft breaks (yaks-e9de), bodies written one-item-per-line (`Decision: …` / `Evidence: …` runs, pasted output) render as run-on paragraphs. A survey of this farm on 2026-10-17 found ~440 such adjacent plain lines vs ~90 genuinely hard-wrapped prose lines. Later: a `yaks` command or skill recipe that has a model rewrite a body into lists / blank-line paragraphs / fenced blocks, shown as a diff for approval. Not now.

---
▸ 2026-10-17T19:55:43Z [delta-lead]

Moved out of the yaks-e9de arc (Joel: leave out of this release); stays hairy for later.

---
▸ 2026-10-26T20:59:03Z [delta-lead]

## Design pass (delta-lead, 2026-10-26)

Joel asked whether yaks should call a model itself (e.g. via LiteLLM), or hand requests like this back to the user's agent harness. The same question comes up in yaks-00c4 (import an issue, then tidy it).

### Option A: yaks calls a model

`yaks reformat <id>` sends the body to an OpenAI-compatible endpoint (LiteLLM, OpenRouter, a local server) and shows the diff.

- For: works from the TUI with no agent running; one keystroke.
- Against: yaks becomes a network client with credentials, an endpoint/model config surface, cost and nondeterminism inside a self-contained binary. It needs an HTTP/TLS dependency (the release matrix is C-free today). It also duplicates the agent the user already runs, which knows the repo, the skills and the yak's history.

### Option B: hand the request to the agent (recommended)

The farm already has a human-to-agent queue: `needs: agent` plus `yaks inbox --for agent`, which every agent reads at session start (the skill's step 1), and `pickup` clears. Today only `answer` writes into it. Add a **request**: a needs-agent note the human starts.

- CLI `yaks request <id> --note "tidy the markdown"` (or a TUI key; `a` on a yak that isn't blocked could offer "ask" or "request"). It sets `needs: agent` and records the request as an attributed note, the same shape as `answer`.
- **Canned requests** cover the common chores so nobody types a prompt: `--tidy` (reformat for CommonMark per the skill's Notes rules), `--summarize` (condense the notes into a summary note). Each is one sentence in the skill that tells the agent what to do. yaks-00c4's imported yak gets `--tidy` set automatically.
- The agent does it when it next runs: `yaks inbox --for agent` shows the request, the agent rewrites with `yaks update --description -`, and `pickup` closes it with a note. Approval is the diff the human already has: the farm is git (`yaks status`, `git diff` on the farm branch, or `yaks regrow`-style revert).
- **Finding candidates is deterministic, so yaks can do it:** a `doctor` check (or `yaks list --lint markdown`) flags bodies with runs of adjacent plain lines (the ~440 lines the 2026-10-17 survey found), and `yaks bulk ... --request tidy` queues them in one go.
- For: no network, no keys, no new dependency; works with any harness (Delta, Claude Code, Zed); the agent that does the work also follows the skills, so the output matches. The queue already exists and is tested.
- Against: nothing happens until an agent runs; the request sits in the inbox meanwhile (fine for chores like this).

### Option C: both

Ship B, and later let a configured command run the request (`request_runner: "claude -p ..."` in config), so a user who wants it automatic wires their own harness in. yaks still makes no model calls itself. Defer until someone asks.

### Lean

B. It reuses the existing ask/answer/pickup loop instead of adding a second integration, keeps yaks offline, and serves yaks-00c4's "tidy after import" too. Open questions for Joel: a new verb `request`, or `answer` without a prior ask? And is a canned-request list (`tidy`, `summarize`) worth it over a free-text note?

---
▸ 2026-10-26T20:59:03Z [delta-lead]

asked: needs human

Design fork, written up in the note above: should yaks call models itself (A, e.g. LiteLLM), hand chores to the user's agent through the existing needs: agent inbox (B), or B now plus an optional configured runner later (C)? Lean: B, with a new 'request' verb and canned 'tidy'/'summarize' requests; doctor flags candidate bodies deterministically. Need your call on A/B/C and on 'request' vs reusing 'answer'.

---
▸ 2026-10-26T21:46:24Z [Joel Webber]

Strongly agree on (B). I really like the idea of using the inbox as a general queue of "stuff that needs attention in some way", either from the user or agent. Then we can leverage the same affordances to ensure they get taken care of by whoever's responsible (eg, as described in yaks-f313).

---
▸ 2026-10-26T21:51:27Z [delta-lead]

picked up

Going with B (inbox-centric). Design for the build and how it meshes with yaks-f313 follows in the next note.

---
▸ 2026-10-26T21:51:27Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T21:51:27Z [delta-lead]

## Design: the inbox as the one queue of work waiting on someone (delta-lead, 2026-10-26)

Joel picked B: the inbox becomes the general queue of "stuff that needs attention", either from the human or an agent, and yaks-f313 lets whoever is responsible wait on it.

### The model: `needs` names who acts next

The field already says who acts next: `human` or `agent`. We add no new field. There are three ways in:

- `ask` → `needs: human` (an agent's question, unchanged).
- `answer` → `needs: agent` (the human's reply, unchanged).
- **new `request`** → `needs: agent`, with no prior question. A human (or a yakherd) asks for work on a yak: tidy it, summarize it, check something. It records a `requested: needs agent` marker note, like `ask` does.

`pickup` stays the only way to clear `needs: agent`, for answers and requests alike. The inbox's agent section becomes **"Awaiting an agent"**, and each row says why: `answered` or `requested` (derived from the latest marker note, never stored). JSON gets `"why": "answered"|"requested"`.

### Waiting on the queue (yaks-f313)

- `yaks wait [--for agent|human] [filters] [--timeout 10m] [--json]` blocks until the inbox for that audience is non-empty, then prints it like `inbox` and exits 0. A timeout exits 2 and prints nothing.
- `--new` ignores rows already present when it started (for a watcher that leaves some rows for later).
- It polls the farm files (cheap, local, no daemon). Harnesses with background tasks (Claude Code) run it in the background; in Delta, a foreground call with a timeout works.
- The mesh: requests and answers share `needs: agent`, so one `yaks wait --for agent` wakes an agent for either. A human-side tool (or a yakherd) can do `yaks wait --for human` the same way.

### TUI

- `a` stays context-sensitive: on a yak awaiting a human it answers. Otherwise it now **requests** (needs: agent) rather than asks, since the TUI user is the human, and handing work to an agent is the human's verb.
- Asking from the TUI moves to `Q`. It's rare, and `Q` is free.
- Both open the multi-line editor (yaks-ce5e).

### Not in this round (filed separately)

Canned requests (`--tidy`, `--summarize`), a `doctor` lint for run-on bodies, `bulk --request` and auto-tidy on import (yaks-00c4) all build on `request` and can follow.

### Work split

- `request-1` (child yak): the `request` verb, the inbox's `why`, `Farm`/store, TUI `a`/`Q`, CLI docs and help.
- `wait-1` (yaks-f313): `yaks wait`, CLI docs and help.
- delta-lead: the skills prose (how an agent treats a request, when to `wait`) and the landing.

---
▸ 2026-10-26T21:52:24Z [delta-lead]

Fan-out (delta-lead): request-1 on yaks-b75b (thread ksQQXwyj…y-WpIB…), wait-1 on yaks-f313 (thread ksQQXwyj…y-g5IB…), both on the profile default model, base 4069286. Skills prose stays with delta-lead, after both land. Follow-ups in yaks-db07.

---
▸ 2026-10-26T22:06:21Z [delta-lead]

Done (delta-lead): the inbox is now the queue of work waiting on someone. Pushed to the human's checkout as `696348e`.

- yaks-b75b (`request`, inbox `why`, TUI `a`/`Q`) landed as `25b9cb1`.
- yaks-f313 (`yaks wait`) landed as `655dacf`.
- Skills, `696348e`:
  - the `yaks` skill's "Asking a human" section is now "The inbox: work waiting on someone": ask, request, pickup and wait, plus how an agent treats a request;
  - `yaks-working` covers wait after an ask and requested rows;
  - `yaks-coordinating` covers `wait --for human --new` for workers' asks and `request` for chores.
- Gate: 866 passed, 0 failed. Installed with `cargo install --path .`.
- Follow-ups: yaks-db07 (canned tidy/summarize, a run-on lint, `bulk --request`, auto-tidy on import with yaks-00c4).

---
▸ 2026-10-26T22:06:22Z [delta-lead]

moved: shaving -> shorn
