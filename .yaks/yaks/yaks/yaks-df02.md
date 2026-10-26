---
id: yaks-df02
title: Tighten skill guidance around completion notes & evidence
status: shorn
type: task
priority: 1
created: '2026-10-17T13:57:13Z'
updated: '2026-10-26T21:00:47Z'
labels:
- skills
---

It seems to be a pretty mixed bag as to whether we get evidence in agent comments and completion messages. Sometimes we get nothing (just "shorn"), and other times we get War and Peace. Sometimes we get attached evidence, other times not even when it would have been trivial.

Some general principles (leave a little wiggle room for the agent to ignore when it's nonsensical):
- Explain the basic shape of the fix, unless it's truly trivial.
- Attach evidence wherever it makes sense:
  - Web UI - screenshots. Use `sightmap browser` when available; it gives good tools for introspecting the DOM and component tree so that you can get focused screenshots when needed.
  - TUI - text output at least. If using a TUI library that supports it, screenshots.
  - Code - link to code when possible; small snippets if appropriate (see also: yaks-da5e).
  - Logs - focused summaries of log output when it's helpful.
- Keep completion/evidence comments reasonably terse. Design docs belong in attachments, not comments.

Also relevant: yaks-699b

---
▸ 2026-10-19T04:30:38Z [delta:herd-pass]

Herd pass 2026-10-19: still open and still p1; land it with the skills rewrite (yaks-b048) since the evidence guidance lives in yaks and yaks-working, or do it first as a small edit if that rewrite waits.

---
▸ 2026-10-26T20:55:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T20:56:25Z [delta-lead]

Claim (delta-lead, yakherd): skills prose, done in the coordinating thread together (all three edit .agents/skills/yaks and yaks-working; AGENTS.md says keep skill rewrites in the coordinator). Worker answer-editor-1 (yaks-ce5e, thread ksQQXwyj…, model: profile default, base 2b1f3ba) runs in parallel and touches no skills.

---
▸ 2026-10-26T21:00:47Z [delta-lead]

Done in fb432be (delta-lead), with yaks-8aeb, yaks-df02 and yaks-248d together, since they share the same skill sections.

- `yaks` skill, "Notes and descriptions": the blank-line-between-blocks rule, short blocks, lists for parallel facts, what renders and what doesn't, a wrong/right example, every link form (including linking a yak's absolute `yaks path` from a chat harness), and stdin for any note with a newline, backtick or `$`.
- "Evidence before you shear": evidence by kind of change (web UI with `sightmap browser`, TUI frame, code link, focused log lines), and a short summary-note spec (no bare "done", long write-ups go in an attachment).
- `yaks-working` and the `src/brief.rs` notes and finish lines say the same thing in brief.
- AGENTS.md: a markdown rendering change must update the skill's Notes section and `brief.rs`.

Evidence: a cold-read by Sonnet scout cold-read-1 (scenarios 1-10 answered correctly from the text); its gaps were fixed, except two that already existed before this change, now filed as a new yak. `cargo test`: 708 passed, 0 failed.

---
▸ 2026-10-26T21:00:47Z [delta-lead]

moved: shaving -> shorn
