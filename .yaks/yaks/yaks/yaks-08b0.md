---
id: yaks-08b0
title: 'Skills: one source per thing, with --url and --title; conversions are not sources'
status: hairy
type: task
priority: 3
created: '2026-11-04T17:23:24Z'
updated: '2026-11-04T17:23:24Z'
parent: yaks-dedc
depends_on:
- yaks-d19c
tags:
- wiki
- docs
---

The skill half of yaks-dedc (diagnosis in its notes). Depends on the tool yak, since the skill must name `--text`.

## Changes

- `.agents/skills/yaks-research/SKILL.md`, "Save before you rely on it": rewrite the conversion bullet. The rule: **one source per thing**.
  - Save the form you will read and cite. When you convert (HTML, PDF), that is **one** `source add` of the original with `--text <conversion>`, `--url <where it came from>` and a `--title` that names the thing, not the file.
  - Say when to keep the original at all: only if it carries something the conversion loses (a PDF's figures) or it is the evidence itself. A raw HTML export of a page you converted is not; save just the markdown (`--url` still names the page).
  - `--part-of` is for parts of one thing (a page's screenshots), never for a conversion of the same thing.
  - Never describe provenance with a scratch path; `--url` and `--title` carry it.
- `.agents/skills/yaks/SKILL.md` "Pages" source bullet: one line pointing the same way.
- The `yaks` skill's `source` notes in `src/main.rs` help (`Source` doc comment) and `docs/cli.md` agree with it (the tool yak edits the flag text; this one checks the whole set reads as one story).
- Cold-read check (AGENTS.md): a cheap-model scout that reads only the two skill files answers "I downloaded a Confluence page as HTML and converted it to markdown: what do I run?" and "I have a PDF and its text: what do I run?".

## Evidence

- `cargo test -p yaks skills`; grep for the old `--part-of ... when it is a separate file` wording across `docs/ .agents/ README.md src/`.
- The scout's two answers, attached.
