---
id: yaks-7c76
title: Find-references affordance in CLI/TUI
status: shorn
type: feature
priority: 3
created: '2026-10-17T14:22:08Z'
updated: '2026-10-18T06:10:55Z'
labels:
- cli
- tui
- links
---

I don't recall whether this functionality is hiding in the core CLI already, but it would be really helpful to be able to find references *to* a given yak, from desc/comments. We only model forward-references formally, which is the right choice, so we don't have to maintain an external back-ref index (or make updates more complex).

But find-references can be a bit slower, and perhaps (eventually) leverage a hidden index if needed. We'd want the core functionality in the lib/cli, and a simple way in the tui to quickly find and filter them. I imagine we could leverage the existing fuzzy-search tui for filtering from the set of incoming references. Including parents/children/blockers/blocked-byj -- even though those are easy to find in the detail UI, it would still be convenient to quickly filter them alongside other back-refs.

---
▸ 2026-10-18T05:53:21Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T05:53:21Z [delta-farm]

Claim (delta-farm, yakherd; trial round 1 of yaks-6a34). Owner: `backrefs-1`.

Goal: find what points AT a yak, computed on demand (no stored back-ref index). Incoming = every yak that names this one: as `parent:` (its children), in `depends_on:` (yaks it blocks), or as an id mention in its title, description or a note (the same matching `yaks refs` uses: src/refs.rs `scan`/`scan_text`, whole real ids only). Each result says HOW it refers (child / depends-on / mention in title|body|note, with the note's timestamp and actor for a note).

Surfaces (decided by the yakherd, so not yours to fork):
- Library: `Farm::backrefs(id)` in src/farm.rs beside `Farm::refs`.
- CLI: `yaks refs <id> --incoming` (extends the existing `refs` command; `--json` like its siblings). Plain `yaks refs <id>` is unchanged.
- TUI: one key in the yak views/detail opens the existing fuzzy picker (src/tui/fuzzy.rs, `FuzzyAction::Goto` or a new variant at the END of the enum) over the incoming yaks, each row labelled with how it refers; Enter goes to it. Pick a free key (grep the key handlers), record it as a `Decision:`, and put it in `help_content`.

Out of scope: an index or cache; outgoing refs (exists); the shed-aware variants (every shed shares the farm now).

Evidence: tests that fail before (lib: children, deps, title/body/note mentions found, lookalike prose not; CLI output and `--json`; TUI: the key opens the picker with the right rows). A headless toque frame of the picker, attached. docs/cli.md, README command table, the yaks skill's `refs` row, clap help for `--incoming`, docs/tui.md and `?` help. Gate: `cargo test -p yaks`. Judge: delta-farm at landing.

---
▸ 2026-10-18T05:54:36Z [backrefs-1]

Start (backrefs-1): `date -u` = Sun Oct 18 05:54:30 AM UTC 2026

`yaks path yaks-7c76` = /home/joel/src/yaks-farm/.yaks/yaks-7c76.md

[backrefs-picker-frame](yaks-7c76/backrefs-picker-frame.txt)

---
▸ 2026-10-18T06:08:59Z [backrefs-1]

Decision: TUI key is `B` ("back-references"), bound in the list and detail panes beside `#`. It is free in the list, detail and Sheds key maps (grepped `KeyCode::Char` in src/tui/). Rejected: `r` (rename attachment in detail), `<` / `>` (not mnemonic, awkward shifted punctuation), and `Ctrl-` combos (none of the pickers use one). `B` only reads, so it is not added to `guard_shed_only`'s write list.

---
▸ 2026-10-18T06:08:59Z [backrefs-1]

Decision: the picker reuses `FuzzyAction::Goto` (Enter goes exactly like `#`, with o/i history) plus one new `FuzzyPick::only: Option<BTreeMap<id, tag>>` field, which restricts the universe, lifts the 20-row cap and tags each row. I added no `FuzzyAction` and no `Overlay` variant. Rejected: a new `FuzzyAction::GotoRef` (the commit behavior would be identical), and building an `exclude` set of every non-referrer (inverted, and there is nowhere to put the tags).

---
▸ 2026-10-18T06:08:59Z [backrefs-1]

Decision: granularity is one `BackRef` per (referring yak, way): child, depends, title, one per description LINE, one per NOTE (with ts + actor). The CLI prints one line per reference (like outgoing `refs` prints one per body line). The TUI picker merges them into one row per yak, tags joined (`[child, body:L1]`), because the picker's unit is a yak to go to. A yak mentioning its own id is not a back-ref. Order: by referring yak in `Farm::list` order (status rank, then id), then child, depends, title, body, notes. Rejected: one row per reference in the picker (duplicate rows for one destination), and `body:L<n>` numbering over the whole body including notes (a note's location is its timestamp, which is more useful).

---
▸ 2026-10-18T06:08:59Z [backrefs-1]

Decision: `--json` is `{id, title, incoming:[{id,title,status,kind, in?, line?, ts?, actor?}]}`, where `kind` is child|depends|mention (the outgoing `refs` vocabulary) and `in` is title|body|note for a mention. clap enforces `--json` `requires = "incoming"`, because plain `refs` has no JSON today and adding it is out of scope. It is rendered by `backrefs_json` in src/main.rs (like `render_scan_ids`), not in src/json.rs, which is outside the brief's file list. The matching lives in a pure `farm::backrefs_in(&[Task], id)` that both `Farm::backrefs` and the TUI (over `App.all`, so no re-read) call. It reuses `refs::scan` + `strip_wikilinks` with an only-this-id predicate, so whole-id semantics are identical to `refs`.

---
▸ 2026-10-18T06:09:14Z [backrefs-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-18T06:09:22Z [backrefs-1]

Summary (backrefs-1): incoming references are computed on demand, with no index.

- Lib: `farm::backrefs_in(&[Task], id)` plus `Farm::backrefs(id)` (beside `refs`), and the types `BackRef` / `BackRefVia` (Child, Depends, Title, Body{line}, Note{ts,actor}) / `TaskBackRefs`. Matching is `refs::scan` + `strip_wikilinks`, whole real ids only.
- CLI: `yaks refs <id> --incoming [--json]`. Plain `refs` is unchanged; `--json` requires `--incoming`.
- TUI: `B` (list + detail) opens the fuzzy picker restricted to the referrers, each row tagged with how it refers; Enter goes to it (FuzzyAction::Goto).
- Docs: docs/cli.md, docs/tui.md, the README table, the skill row + prose, clap help, `help_content` (beside Follow link).
- Tests that fail before: `farm::tests::backrefs_find_children_deps_and_mentions_but_not_lookalikes`; `tests/cli.rs` `refs_incoming`, `refs_incoming_json`, `refs_incoming_none` (they errored "unexpected argument '--incoming'" before); `tui::tests::b_opens_references_picker_over_incoming_yaks_and_enter_goes` (new snapshot `backrefs_picker`, eyeballed: a1 lists a0 [body:L1] and b0 [note:... [ann]], lookalike a10 not listed), `b_with_no_incoming_references_says_so`.
- Real run on the live farm with my release build: `refs yaks-6a34 --incoming` lists 6 children (each also body:L1) and 3 note mentions; the headless frame of `B` on yaks-6a34 (9 rows) is attached as backrefs-picker-frame.txt.
- Gate: `yaks verify` PASS. Existing insta snapshots: none changed.

---
▸ 2026-10-18T06:09:22Z [backrefs-1]

moved: shaving -> shorn

---
▸ 2026-10-18T06:10:55Z [delta-farm]

Landed (delta-farm): worker commit 5acf2ba merged by SHA as ff95142 (land.sh: base an ancestor, result identical to the computed merge, doctor clear). My checks: full `cargo test --release` 851 passed; read the library code (reuses `refs::scan`, description ends at the first note); real run on the live farm with my build: `refs yaks-6a34 --incoming` lists 6 children, their body:L1 mentions and 4 note mentions; `#` to yaks-6a34 then `B` opens the picker with 9 rows, one per yak, tags merged. Cosmetic, not acted on: a row with two notes carries both full timestamps in its tag, which crowds the title.
