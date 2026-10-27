---
id: yaks-31e2
title: Attachments attempt to open on wrong path across sheds
status: shorn
type: bug
priority: 2
created: '2026-10-17T13:21:35Z'
updated: '2026-10-27T01:10:50Z'
labels:
- sheds
- tui
- attach
needs: agent
---

When opening an attachment from the TUI, for a yak that exists on a different shed (or if only the _attachment_ exists on said shed), it fails. Because it always attempts to open attachments from the cwd.

---
▸ 2026-10-17T13:26:59Z [Joel Webber]

Let's ensure that there's an equivalent CLI affordance so we make this easy for agents as well.
Related: yaks-991c

---
▸ 2026-10-17T13:34:45Z [delta-attach]

moved: hairy -> shaving

---
▸ 2026-10-17T13:36:27Z [delta-attach]

Plan (delta-attach, 2026-10-17).

**Root cause.** `App::open_external` joins every attachment link to *this* checkout's farm root. When the yak exists only in another shed, or only that shed has the attachment file (a worker attached evidence and hasn't landed), the path doesn't exist here, so `xdg-open` fails.

**One rule, shared by the CLI and the TUI:** resolve attachment `<id>/<file>` to the first farm that has the file, in this order:

1. this checkout's farm;
2. the shed whose copy is on screen (a shed-only yak's shed);
3. any other shed's farm, in the sheds scan order.

The answer names the shed it came from when it isn't this checkout.

- **Library:** `Farm::find_attachment(cwd, id, name) -> Option<(PathBuf, Option<String>)>` in src/farm.rs: the path, plus the shed name when it was found elsewhere. It reuses `other_sheds` / `store::discover_with` like `show_sheds` does. Read-only.
- **CLI:** `yaks attachment <id> [<file>]` (new, read-only). It lists a yak's attachments across sheds (`<shed>  <abs path>`), or with `<file>` prints the absolute path of that one (exit 1 if no farm has it). `--json` is available. Agents can then `cat`/open evidence from a worker's shed without knowing where it lives. Answering questions across sheds already exists (`yaks answer <id>@<shed>`, `inbox --sheds`); this closes the "inspect attachments" half.
- **TUI:** `open_external` for a `Target::Artifact` asks the TUI's sheds snapshot first: the selected shed-only yak's shed, then other sheds. It does no git discovery on keypress, since the snapshot already has every shed's path. The notification names the shed (`opened build-log.txt (from shed hscroll-1)`).

Out of scope: copying attachments between sheds; showing other sheds' attachments inline in the detail pane.

[cross-shed-attachments](yaks-31e2/cross-shed-attachments.txt)

---
▸ 2026-10-17T13:42:49Z [delta-attach]

Done (delta-attach).

- **Library** (src/farm.rs): `locate_attachment` / `list_attachments` over a `(shed, farm root)` list, this checkout first; `Farm::attachment_farms` builds that list from `yaks sheds` discovery. One rule for the CLI and the TUI.
- **CLI:** new read-only `yaks attachment <id> [<file>]` (`--json`). It lists `<shed>  <path>` across sheds, or prints one file's absolute path; exit 1 when no farm has it.
- **TUI:** `open_external` resolves attachments via `App::resolve_attachment`, built from the sheds snapshot (the shed-only yak's shed first, then the others; no git on keypress). It reports `opened … (from shed <name>)`, or `… not in this farm or any shed` instead of handing `xdg-open` a path that doesn't exist.
- **Docs:** docs/cli.md row, docs/tui.md `A`/`O` row, `?` help entry, the yaks skill's command table, the coordinating skill's watch list.

Evidence:

- Tests that fail without the fix: `farm::tests::attachments_resolve_across_shed_farms_in_order` (fails when only this farm is searched), `tests/attachment_sheds.rs` (real git worktree shed; fails with shed discovery stubbed out), `tui::tests::an_attachment_only_another_shed_has_resolves_to_that_shed`.
- `cargo test --release` green (18 suites).
- Attached `cross-shed-attachments.txt`: the real binary on a real repo + worktree. `yaks attachment` finds the worker's file from the primary, and the headless TUI opens a shed-only yak's 📎 attachment from the worker's farm (xdg-open stubbed to record its argument).

Answering open questions across sheds was already covered by `yaks answer <id>@<shed>`, `inbox --sheds` and the TUI's `a`; nothing changed there. yaks-991c (route every agent interaction through the CLI so it sees the whole farm) is the larger follow-up this touches.

---
▸ 2026-10-17T13:43:14Z [delta-attach]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T13:43:14Z [delta-attach]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-31e2` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-31e2 --drop-request`, which removes this request note, so no new note is left behind.
