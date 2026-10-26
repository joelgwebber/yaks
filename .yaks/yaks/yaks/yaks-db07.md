---
id: yaks-db07
title: Canned requests (tidy/summarize), markdown lint for run-on bodies, bulk --request
status: shorn
type: feature
priority: 3
created: '2026-10-26T21:51:28Z'
updated: '2026-10-26T23:19:10Z'
parent: yaks-33c8
labels:
- cli
- markdown
---

Follow-ups to the request verb (see the design note on yaks-33c8): canned request kinds the skill defines, a doctor/list lint that finds bodies with runs of adjacent plain lines, bulk --request to queue them, and auto-tidy on import (yaks-00c4).

---
▸ 2026-10-26T23:07:03Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T23:07:03Z [delta-lead]

## Plan (delta-lead, 2026-10-26)

Joel: go ahead; he will try tidies once this lands.

Decision: canned requests are **request kinds whose instruction text is written into the note**, so the agent reading the inbox has the whole instruction with no skill lookup. `yaks request <id> --kind tidy|summarize [--note extra]`.

- **tidy:** change only the line structure of the body (description and every note) so it renders as intended under CommonMark: blank lines between blocks, lists for run-on facts, fences for pasted output. Never change words, note headers or frontmatter. Edit the file at `yaks path <id>` directly, because `update --description` replaces the whole body, notes included.
- **summarize:** append one summary note (current state, decisions, what is left), leaving everything else alone.

Rejected: kinds defined only in the skill (an agent without the skill gets a bare word), and free text only (Joel wants one-keystroke chores).

Decision: finding candidates is a **shared filter flag `--run-on`**: a yak whose description or any note has a paragraph with a soft line break (pulldown-cmark `SoftBreak`), i.e. lines that will join. It composes with `list`, `path`, `bulk` and the rest. Hard-wrapped prose is flagged too, which is right (the skill says don't hard-wrap). Rejected: a `doctor` check (doctor is integrity, not style) and a separate `lint` command (the filter already composes with `bulk`).

**Queue in bulk:** `yaks bulk --run-on --request tidy --commit`.

Out of scope: auto-tidy on import, since yaks-00c4 (import) isn't built; noted there.

Workers:

- `runon-1`: the `--run-on` filter (`FilterSpec` + `FilterFlags`, `src/filter.rs`, docs).
- `kinds-1`: `request --kind` and `bulk --request` (`src/farm.rs`, `store`, `main.rs` Request/Bulk, TUI request editor gets no kind picker this round).
- delta-lead: the skills.

---
▸ 2026-10-26T23:07:58Z [delta-lead]

Fan-out (delta-lead): runon-1 on yaks-ef3b (thread …AC17XpIB…), kinds-1 on yaks-f853 (thread …AC17k5IB…), both on the profile default model, base 696348e.

---
▸ 2026-10-26T23:19:10Z [delta-lead]

Done (delta-lead). Pushed to the human's checkout as `ab5e667`; installed with `cargo install --path .`.

- yaks-ef3b: the `--run-on` filter (`537960e`). 124 live yaks match.
- yaks-f853: `request --kind tidy|summarize` and `bulk --request` (landed as `58c6b3d`).
- Skills (`ab5e667`): canned requests are followed literally, `--run-on` is in the filter list, and the coordinating skill shows the bulk recipe.
- Gate: 873 passed, 0 failed.

Auto-tidy on import stays with yaks-00c4 (noted there).

---
▸ 2026-10-26T23:19:10Z [delta-lead]

moved: shaving -> shorn
