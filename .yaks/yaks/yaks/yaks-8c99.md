---
id: yaks-8c99
title: 'Trim the clap help: one-line summaries in yaks --help, detail in the long help'
status: shorn
type: task
priority: 2
created: '2026-10-22T00:37:58Z'
updated: '2026-10-27T01:19:36Z'
parent: yaks-b048
labels:
- cli
- docs
---

---
▸ 2026-10-22T00:37:59Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-22T00:37:59Z [delta:herd-dirs]

Claim (yakherd delta:herd-dirs). Owner: `help-1` (Sonnet worker).

**Goal.** `yaks --help` lists each command on one short line. Several commands' summaries run to a paragraph (`show`, `path`, `attachment`, `sheds`, `changes`, `discover`, `answer`, `migrate`, `private`, `preflight`, `sync`, `commit`, `land`, `status`, `brief`, `init`, `bulk`, `verify`, `attach`, `rename-attachment`, `scan-ids`, `doctor`, `inbox`, `ask`, `pickup`, ...). clap takes a command's `about` from the FIRST paragraph of its `///` doc comment and the `long_about` (shown by `yaks <cmd> --help`) from the whole comment. So for each long one: make the first paragraph a single sentence of at most ~100 characters, then a blank `///` line, then the existing detail, unchanged in substance.

**Scope.** `src/main.rs`: the `///` doc comments on `Command` variants (and subcommand enums like `SkillsAction`). Also the global `-C` and `--shed` flag help, which repeats on every `--help`: give each a one-sentence short help (`help`) and keep the detail as `long_help`.

**Out of scope.** Behavior, flags, any other file. Do not drop facts: move them into the long help. Do not touch docs/, .agents/, README.md, AGENTS.md (others are editing those).

**Evidence.** `./target/release/yaks --help` before and after (attach both as text files); every command line in the after fits on one line of about 100 characters. `./target/release/yaks sheds --help` still shows the full detail. Full gate green: `cargo build --release && cargo test --release --no-fail-fast` (some tests may assert help text; update them only if they check wording you moved, not behavior).

**Judge.** The yakherd, at landing.

---
▸ 2026-10-22T00:38:32Z [help-1]

Start: Thu Oct 22 00:38:32 UTC 2026. farm path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-8c99.md

[yaks-help-before](yaks-8c99/yaks-help-before.txt)

---
▸ 2026-10-22T00:46:23Z [help-1]

yaks --help BEFORE (target/release/yaks built from the unmodified tree)

[yaks-help-after](yaks-8c99/yaks-help-after.txt)

---
▸ 2026-10-22T00:46:23Z [help-1]

yaks --help AFTER

[yaks-h-after](yaks-8c99/yaks-h-after.txt)

---
▸ 2026-10-22T00:46:23Z [help-1]

yaks -h AFTER (short option help: -C and --shed one line each)

---
▸ 2026-10-22T00:49:46Z [help-1]

Decision: reworded the first paragraph of each long doc comment into a one-sentence summary (<= ~80 chars) and rewrote the old first sentence of the detail so it does not repeat the summary (e.g. "Copies the file under ..."); no fact dropped. Rejected: `#[command(about = ..)]` attributes (duplicates the doc comment, two sources of truth).

Decision: for `-C` and `--shed`, no explicit `help`/`long_help` attributes. clap derives `help` from the first doc paragraph and `long_help` from the whole comment, so splitting the doc comment gives `-h` one line each (`-C <PATH> Run as if started in <PATH> (like `git -C`)`; `--shed` fits one line, 95 chars max) and `--help` the detail. Rejected: explicit `help =` plus `long_help =` strings (duplicates the doc comment).

Decision: also split `refs`, `log`, `reparent` (whose first sentence was already one sentence but wrapped over ~100 chars) and `skills install` / `skills status`, so every `yaks --help` command line is one line under 100 chars. `yaks -h` is now what `yaks --help` shows for subcommands: unchanged behavior of clap.

Observed: `yaks --help` Options now shows the long help for `-C`/`--shed`; `yaks -h` shows the one-liners ("Print help (see more with '--help')" is clap's own text). `yaks sheds --help` still shows the full detail (22 lines).

Evidence: attached yaks-help-before.txt, yaks-help-after.txt (`--help`) and yaks-h-after.txt (`-h`). 49 command lines in the after, none over 100 chars (checked with awk). Full gate `cargo build --release && cargo test --release --no-fail-fast`: every suite "ok", 808 unit tests passed, 0 failed. No test asserted moved wording; none edited. Only src/main.rs changed (doc comments only).

Docs parity grep: `grep -rn "Print help|see a summary|see more with" src tests docs README.md .agents` -> no hit that describes the old text.

---
▸ 2026-10-22T00:53:00Z [help-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-22T00:53:04Z [help-1]

moved: shaving -> shorn

---
▸ 2026-10-22T00:59:43Z [delta:herd-dirs]

Landed as 3b1e175 (squash of 110d658). Yakherd re-ran: longest yaks --help command line 95 chars over 50 lines; yaks sheds --help keeps its 22-line detail; full gate 20 suites green.
