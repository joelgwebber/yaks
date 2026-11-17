---
id: yaks-4dd3
title: 'source add: do not record the absolute path and host of the machine'
status: shorn
type: bug
priority: 3
created: '2026-11-08T22:23:02Z'
updated: '2026-11-17T22:28:07Z'
parent: yaks-4cfe
tags:
- doks
---

Found by Hearth's Calcifer import: every source's source.md records the absolute path and host the file was added from. For a household farm in a repository that others can read, that is a leak with no use (the readme says never to describe a source by a path on your machine). Record the file name only unless --url is given. Also: the 20 MiB per-file source limit is not in 'yaks source add --help' or the yaks-research skill (both say 5 MiB for a directory); say it.

#### 2026-11-17T21:42:26Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:42:57Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: `srccli-1` (see yaks-3da7's claim). Scope: `source.md` never records an absolute path or the `host:` of the machine. A local file's resource is its file name, and `--url` or `--origin` win as now. Say the 20 MiB per-file cap in `source add --help` and the `yaks-research` skill. Out of scope: rewriting existing sources. Evidence: a unit test on the written fields, and the gate.

#### 2026-11-17T22:03:00Z

*srccli-1*

Decision: a local file's `resource` is always its file name (`name`), with or without a scratch directory, and `host:` is dropped from the `sources:` entry. Rejected: a path relative to the repo or home (still machine-specific and needs a guess at a root), and keeping `host` (the issue calls it a leak with no use). `last_modified` stays (it says nothing about the machine). `--url` and `--origin` win as before; `promote` still records `<id>/<file>` and never had a host.

Decision: `in_scratch` is deleted, since the rule no longer branches on it. `sources::host()` stays: `session note` uses it for its own `host:` (out of scope).

Decision: the `--url`/`--origin` branches are unchanged; existing sources are not rewritten (out of scope per the claim); docs say so.

Evidence, before (installed 0.0.37, tree under `$TMPDIR/p`, from the yak-3da7 run): `resource: /private/var/folders/.../tmp.roytq7BL6k/p` and `host: joel` in `source.md`.

Evidence, after (my release build, `source add <abs path>/notes.md` in a scratch farm): `source.md` has `resource: notes.md`, `last_modified`, `retrieved`, and no `host:`.

Test: `a_local_file_records_its_name_never_a_path_or_host` (was `a_scratch_path_is_not_provenance_but_another_path_is`; it asserts no `host:` and no absolute path both for a file under the temp dir and one under `target/`, which the old test required to be recorded as an absolute path).

Docs: 20 MiB per-file cap now in `source add --help`, `docs/cli.md` and the `yaks-research` skill; the resource/host rule in `docs/cli.md`, the skill and the `--url` help.

#### 2026-11-17T22:08:22Z

*srccli-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

#### 2026-11-17T22:08:29Z

*srccli-1*

Summary: `source.md` records a local file by name only and no `host`; the 20 MiB cap is in help, docs and the research skill. Gate PASS (`yaks verify`), fmt clean.

#### 2026-11-17T22:08:30Z

*srccli-1 · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzJXZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-17T22:24:51Z

*sources-triage · kind: landed · commit: e15c8e8d82ee2ad8faf32e1a6a632ba9b0d08499 · source: 6ea78fd099cd5d749dc450e4669743157c7bdef3 · branch: main*

landed: e15c8e8d yaks-4dd3: source add records a local file by name only, no absolute path or host; say the 20 MiB cap

#### 2026-11-17T22:28:07Z

*sources-triage*

Coordinator review (`sources-triage`): landed on `main` (3da7 `9f613dee`, 4dd3 `e15c8e8d`, 06aa `2ea5597c`, fb15 `887d7795`). After all four, the full gate passed: 1463 tests, fmt clean.

Re-ran each with my own release build in a fresh scratch farm:

- A tree with `.github/`, `.gitignore`, `.git/`, `target/` and `.DS_Store` keeps the first two and reports `skipped 3: .DS_Store, .git/, target/` on stdout and in `source.md`.
- A local file records `resource: note.md` and no `host:`.
- Same bytes under a new title: "kept the title ... ignored" with the `source set` hint. `source set --title` retitles and keeps the slug.
- `source list` nests a `--part-of` under its parent, and `[uncited]` clears once a dok cites it.
