---
id: yaks-06aa
title: source add --refresh ignores a new --title when the content is unchanged (no way to retitle or fix a source)
status: shaving
type: bug
priority: 3
created: '2026-11-07T15:58:15Z'
updated: '2026-11-17T22:14:20Z'
parent: yaks-4cfe
tags:
- cli
- doks
---

Found while ingesting sources in yaks-3610 (0.0.33).

I saved a directory source with a title that contained a name I had guessed, and there was no command to correct it. The only fix was hand-editing `source.md`, which the research skill says never to do for marks and which then needs a `yaks commit`.

## Reproduction

```sh
yaks source add a.md --title "Wrong title" --url https://example.com/a
yaks source add a.md --title "Right title" --url https://example.com/a --refresh
yaks source list
```

The second command prints `Already saved: sources/<id>/source.md` and the title stays "Wrong title". `--refresh`'s own spec (yaks-87d3) says "same content: reports unchanged and writes nothing", so a corrected `--title` is dropped silently.

Without `--refresh` the same bytes also return the existing source and ignore the new title.

## Decide

- Either `--refresh` (or a new `yaks source set <id> --title ...`) updates the title and description when they differ, with one per-change commit.
- Or `source add` says it ignored a different `--title` instead of reporting "Already saved".

Lean: the second at minimum (silence is the bug), plus `source set --title`, since a title is the one field a human would correct and `source mark` already shows the pattern for editing a `source.md` field through a command.

#### 2026-11-17T21:42:26Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:42:58Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: `srccli-1`. Scope, the issue's lean:

- `source add` that finds the same bytes with a different `--title` says it kept the old title, instead of a bare "Already saved".
- `yaks source set <id> --title <t>` rewrites the title through a command, one per-change commit, following `source mark`'s pattern.

Docs, help and skills in the same change. Evidence: tests for both, and the gate.

#### 2026-11-17T22:10:46Z

*srccli-1*

Decision: both halves of the lean: `source add` of the same bytes (plain or `--refresh`, unchanged content) with a `--title` that differs from the saved one prints `Already saved: <dir>/source.md (kept the title "Old"; --title "New" ignored: `yaks source set <id> --title ...` changes it)`; no `--title`, or the same title, prints as before. Rejected: letting `--refresh` retitle (its spec, yaks-87d3, is "same content writes nothing"; one explicit command is clearer).

Decision: `yaks source set <id> --title T` (new `SourceAction::Set`, `sources::set_title`, `Farm::source_set_title`), not an `--set key=value`: title is the one field to fix, and a generic setter would let people edit `sha256`. The directory and slug are NOT renamed (a rename breaks every link, the same reason `--refresh` keeps them); `source list` reads the title from `source.md`. Empty or multi-line titles are refused. Same title: `No changes`.

Decision: the note is a `Report::kept_title` field filled by `add_reporting` (not a new `Saved` variant), same plumbing as yaks-3da7. The CLI commits the farm change itself after the command (as it does for `source mark`): the scratch run's `git log` shows `yaks: doks sources/.../source.md` after `source set`.

Evidence, scratch farm, my release build:

    $ source add a.md --title "Wrong title" --url https://example.com/a
    Saved sources/2026-11-17-87428fc5-wrong-title/source.md
    $ source add a.md --title "Right title" --url https://example.com/a --refresh
    Already saved: sources/2026-11-17-87428fc5-wrong-title/source.md (kept the title "Wrong title"; --title "Right title" ignored: `yaks source set 2026-11-17-87428fc5 --title ...` changes it)
    $ source set 2026-11-17-87428fc5 --title "Right title"
    Retitled sources/2026-11-17-87428fc5-wrong-title/source.md: Right title
    $ source set ... (again)       -> No changes to sources/.../source.md
    $ source set ... --title ""    -> Error: --title needs a title: say what this source is
    $ source list
      sources/2026-11-17-87428fc5-wrong-title  text/markdown  Right title  (https://example.com/a)

Test: `same_bytes_with_another_title_says_so_and_set_title_changes_it` (plain add, `--refresh`, same/no title says nothing, set, no-op, directory unchanged, refused forms). Docs: `source add` and `source set` `--help`, `docs/cli.md`, README, `yaks-research` skill.

#### 2026-11-17T22:14:13Z

*srccli-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-17T22:14:20Z

*srccli-1*

Summary: `source add` says when it kept a different title; `source set <id> --title` rewrites it (directory unchanged), with docs, help and skill. Gate PASS (`yaks verify`), fmt clean, scratch-farm run above.
