---
id: yaks-c53a
title: 'Body content input beyond inline args: --description-file, stdin, and yaks edit'
type: feature
priority: 2
created: '2026-08-23T03:43:29Z'
updated: '2026-10-05T22:39:49Z'
parent: yaks-8d53
labels:
- cli
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
