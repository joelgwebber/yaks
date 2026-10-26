---
id: yaks-f853
title: request --kind tidy|summarize and bulk --request
status: shaving
type: feature
priority: 2
created: '2026-10-26T23:07:03Z'
updated: '2026-10-26T23:07:33Z'
parent: yaks-db07
labels:
- cli
---

---
▸ 2026-10-26T23:07:33Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T23:07:33Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `kinds-1`. Plan: the latest note on yaks-db07.

**Goal:** canned request kinds, and queueing requests in bulk.

- `yaks request <id> --kind tidy|summarize [--note extra]`: the note recorded under the `requested: needs agent` marker becomes the kind's instruction text, then the extra note (if any) after a blank line. `--kind` or `--note` (or both) are required; a kind is a clap `ValueEnum`. The instruction texts live in one place (e.g. `store::RequestKind::instruction()`), and the plan note on yaks-db07 has the wording intent:
  - **tidy:** change only the line structure of the body (description and every note) so it renders as intended under CommonMark: blank lines between blocks, lists for run-on facts, fences for pasted output. Never change words, note headers or frontmatter. Edit the yak's file (`yaks path <id>`) directly, since `update --description` replaces the whole body, notes included. Then `pickup` with a one-line note.
  - **summarize:** append one summary note (current state, decisions made, what is left); change nothing else; then `pickup`.
  - Keep each to a few sentences; they show up in the inbox and in the yak.
- The first line of the kind text should name the kind (e.g. `Tidy request: ...`) so a human scanning the yak sees it. Don't change the `requested: needs agent` marker or `agent_why`.
- `yaks bulk <filters> --request <kind> [--note extra] [--commit]`: queue that request on every matched yak, as a dry run unless `--commit`. It may combine with the existing field edits or stand alone. Skip yaks awaiting a human and report them, and call `Farm::request` for each (it takes the lock per yak; fine).
- **Docs:** `docs/cli.md` (`### request`, `### bulk`), `README.md` table rows if they change, and the `--help` doc comments.

**Scope:** `Request` and `Bulk` variants and their dispatch arms in `src/main.rs`, `describe_bulk_mutation`, `src/farm.rs` (`request`, or a wrapper), `src/store.rs` (kind texts), `tests/needs_loop.rs` or a new test file.

**Out of scope:** the `--run-on` filter (`runon-1` owns `FilterFlags`, `build_spec` and `src/filter.rs`; `bulk` will pick the flag up through `FilterFlags` with no work from you). The TUI (no kind picker this round). The skills (delta-lead).

**Evidence:**
- Tests that fail first: `--kind tidy` writes the instruction plus the extra note; neither flag gives an error; `bulk --request tidy` is a dry run without `--commit`, queues on `--commit`, and skips a yak awaiting a human.
- A real-binary transcript (request `--kind`, inbox, `bulk --request` dry run and commit) on a throwaway farm, pasted in a note.

**Gate:** `cargo test -p yaks`. **Judge:** delta-lead at landing.
