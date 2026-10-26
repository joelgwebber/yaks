---
id: yaks-f853
title: request --kind tidy|summarize and bulk --request
status: shorn
type: feature
priority: 2
created: '2026-10-26T23:07:03Z'
updated: '2026-10-26T23:19:10Z'
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

---
▸ 2026-10-26T23:08:04Z [kinds-1]

Start (kinds-1): `Mon Oct 26 23:08:00 UTC 2026`.

`yaks path yaks-f853`: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f853.md`

---
▸ 2026-10-26T23:12:22Z [kinds-1]

Decision: the instruction texts live in `store::RequestKind::instruction(id)`, a clap `ValueEnum` in `src/store.rs` next to `request_text`, and take the yak id so the text says `yaks path <id>` and `yaks pickup <id>` literally. `store::compose_request` joins kind text and extra note.

Rejected:

- a generic `yaks path <id>` placeholder: the agent would have to substitute it, and a bulk request then gives every yak an identical, less direct note;
- putting the kind inside `Farm::request` (a new parameter): every caller, the TUI included, would change for what is only text composition. `Farm::request` stays as it was; the CLI composes the note and passes it.

---
▸ 2026-10-26T23:12:22Z [kinds-1]

Decision: "`--kind` or `--note` or both required" is a clap `ArgGroup` (`kind`, `note`, `note_file`; required, multiple), so the usage line shows it and clap exits 2 with its own error.

Rejected: a hand-written check in the dispatch arm (exit 1, and invisible in `--help`). Before this change a bare `yaks request <id>` was allowed and recorded only the marker; it is now an error, as the claim asks.

---
▸ 2026-10-26T23:12:23Z [kinds-1]

Decision: `bulk --request` reuses the existing `bulk` dry-run/commit flow. Its extra text is a new `bulk --note` that `requires = "request"`; it is passed through `resolve_text`, so `--note -` reads stdin. The commit path is a new `request_many` that calls `Farm::request` per yak, like `update_many`. A yak awaiting a human is skipped, listed on one `skipped N awaiting a human (answer instead): ...` line, and does not fail the run (exit 0). An unknown id still exits 1. The dry run marks such a yak `(awaiting a human: request skipped)`, and `describe_bulk_mutation` adds `request <kind>` (with `(note: "...")` when given).

Rejected:

- a non-zero exit on skips: the claim says skip and report, and a skip is expected, not an error;
- dropping skipped yaks from the dry-run list: then the dry run would not show what will be skipped;
- naming the flag `--request-note`: `--note` matches `request --note`, and `requires` makes the dependency explicit.

The actor for bulk requests comes from `actor::resolve(None)`, since `bulk` has no `--as` (same as its field edits).

---
▸ 2026-10-26T23:12:23Z [kinds-1]

Tests fail first. In `tests/needs_loop.rs` I added `request_kind_writes_the_instruction_then_the_note` (it also checks that neither flag is an error) and `bulk_request_dry_runs_then_queues_and_skips_awaiting_human`, plus a `Farm::body` helper that reads the file via `yaks path`. With `git stash push -- src` (the original source):

```
test request_kind_writes_the_instruction_then_the_note ... FAILED
test bulk_request_dry_runs_then_queues_and_skips_awaiting_human ... FAILED
test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s
```

With the change, `cargo test --test needs_loop` gives `test result: ok. 8 passed; 0 failed`.

---
▸ 2026-10-26T23:12:23Z [kinds-1]

Real-binary transcript (`./target/debug/yaks` on a throwaway `mktemp -d` farm, `git init`, `init --mode private`, three yaks labelled `docs`):

```
$ YAKS_ACTOR=coord yaks ask yak-a0a7 --note 'which?'
Asked yak-a0a7: needs human (dropped from next until answered)
$ yaks request yak-590d
error: the following required arguments were not provided:
  <--kind <KIND>|--note <NOTE>|--note-file <PATH>>

Usage: yaks request <--kind <KIND>|--note <NOTE>|--note-file <PATH>> <ID>

For more information, try '--help'.
exit 2
$ yaks request yak-590d --kind tidy --note 'esp. the plan note'
Requested yak-590d: needs agent (awaiting pickup; see `yaks inbox`)
$ sed -n '/requested/,$p' "$(yaks path yak-590d)"
requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yak-590d` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yak-590d --note "<one line>"`.

esp. the plan note
$ yaks inbox
Awaiting a human:
  [H] yak-a0a7  p3 task     Held [docs] ⚠ needs:human
Awaiting an agent (`yaks pickup <id>`):
  [H] yak-590d  p3 task     Alpha [docs] ✓ requested (needs:agent)
$ yaks bulk --label docs --request summarize
would update 3 yaks:
  yak-590d  Alpha
  yak-874d  Beta
  yak-a0a7  Held  (awaiting a human: request skipped)
mutation: request summarize
(dry run — pass --commit to apply)
$ yaks bulk --label docs --request summarize --commit
updating 3 yaks: request summarize
Requested yak-590d: summarize (needs agent)
Requested yak-874d: summarize (needs agent)
skipped 1 awaiting a human (answer instead): yak-a0a7
exit 0
$ yaks inbox --for agent
Awaiting an agent (`yaks pickup <id>`):
  [H] yak-590d  p3 task     Alpha [docs] ✓ requested (needs:agent)
  [H] yak-874d  p3 task     Beta [docs] ✓ requested (needs:agent)
$ sed -n '/requested/,$p' "$(yaks path yak-874d)"
requested: needs agent

Summarize request: append one note summarizing this yak: its current state, the decisions made, and what is left.

Change nothing else. Add it with `yaks update yak-874d --note -`, then `yaks pickup yak-874d`.
```

---
▸ 2026-10-26T23:14:11Z [kinds-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-26T23:15:20Z [kinds-1]

Summary (kinds-1):

- `yaks request <id> --kind tidy|summarize [--note extra]`: the note under `requested: needs agent` is the kind's instruction (first line `Tidy request:` / `Summarize request:`), then the extra note after a blank line. `--kind`, `--note`, or both is required (a clap `ArgGroup`). The texts live in `store::RequestKind::instruction(id)` in `src/store.rs`.
- `yaks bulk <filters> --request <kind> [--note extra] [--commit]`: the dry run marks yaks awaiting a human as skipped. The commit calls `Farm::request` per yak (`request_many`), skips and reports yaks awaiting a human, and exits 0. It works alone or with the field edits.
- `Farm::request`, the marker and `agent_why` are unchanged.
- Docs: `--help` for `request`/`bulk`, `docs/cli.md` (`### request`, `### bulk`, both tables), `README.md` (table rows and the bulk paragraph).

Evidence: two new tests in `tests/needs_loop.rs` fail on the old source and pass now; the real-binary transcript is above; `yaks verify` gives PASS for `cargo test -p yaks`.

---
▸ 2026-10-26T23:15:20Z [kinds-1]

moved: shaving -> shorn

---
▸ 2026-10-26T23:19:10Z [delta-lead]

Landed (delta-lead, yakherd) and pushed in ab5e667. Gate on the combined tree: 873 passed, 0 failed. Reproduced on a throwaway farm with my own build: list --run-on found the run-on yaks, bulk --run-on --request tidy dry-ran (skip marked), --commit queued one and skipped the yak awaiting a human, the tidy instruction is in the note, a bare request exits 2. Live farm: 124 yaks match --run-on.
