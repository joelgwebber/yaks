---
id: yaks-8dbb
title: 'Tidy batch 2026-10-27: work through Joel''s 122 tidy requests'
status: shorn
type: chore
priority: 2
created: '2026-10-27T01:14:29Z'
updated: '2026-10-27T02:37:29Z'
labels:
- markdown
---

---
▸ 2026-10-27T01:14:30Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-27T01:14:30Z [delta-lead]

Claim (delta-lead, yakherd). Joel queued 122 `--kind tidy` requests after `pickup --drop-request` landed (yaks-092f). The farm is committed at `9c9cabc`, the state before any tidy.

- Four workers, tidy-1 to tidy-4, each with a disjoint list of about 30 ids (`/tmp/tidy/batch-0{0..3}` on this machine).
- Each yak: follow its request literally (edit the file at `yaks path`, then `yaks pickup <id> --drop-request`), and add no notes.
- Judge: `/tmp/tidy/check.py <ids>` against `9c9cabc`. It checks that the frontmatter is unchanged apart from `needs:`/`updated:`, that exactly the latest request note is removed, and that the words are identical in order (ignoring whitespace, fence lines and added list markers). It also reports what `--run-on` still flags.
- delta-lead re-runs the checker over all 122 at the end and spot-reads a sample in the TUI.

---
▸ 2026-10-27T01:15:25Z [delta-lead]

Fan-out (delta-lead): yaks-00c4 done by hand to validate the brief and the checker (OK). tidy-1 to tidy-4 on claude-sonnet-5-5 (cheap, mechanical, per AGENTS.md), with batches of 30/31/31/29 ids; the brief is /tmp/tidy/BRIEF.md.

---
▸ 2026-10-27T02:37:29Z [delta-lead]

Done (delta-lead). All 122 tidy requests are handled. Each request note was removed with `pickup --drop-request`, and no other notes were added.

- Checker over all 122 against `9c9cabc`: 122 OK, 0 FAIL. Words unchanged in order; frontmatter unchanged apart from `needs:`/`updated:`; exactly the request note gone.
- `yaks inbox --for agent` is empty and `yaks doctor` is clean.
- 11 of the 122 still match `--run-on`. I checked each: they are list-item continuation lines or prose wrapped across lines, which renders as intended, so the brief said to leave them. yaks-b32e (a 388-line research note) is most of them.
- Headless TUI spot check: yaks-e545's pasted `yaks lanes` table now renders as a fenced block; yaks-7eea's lists render as bullets.
- Workers: tidy-1 30, tidy-2 31, tidy-3 31, tidy-4 29, delta-lead 1 (yaks-00c4, the pilot).

Lesson for the brief and the check script: `yaks` only finds the farm from inside a checkout, and three workers lost a first run by working from `/tmp/tidy`. A future brief should say to run from the repo.

---
▸ 2026-10-27T02:37:29Z [delta-lead]

moved: shaving -> shorn
