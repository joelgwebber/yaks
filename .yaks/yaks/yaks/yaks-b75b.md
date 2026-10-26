---
id: yaks-b75b
title: 'request: hand a yak to an agent through the inbox (needs: agent without a prior ask)'
status: shaving
type: feature
priority: 2
created: '2026-10-26T21:51:27Z'
updated: '2026-10-26T21:51:51Z'
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
