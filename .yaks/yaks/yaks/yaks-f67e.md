---
id: yaks-f67e
title: 'Fix: markdown link URL target swallows the closing paren'
status: shorn
type: bug
priority: 2
created: '2026-10-17T04:24:29Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-ea13
labels:
- markdown
- tui
needs: agent
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

---
▸ 2026-10-17T04:32:42Z [links-1]

links-1 start: Sat Oct 17 04:32:39 AM UTC 2026; path /home/joel/src/yaks/.delta/worktrees/8sw91qzt73am/yaks/.yaks/yaks-f67e.md

---
▸ 2026-10-17T04:37:30Z [links-1]

verify: `cargo test --workspace` -> PASS (exit 0)

[links-frame](yaks-f67e/links-frame.txt)

---
▸ 2026-10-17T04:37:51Z [links-1]

Decision: new helper `trim_url_end` in detail.rs, called from `scan_body_links`, trims after the whitespace-delimited scan (GFM extended-autolink rule). Rejected: teaching the scan about `[text](url)` syntax (that is the pulldown-cmark renderer's job later; bare-URL autolinking keeps this rule), and a regex.
Decision: trailing `)` is trimmed only when `)` outnumber `(` in the URL so far, so `Foo_(bar)` stays and `[x](url)` / `(url).` lose the paren; trimming loops, so `a).` and `a?!` both resolve. Trimmed set is `.,;:!?'"` per the claim note.
Summary: fixed in src/tui/detail.rs only. Tests (failed first with the trim call disabled: 3 failed; pass with it): url_in_markdown_link_stops_before_the_closing_paren, url_drops_trailing_paren_and_punctuation, url_keeps_balanced_parens. Gate `cargo test --workspace` PASS via yaks verify; `cargo test --release` 604+7+8+11+6 pass, 0 failed. Frame (attached, target/links-frame.txt) shows the body; headless text can't show the span, which the unit tests pin by (col,len). No user-visible docs/help describe URL extent (grep for autolink/bare URL/unbalanced in docs, README, render.rs, skills: no hits), so none changed. No snapshots moved. Note: .yaks/yaks-2362.md was modified in my tree by an external edit (not mine) - left uncommitted.

---
▸ 2026-10-17T04:37:51Z [links-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-f67e` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-f67e --drop-request`, which removes this request note, so no new note is left behind.
