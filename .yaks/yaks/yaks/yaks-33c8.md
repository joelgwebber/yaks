---
id: yaks-33c8
title: 'Idea: model-driven reformat of legacy yak bodies for CommonMark'
status: hairy
type: idea
priority: 4
created: '2026-10-17T04:25:58Z'
updated: '2026-10-26T20:59:03Z'
labels:
- markdown
needs: human
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
