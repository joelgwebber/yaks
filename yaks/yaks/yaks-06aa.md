---
id: yaks-06aa
title: source add --refresh ignores a new --title when the content is unchanged (no way to retitle or fix a source)
status: shaving
type: bug
priority: 3
created: '2026-11-07T15:58:15Z'
updated: '2026-11-17T21:42:58Z'
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
