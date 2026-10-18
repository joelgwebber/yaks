---
id: yaks-7c76
title: Find-references affordance in CLI/TUI
status: shaving
type: feature
priority: 3
created: '2026-10-17T14:22:08Z'
updated: '2026-10-18T05:54:36Z'
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
