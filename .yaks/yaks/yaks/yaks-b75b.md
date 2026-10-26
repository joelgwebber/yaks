---
id: yaks-b75b
title: 'request: hand a yak to an agent through the inbox (needs: agent without a prior ask)'
status: shaving
type: feature
priority: 2
created: '2026-10-26T21:51:27Z'
updated: '2026-10-26T22:01:00Z'
parent: yaks-33c8
labels:
- cli
- tui
---

---
▸ 2026-10-26T21:51:51Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T21:51:51Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `request-1`. Design: the latest design note on yaks-33c8.

**Goal:** a `request` verb that sets `needs: agent` on a yak with no prior ask, recording a `requested: needs agent` marker note (plus the text), shaped like `ask`.

- `Farm::request` in `src/farm.rs`, under `store::lock`. Add `store::request_text` / `is_request_text` beside `ask_text` / `pickup_text` in `src/store.rs`.
- On a yak already `needs: human`, refuse with a clear message: answer it instead.
- On a yak already `needs: agent`, append the request and keep the state.
- CLI `yaks request <id> [--note|--note-file] [--as]` in `src/main.rs`: the new variant goes right after `Pickup`, and its dispatch arm after `Command::Pickup`.
- Inbox: the agent section is retitled "Awaiting an agent (`yaks pickup <id>`):" and each row is marked `answered` or `requested`, derived from the latest ask/request marker note (`store::needs_why` or similar; derived, never stored). `json::inbox_array` gets `"why"`. You own `render_inbox`'s body and `Audience`'s doc comments.
- TUI: `a` on a yak not awaiting a human now opens a **Request** editor (the multi-line editor, like Answer). Asking moves to a new `Q` key. Update `open_ask_or_answer` (and rename it to fit), `EditAction` (add `Request`), `needs_edit`, the blockedit hints, `help_content` and `help_hint` if `a` is there, and the TUI detail line for `needs: agent` (say requested vs answered).

**Out of scope:** `yaks wait` (another worker, `wait-1`, adds a `Wait` variant after `Inbox` in `src/main.rs` and calls `render_inbox` unchanged by signature: keep `render_inbox(rows, audience, json)` as it is). The skills (delta-lead does them). Canned requests (yaks-db07).

**Docs:** `docs/cli.md` "Human-in-the-loop" (a new `### request` and the inbox text), `docs/tui.md` (`a`, `Q`, inbox), `README.md` command table if ask/answer are there. `--help` doc comments.

**Evidence:**
- Tests that fail first: a farm test for request on no-needs, on human (refused) and on agent; inbox `why` in text and JSON; a TUI test that `a` on a plain yak saves a multi-line request (`needs: agent`) and that `Q` asks; an insta snapshot of the request editor.
- A real-binary transcript (request, inbox, pickup) pasted in a note, and a headless TUI frame attached.

**Gate:** `cargo test`. **Judge:** delta-lead at landing.

---
▸ 2026-10-26T21:52:29Z [request-1]

Started (request-1): `Mon Oct 26 21:52:25 UTC 2026`.

`yaks path yaks-b75b`: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-b75b.md`

[yaks-b75b-tui-frames](yaks-b75b/yaks-b75b-tui-frames.txt)

---
▸ 2026-10-26T22:00:45Z [request-1]

Decision: the inbox `answered`/`requested` mark lives in `fmt_row` (and `fmt_plain_row` for `next`), replacing the old fixed `✓ answered (needs:agent)` suffix with `✓ <why> (needs:agent)`.

- Rejected: a separate marker column appended only in `render_inbox`; it would print `answered` twice on an answered row and keep `list`/`next` calling a request "answered".

---
▸ 2026-10-26T22:00:45Z [request-1]

Decision: the derivation is `store::agent_why(task) -> Option<AgentWhy>`: the latest `asked:`/`requested:` marker note decides; a `needs: agent` yak with neither (hand-set, or pre-marker) is `answered`.

- Rejected: storing the why in frontmatter (the claim says derived, never stored), and a third "unknown" value for marker-less yaks (no reader needs it; `answered` matches the old wording).

---
▸ 2026-10-26T22:00:45Z [request-1]

Decision: `Farm::request` returns a new `farm::Request` enum (`NotFound` / `AwaitingHuman(needs)` / `Requested(NeedsChange)`), shaped like `Pickup`, so the refusal is a typed outcome and writes nothing.

- Rejected: `Result<Option<NeedsChange>>` plus an `anyhow` error for the refusal (callers would string-match it; the TUI wants its own message).

---
▸ 2026-10-26T22:00:45Z [request-1]

Decision: in the TUI, `open_ask_or_answer` became `open_answer_or_request`, `open_answer` was removed (only `a` needs it, via `open_needs_edit(NeedsVerb::Answer)`), and a `NeedsVerb {Ask, Answer, Request}` enum replaced the `answer: bool` threaded through `open_needs_edit` / `needs_edit` / `commit_needs`.

- Rejected: a second bool (`answer`, `request`), which admits an impossible combination.
- `Q` is bound in both the list and detail key maps, next to `a`.
- JSON `why` is `null` (not absent) for rows awaiting a human, so consumers can read the key unconditionally.

---
▸ 2026-10-26T22:01:00Z [request-1]

Real-binary transcript (own `target/debug/yaks`, throwaway `mktemp -d` private farm; `yaks` symlinked to it). The last `inbox --json` is condensed to id/needs/why by a python one-liner.

```
$ yaks request yak-3055 --note Tidy the body.
Requested yak-3055: needs agent (awaiting pickup; see `yaks inbox`)
[exit 0]
$ yaks request yak-3055 --note Then summarize.
Requested yak-3055: still needs agent (request appended)
[exit 0]
$ yaks ask yak-f206 --note Disk or memory?
Asked yak-f206: needs human (dropped from next until answered)
[exit 0]
$ yaks request yak-f206 --note x
error: yak-f206 is awaiting a human (needs: human); answer it instead (`yaks answer yak-f206`)
[exit 1]
$ yaks answer yak-f206 --note Disk.
Answered yak-f206: needs agent (awaiting pickup; see `yaks inbox`)
[exit 0]
$ yaks inbox
Awaiting an agent (`yaks pickup <id>`):
  [H] yak-3055  p3 task     Messy notes ✓ requested (needs:agent)
  [H] yak-f206  p3 task     Pick a cache ✓ answered (needs:agent)
[exit 0]
yak-3055 agent requested
yak-f206 agent answered
$ yaks next
Ready to shave (all dependencies met):
  yak-3055  p3 task     Messy notes ✓ requested, awaiting pickup
  yak-f206  p3 task     Pick a cache ✓ answered, awaiting pickup
[exit 0]
$ yaks pickup yak-3055 --note On it.
Picked up yak-3055: needs cleared
[exit 0]
$ yaks inbox --for agent
Awaiting an agent (`yaks pickup <id>`):
  [H] yak-f206  p3 task     Pick a cache ✓ answered (needs:agent)
[exit 0]
```

Headless TUI frames (a → Request editor, Ctrl-S → `Needs: agent (requested, awaiting pickup)`, and `a` on an answered yak) are attached as `yaks-b75b-tui-frames.txt`.
