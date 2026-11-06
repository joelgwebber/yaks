---
id: yaks-bca6
title: 'Design: session notes in a farm (narrative plus references to yak notes)'
status: hairy
type: task
priority: 2
created: '2026-11-06T04:28:29Z'
updated: '2026-11-06T04:56:54Z'
parent: yaks-1c89
tags:
- design
- doks
---

Settle the shape of a session note in a farm before building the command (yaks-1c89).

Hearth's note (`hearth-session-notes`) has four sections: Summary, Turns (one verdict per request: good, partial, failed, unclear), Saved, Lessons. It lives at `raw/sessions/YYYY/MM/local-<person>-<host>-<day>-<time>-<title>.review.md` with `type: session-review`, and a delegated sub-session writes none: it reports friction to its parent instead.

In a farm, much of a session's work is already recorded in yak notes and shear notes. The design here: a session note points at those rather than repeating them.

---
▸ 2026-11-06T04:28:54Z [herd-upkeep]

Decision (Joel, 2026-11-05, in the assessment thread): a session note references what yak notes already record instead of restating it.

- A session heavy on yak work cites those yaks, and the specific notes, by link; its own text is a brief narrative tying them together.
- A session heavy on discussion (no yak captures it) carries relatively more exposition in the note itself.
- Pointing at specific notes needs a link form for one note: yaks-c5bd.

Rejected: Hearth's four sections restated in full (duplicates yak notes); shrinking a note to verdicts, lessons and bare yak ids (loses the narrative and the discussion).

---
▸ 2026-11-06T04:28:54Z [herd-upkeep]
kind: ask
needs: human
---

asked: needs human

Four details to settle before yaks-c5bd and yaks-17f5 are built; my lean on each.

1. **Link form for one note.** Lean: a fragment with the note's timestamp, `yak-0af1#2026-11-04T22:13:03Z`, since timestamps are already in every note header and survive `--drop-request` (an index would shift). `doctor` flags the rare second shared by two notes, and `yaks update` prints the new note's link so agents never construct one by hand.
2. **Sections.** Lean: keep Hearth's four (Summary, Turns, Saved, Lessons) so reflection reads farm and Hearth notes alike, but let Turns and Saved items be links (`- good: landed the parser, [note](yak-0af1#...)`), and allow an optional `## Discussion` for exposition no yak holds. Lessons stay in full: they are the part no yak note has.
3. **Path.** Lean: `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`, `type: session-review`, Hearth's frontmatter fields (person, host, harness, day, saved time, verdict counts).
4. **Back-links.** Lean: no note on each yak a session touched (noise); `yaks refs <id> --incoming` finds the session notes that cite it.

---
▸ 2026-11-06T04:31:03Z [sched-1]
kind: answer
needs: agent
---

1. I was thinking the same thing. They really shouldn't end up conflicting.
2. SGTM
3. Also SGTM.
4. Also, also SGTM.

---
▸ 2026-11-06T04:38:03Z [herd-upkeep]
kind: pickup
---

picked up

herd-upkeep: recording the answers and building on them (yaks-c5bd, yaks-17f5).

---
▸ 2026-11-06T04:56:54Z [herd-upkeep]

Decision (Joel, 2026-11-06, answering the ask): all four leans, with one refinement to 1.

- **Note links** are `<id>#<timestamp>`. Notes written in the same second (a shear note and its move, an ask and its decision) share a timestamp often, so the later ones take `-1`, `-2` in file order, as GitHub numbers duplicate headings; the first stays the plain timestamp. Built in yaks-c5bd.
- **Sections:** Summary, Turns, Saved, Lessons, optional Discussion; Turns and Saved items may be links.
- **Path:** `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`, `type: session-review`, `source: session-note`.
- **Back-links:** none written; the session notes citing a yak are found with `yaks find <id> --kind sessions`.
