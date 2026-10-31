---
id: yaks-b8cd
title: 'Note metadata surface: kinds from commands, update --meta, JSON, TUI header, doctor drift'
status: shaving
type: feature
priority: 2
created: '2026-10-31T16:17:00Z'
updated: '2026-10-31T16:18:06Z'
parent: yaks-6a11
labels:
- cli
- ui
---

The command and TUI surface for note metadata. The format and core API landed in `7f78a28` (see the decision on `yaks-6a11`): `NoteEntry.meta` / `kind()` / `get()`, `store::append_note_meta`, `append_entry`, `meta_block`, `parse_meta`, and the TUI comment block's `meta`.

## Scope

1. **Commands write `kind`.** Every command that writes a conventional note adds `kind:` metadata and keeps its existing body text: moves (`kind: move`, plus `from` and `to`), `ask`, `answer`, `request`, `pickup` and `verify` (plus `result: pass|fail`).
2. **`yaks update <id> --meta key=value`**, repeatable, with `--note`. Reject a bad key (`store::is_meta_key`) or a missing `=`.
3. **JSON.** `yaks show --json` gains `notes: [{ts, actor, meta: {…}, text}]`, and `yaks log --json` entries gain `meta`.
4. **TUI.** The comment header shows a note's metadata compactly after `actor · when` (kind first, then the other keys, e.g. `checkpoint · commit 47076b0`). Editing a comment keeps its metadata.
5. **`yaks doctor`: drift between `needs:` and the notes.** If the latest ask/answer/request/pickup note *with* `kind` metadata implies a `needs` state that disagrees with frontmatter `needs:`, report it. Yaks with no kinded notes are never flagged.
6. **Docs and help** for all of the above: `docs/cli.md`, `docs/tui.md`, `--help`, the `?` overlay if a key changes. Not `.agents/skills/`; the yakherd writes the skills.

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:15Z [yakherd]

Claim (yakherd). Owner: worker `meta-1`. Scope: as described. Owns in `src/main.rs`: the `Update` command's args and dispatch, `Show`/`Log` JSON. In `src/farm.rs`: the note-writing methods (moves, ask, answer, request, pickup, verify). TUI: `detail::comment_label` / the comment header. Doctor: a new drift check. Out of scope: `sheds`, `land`, `commits`, `.agents/skills/`. Evidence: a failing-first test per item, a TUI text frame of a comment with metadata attached, the full gate. Judge: yakherd at landing.

---
▸ 2026-10-31T16:18:06Z [meta-1]

Start (meta-1).

- `date -u`: Sat Oct 31 16:18:03 UTC 2026
- `yaks path yaks-b8cd`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-b8cd.md
