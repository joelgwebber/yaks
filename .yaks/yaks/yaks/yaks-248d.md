---
id: yaks-248d
title: Skill guidance for agents to link into yaks
status: shorn
type: feature
priority: 3
created: '2026-10-12T14:05:15Z'
updated: '2026-10-26T21:00:47Z'
depends_on:
- yaks-b048
labels:
- skills
---

Agent harnesses that can render links in conversation, should be able to link to yaks. Eg, in Zed/Delta, the yak markdown files already render nicely, so opening them directly is a decent experience.

Hard part: Links will drift as yaks are shorn.

---
▸ 2026-10-19T04:29:29Z [delta:herd-pass]

Herd pass 2026-10-19: the 'hard part' is gone. Since schema 5 a yak's file is `.yaks/<id>.md` for its whole life (no status directories), and the farm worktree is at a stable absolute path (`yaks path <id>`), so a link written once keeps working. What is left is the guidance itself (fits in the skills rewrite, yaks-b048).

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
