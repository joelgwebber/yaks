---
id: yaks-c53a
title: 'Body content input beyond inline args: --description-file, stdin, and yaks edit'
status: shorn
type: feature
priority: 2
created: '2026-08-23T03:43:29Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-8d53
labels:
- cli
needs: agent
---

create and update accept --description-file PATH and --description - (read body from stdin), plus a yaks edit ID command that opens the yak (frontmatter + body) in EDITOR and validates on save. Motivation: rich markdown bodies with backticks, quotes and newlines are painful and unsafe as inline --description args, especially from an agent shell; this drove hand-edits of .yaks/*.md during the skills port and the description-restore pass.

---
▸ 2026-10-05T22:39:49Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-05T22:39:49Z [delta-lead]

Claim (coordinator delta-lead), scoped by Joel's yaks-382b. Agents cram multi-line markdown into one-line --description/--note arguments, and the TUI's markdown renderer is a line-at-a-time highlighter, so a one-line blob renders as one wrapped paragraph.

Owner: worker inp-1.

Scope for THIS change (not `yaks edit`):
- `-` as the value of --note and --description means "read stdin", on create, update, ask and answer.
- --description-file PATH and --note-file PATH on the same commands.
- Multi-line input is stored exactly (trailing newline trimmed). An empty stdin is an error, not an empty note. `-` together with another stdin consumer is an error.
- `yaks edit` stays out of scope (split it to its own yak if you think it is worth it).

Judge: coordinator re-runs the gate and pipes a heredoc into create, update, ask and answer.

Forbidden: changes to the on-disk format, the farm lock rule (take it once, outermost), coordination skills prose (the coordinator writes the guidance using this interface).

---
▸ 2026-10-05T22:40:46Z [inp-1]

Start: 2026-10-05T22:40:46Z. Pre-existing: .yaks/shaving/yaks-c53a.md. Built ./target/release/yaks; starting on args/handlers in src/main.rs.

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: one helper (resolve_text + StdinText) in main.rs resolving inline | `-` | file; handlers resolve before touching the farm, so Farm/store and the on-disk format are unchanged (no Farm change needed). Rejected: pushing resolution into Farm (CLI concern; would add stdin to the lock path).

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: `--note-file -` / `--description-file -` also mean stdin (Unix convention), one rule: `-` is stdin in either flag form. Rejected: treating `-` as a literal file name, or erroring on it.

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: inline-vs-file conflict is enforced by clap `conflicts_with` (exit 2, 'cannot be used with'), not hand-rolled; the other errors (empty, double stdin, unreadable file, tty) are anyhow errors, exit 1. Rejected: a single hand-rolled message for all.

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: trimming = strip ALL trailing \n/\r (not just one), keep leading/interior whitespace; empty means whitespace-only (error). Inline values are never trimmed or checked, so `--description ""` still clears a body and existing inline use is byte-identical. Rejected: trim() both ends (would eat leading indent of a code block); trimming exactly one newline (a file with two trailing newlines would leave a stray blank line before the next note).

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: stdin on a terminal is refused with an error (not a prompt/banner). Rejected: printing 'reading stdin, Ctrl-D to end' and waiting (agents would hang; humans have heredocs/--note-file).

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: only create/update/ask/answer got the new flags. `attach --note` keeps inline-only (out of the brief's four commands; trivially extendable with the same helper). `yaks edit` left out as scoped. `update` with multiple ids reads stdin once and applies the same text to each.

---
▸ 2026-10-05T22:44:20Z [inp-1]

Decision: new integration test file tests/body_input.rs (11 tests: each command with - and file, byte-for-byte text with backticks/quotes/$/blank lines/fence, CRLF trim, multi-id, every error case, failed command leaves the yak file untouched) plus 3 unit tests in main.rs for tty refusal and once-only stdin (tty can't be faked via assert_cmd). Rejected: only in-module tests (don't exercise the real CLI/clap conflicts).

---
▸ 2026-10-05T22:44:20Z [inp-1]

Evidence: gate `cargo test -p yaks` green (417 unit + 8+11+28+19+3 integration; new tests fail before: flags did not exist). Real run in a temp farm: heredoc with heading/list/fence/blank line/backticks piped into create --description -, update --note -, ask --note -, answer --note - and `yaks show` showed the text byte-for-byte, multi-line; script(1) pty with stdin as tty gave 'Error: --note - reads stdin, but stdin is a terminal...'. Headless TUI (`yaks tui --headless`, key l) rendered the stdin-created body as separate lines (heading, list, fence, paragraph), not one wrapped blob. Docs updated: docs/cli.md (create/update/ask/answer rows + new 'Multi-line text' section), README.md, .agents/skills/yaks/SKILL.md commands table, clap --help. Docs-parity grep: grep -rn -e '--note' -e '--description' docs README.md .agents/skills/yaks/SKILL.md src; docs/skills.md and docs/README.md mention only the inline form generically and need no change.

---
▸ 2026-10-05T22:44:20Z [inp-1]

moved: shaving -> shorn

---
▸ 2026-10-10T20:06:05Z [delta-lead]

Failure mode seen 2026-10-10 (worker here-1 on yaks-ad6d, Delta, macOS): the input flags this yak shipped were not used. The worker wrote its note as `yaks update <id> --note "$(cat <<'EOF' ... EOF)"` and the body held backticked words and a `(...)`. The terminal call hung for 10+ minutes: the backticked words ran as commands (one was `top`, which opened a full-screen viewer on the tty and blocked; another was `yaks discover`, whose whole output was pasted into the note), so the note was both stuck and corrupted. A quoted heredoc delimiter was there and did not protect it; I did not pin down whether that is a shell quirk of the macOS `sh` or of how the tool wraps the command, so do not rely on `$( )` with a heredoc body at all.
Safe forms, both already supported here: `yaks update <id> --note - <<'EOF' ... EOF` (heredoc straight to stdin, no `$( )`), or write the text to a file and pass `--note-file PATH`.
Cost: a worker's final result was delayed by the hang until the coordinator found and killed the child; the corrupt text stays in the thread's transcript. See yaks-382b for the guidance side. Possible yaks-side mitigation, not decided: none that detects it (the text is already substituted before yaks runs).

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-c53a` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-c53a --drop-request`, which removes this request note, so no new note is left behind.
