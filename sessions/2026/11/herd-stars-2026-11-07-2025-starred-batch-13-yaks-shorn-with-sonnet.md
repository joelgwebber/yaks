---
type: session-review
title: 'Starred batch: 13 yaks shorn with Sonnet workers'
source: session-note
actor: herd-stars
person: Joel Webber
host: joel
harness: delta
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-07
reviewed: '2026-11-08T01:25:46Z'
good: 2
---

## Summary

Joel starred 13 yaks. herd-stars claimed them, fanned eight out to Sonnet 5.5 workers (stars-thread, stars-dokattach, stars-zoom, stars-peek, stars-form, stars-tz, stars-fonts) plus one default-model worker for the keys rethink (stars-keys), and did yaks-a036, yaks-c316, yaks-1a5f and yaks-8fd1 itself. Everything landed on `main` (local), toque's `main` was pushed to origin for the font subset, and the binary was installed.

## Turns

- good: "PTAL at the starred yaks" — 11 shorn in the first pass, the two blocked ones asked or explained.
- good: "plow ahead and land everything" — yaks-ac6a, yaks-4f4d, yaks-8fd1 and the docshot regeneration landed and pushed to local `main`.

## Saved

- yaks-000b: Mouse tweaks
  - [pickup: herd-stars: both edit modes (form row click-focus, and click-to-position in t...](yaks-000b#2026-11-07T17:04:27Z)
  - [Claim (yakherd herd-stars): worker `stars-form`, after yaks-255d, in the same...](yaks-000b#2026-11-07T17:05:33Z)
  - [answer: Yakherd herd-stars: yes, enable it: `features = "arboard", "mouse-support"` o...](yaks-000b#2026-11-07T17:38:59Z)
  - [landed: f0dafed7 yaks-000b: click focuses form rows and chips, and places the...](yaks-000b#2026-11-07T17:43:50Z)
- yaks-1a5f: Regenerate the docs/assets docshots in one commit (created, hairy -> shorn)
  - [Done by herd-stars in `5f923cb8`: all 32 docshots regenerated in one commit....](yaks-1a5f#2026-11-08T00:55:25Z)
- yaks-255d: Can't edit empty description. (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-form`, who then takes yaks-000b in...](yaks-255d#2026-11-07T17:05:33Z)
  - [landed: 108a5291 yaks-255d: the edit form has the description editor, like cr...](yaks-255d#2026-11-07T17:43:44Z)
- yaks-4736: Editor parity: audit and fill edtui's emacs/standard bindings against its vim set (created)
- yaks-4f4d: Wider embedded font subset for svg rendering (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-fonts`, in the toque repo (attached...](yaks-4f4d#2026-11-08T00:52:10Z)
  - [herd-stars: landed in toque as `f2100c5` + `4fdb374`, pushed to toque's `orig...](yaks-4f4d#2026-11-08T01:19:39Z)
- yaks-6919: TUI phase 7: the key pass (collisions, which-key, overrides)
  - [pickup: herd-stars: Joel's answer defers the leader/chord questions to yaks-7c6e; not...](yaks-6919#2026-11-07T17:04:26Z)
- yaks-7c6e: Holistic key-binding rethink, after the unified-model phases land (hairy -> shaving)
  - [pickup: herd-stars: implementing the six answered points plus the {/} tab idea, after...](yaks-7c6e#2026-11-07T17:04:27Z)
  - [Claim (yakherd herd-stars): worker `stars-keys`, to start once the starred TU...](yaks-7c6e#2026-11-07T17:40:32Z)
  - [landed: 61cda058 yaks-7c6e: one key family, h/l never leave the pane, grouped...](yaks-7c6e#2026-11-07T18:19:19Z)
  - [landed: a29187b1 yaks-7c6e: docs: key families and rules, the grouped ? popup...](yaks-7c6e#2026-11-07T18:19:21Z)
  - [herd-stars: landed `61cda058` (code, worker `4fc439e5`) and `a29187b1` (docs,...](yaks-7c6e#2026-11-07T18:24:44Z)
- yaks-8fd1: yaks render: rasterize an SVG to a PNG with the embedded fonts (no browser) (hairy -> shorn)
  - [Claim (yakherd herd-stars): worker after yaks-4f4d lands in toque. Open point...](yaks-8fd1#2026-11-08T00:52:11Z)
  - [Done by herd-stars in `cddd0bcd`: `yaks render <svg> -o <png> --scale N` (src...](yaks-8fd1#2026-11-08T01:25:06Z)
- yaks-a004: Unify "peek" across yaks + doks (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-peek`.](yaks-a004#2026-11-07T17:05:34Z)
  - [landed: Landed by herd-stars as `e08cfa4f` (worker `9c9b9362`), by cherry-pick: `run_...](yaks-a004#2026-11-07T17:43:43Z)
- yaks-a036: Consistent copy behavior across yaks/docs (hairy -> shorn)
  - [Done by herd-stars in `3e43d41e`: `y` in the text viewer with nothing selecte...](yaks-a036#2026-11-07T17:38:39Z)
- yaks-ac6a: Recurring datetimes keep their wall-clock time across DST; a farm time zone (hairy -> shaving)
  - [ask: Three calls before this is built (herd-stars), my lean on each:](yaks-ac6a#2026-11-07T17:04:37Z)
  - [pickup: herd-stars: building it as answered: farm-wide `timezone:`, chrono-tz embedded.](yaks-ac6a#2026-11-08T00:49:15Z)
  - [Claim (yakherd herd-stars): worker `stars-tz`, per Joel's answers (2026-11-08...](yaks-ac6a#2026-11-08T00:52:09Z)
  - [landed: 913e4f4a yaks-ac6a: farm time zone (timezone: + chrono-tz); recurring...](yaks-ac6a#2026-11-08T01:13:19Z)
  - [herd-stars: landed `913e4f4a` (worker `cd5faac7`, clean squash); gate `cargo...](yaks-ac6a#2026-11-08T01:16:19Z)
- yaks-c316: Dok conventions: dated doks, a created date, and folder layout (hairy -> shorn)
  - [pickup: herd-stars: doing it myself: dok new stamps created:, and the folders-follow-...](yaks-c316#2026-11-07T17:04:28Z)
  - [Done by herd-stars in `2206248e`, per Joel's two answers.](yaks-c316#2026-11-07T17:38:40Z)
- yaks-db66: Images and files in doks and session notes: a sibling-directory convention (hairy -> shaving)
  - [pickup: herd-stars: blessed sibling directory; a worker builds attach/mv/doctor/weigh...](yaks-db66#2026-11-07T17:04:28Z)
  - [Claim (yakherd herd-stars): worker `stars-dokattach` builds the commands; her...](yaks-db66#2026-11-07T17:05:35Z)
  - [landed: 4def079b yaks-db66: attach to doks and session notes via a sibling di...](yaks-db66#2026-11-07T17:28:41Z)
  - [herd-stars: landed `4def079b` (worker `e4c2dd0d`, clean squash) and the skill...](yaks-db66#2026-11-07T17:38:38Z)
- yaks-e5f9: Image zoom affordances (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-zoom`.](yaks-e5f9#2026-11-07T17:05:34Z)
  - [landed: Landed by herd-stars as `04907036` (worker `c46e089a`), by cherry-pick: `docs...](yaks-e5f9#2026-11-07T17:37:45Z)
- yaks-f4ea: YAKS_THREAD: a harness-neutral thread id for moves and session notes (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-thread`.](yaks-f4ea#2026-11-07T17:05:35Z)
  - [landed: Landed by herd-stars as `b7cebb1b` (worker `797b367a`), by cherry-pick: a con...](yaks-f4ea#2026-11-07T17:33:01Z)

## Lessons

- Four parallel workers each editing `docs/tui.md`'s one giant `A`/`O` table row and `run_cmd`'s no-op arm produced cherry-pick conflicts at every landing; a single shared line is a collision point even when the briefs split files by symbol.
- A key-binding change silently broke docshot scenes that pressed the old key (`l` to open the pane): the scenes shot an empty list and nothing failed. Rasterize and look at every regenerated docshot.
- Parallel Rust workers fill a laptop disk (about 3–5 GB of `target/` each); the gate then fails with ENOSPC that looks like test failures. Check `df` before fanning out and clean landed workers' targets.
- `cargo fmt -- <file>` also rewrites `tests/commits.rs`; now listed in AGENTS.md.
