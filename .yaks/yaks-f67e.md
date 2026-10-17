---
id: yaks-f67e
title: 'Fix: markdown link URL target swallows the closing paren'
status: shaving
type: bug
priority: 2
created: '2026-10-17T04:24:29Z'
updated: '2026-10-17T04:31:30Z'
parent: yaks-ea13
labels:
- markdown
- tui
---

`[Links](https://example.com/)` in a body yields a `Target::Url("https://example.com/)")`: `scan_body_links` (src/tui/detail.rs) reads a bare URL up to the next whitespace. Fix the scan to stop at an unbalanced `)` (and trailing `.,;:!?` punctuation), the GFM autolink rule. A test that fails first: the URL target for `[x](https://e.com/)` and for `see https://e.com/a).` .

Standalone now; the renderer core (pulldown-cmark) later replaces the scan for markdown links, but bare-URL autolinking keeps this rule.

---
▸ 2026-10-17T04:31:30Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:31:30Z [delta-lead]
Claim (yakherd delta-lead, 2026-10-17): owner **links-1**.
Scope: `scan_body_links` (and a small helper if needed) in src/tui/detail.rs only; tests in that file's test modules. Rule: a bare URL ends at whitespace, then trailing `.,;:!?'"` and an unbalanced trailing `)` are trimmed (GFM extended-autolink rule); balanced parens inside a URL (e.g. Wikipedia links) stay.
Evidence: failing-first tests for `[x](https://e.com/)`, `see https://e.com/a).`, `https://en.wikipedia.org/wiki/Foo_(bar)`; a headless frame of a yak body with such links, with the link span visibly ending before the paren, attached. Judge: coordinator at landing.
Do NOT regenerate docs/assets docshots (several workers would collide on the SVGs; the coordinator regenerates them after landing). Re-accept any insta snapshot your change moves (`INSTA_UPDATE=always cargo test`, or `cargo insta accept` if installed) and read every changed .snap diff before committing. `cargo fmt` reformats files nobody touched (src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs): run `cargo fmt -- <your files>` and `git checkout` those three if they changed. Docs/help parity per AGENTS.md (docs/tui.md, help_content in src/tui/render.rs) where behaviour a user sees changes.
