---
id: yaks-b8cd
title: 'Note metadata surface: kinds from commands, update --meta, JSON, TUI header, doctor drift'
status: shorn
type: feature
priority: 2
created: '2026-10-31T16:17:00Z'
updated: '2026-10-31T16:52:20Z'
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

---
▸ 2026-10-31T16:28:45Z [meta-1]

Progress (meta-1), 16:5x UTC.

- Done in code: kinds on move/ask/answer/request/pickup/verify, `update --meta`, `show --json` `notes`, `log --json` `meta`, TUI `comment_label`, doctor `needs-drift` (`IssueKind::NeedsDrift`), with farm unit tests.
- Next: CLI/JSON/TUI tests, docs (`docs/cli.md`, `docs/tui.md`), TUI frame attachment, gate.

[comment-header-metadata-frame](yaks-b8cd/comment-header-metadata-frame.txt)

---
▸ 2026-10-31T16:36:44Z [meta-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-31T16:38:48Z [meta-1]

Decision: `answer` notes carry a `needs` key (`agent`, or `none` for `--done` or a yak with no `needs`), and `ask` notes carry `needs: <who>`. Rejected: inferring from `kind` alone, because `answer --done` and an answer on a yak with no `needs` leave no `needs:`, and would read as drift.

Decision: `doctor` allows the two note-less `needs:` changes (`answer` with no `--note`, `pickup --drop-request`) instead of flagging them. Rejected: flagging them, which would false-alarm on routine commands.

Decision: `show --json` adds `notes` only when the yak has notes, so the shape stays additive. Rejected: always emitting `notes: []`, which would rewrite every golden snapshot.

Decision: the TUI header shows `kind` bare and the other keys as `key value`, joined with ` · `. Rejected: `key=value` chips, which are noisier.

Decision: `--meta` splits on the first `=` and requires a note, with an error and no write otherwise. Rejected: silently ignoring `--meta` without a note.

Core bug (flagged): `store::drop_latest_request` assumed note text starts two lines after the `---` delimiter, so `pickup --drop-request` failed on a request note that now has metadata. Fixed minimally by starting after `parse_meta`'s end, with a test in `store.rs`.

Test churn: `tests/attribution.rs` `has_move` now expects the metadata lines. `tests/body_input.rs` reads only the frontmatter when it checks `needs:`, because notes now carry `needs:` lines too. `tests/snapshots/cli__show_json.snap` gains `notes`.

Gate: `GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.fsmonitor GIT_CONFIG_VALUE_0=false cargo test --workspace --no-fail-fast` passed with 0 failures (767 unit tests plus every integration suite). `yaks verify yaks-b8cd` recorded PASS. TUI frame attached as `comment-header-metadata-frame.txt` (the insta snapshot `comment_header_with_metadata`).

Summary: commands write `kind` metadata and keep their body text. `update --meta` is added. `show --json` has `notes` and `log --json` has `meta`. The comment header shows metadata. `doctor` has a `needs-drift` check. `docs/cli.md`, `docs/tui.md`, `README.md` and the clap help are updated. No `help_content` change, since no key changed.

Skills should say (yakherd owns): `update --meta key=value` and `kind`s exist; the commands stamp kinds themselves; `needs-drift` meaning; and the notes line in `src/brief.rs` and the `yaks` skill's notes section could mention metadata.

---
▸ 2026-10-31T16:38:49Z [meta-1]

moved: shaving -> shorn

---
▸ 2026-10-31T16:42:39Z [yakherd]

Landed by the yakherd as `71ca61e` (squash of `1e6ce08`). Re-ran `cargo test --workspace` (all green with the fsmonitor workaround). Reproduced on a scratch farm with the landed build: moves, checkpoints, asks and answers carry `kind` metadata; `--meta` rejects a bad key and a missing note; `show --json` lists notes with `meta`; a hand-edited `needs:` that disagrees with the latest answer is reported by `doctor` as `needs-drift`.

---
▸ 2026-10-31T16:52:20Z [yakherd]
kind: landed
commit: 71ca61e943004f270ffdad04850e685f7a706e41
branch: main
---

landed: 71ca61e yaks-b8cd: note metadata surface: kinds from commands, update --meta, JSON notes/meta, TUI header, doc (recorded by hand: landed with a yaks binary from before `yaks-735a`)
