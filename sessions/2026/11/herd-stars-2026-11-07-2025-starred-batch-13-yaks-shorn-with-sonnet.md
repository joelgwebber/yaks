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
  - [pickup: herd-stars: both edit modes (form row click-focus, and click-to-position in t...](/yaks/yaks/yaks-000b.md#2026-11-07t170427z)
  - [Claim (yakherd herd-stars): worker `stars-form`, after yaks-255d, in the same...](/yaks/yaks/yaks-000b.md#2026-11-07t170533z)
  - [answer: Yakherd herd-stars: yes, enable it: `features = "arboard", "mouse-support"` o...](/yaks/yaks/yaks-000b.md#2026-11-07t173859z)
  - [landed: f0dafed7 yaks-000b: click focuses form rows and chips, and places the...](/yaks/yaks/yaks-000b.md#2026-11-07t174350z)
- yaks-1a5f: Regenerate the docs/assets docshots in one commit (created, hairy -> shorn)
  - [Done by herd-stars in `5f923cb8`: all 32 docshots regenerated in one commit....](/yaks/yaks/yaks-1a5f.md#2026-11-08t005525z)
- yaks-255d: Can't edit empty description. (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-form`, who then takes yaks-000b in...](/yaks/yaks/yaks-255d.md#2026-11-07t170533z)
  - [landed: 108a5291 yaks-255d: the edit form has the description editor, like cr...](/yaks/yaks/yaks-255d.md#2026-11-07t174344z)
- yaks-4736: Editor parity: audit and fill edtui's emacs/standard bindings against its vim set (created)
- yaks-4f4d: Wider embedded font subset for svg rendering (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-fonts`, in the toque repo (attached...](/yaks/yaks/yaks-4f4d.md#2026-11-08t005210z)
  - [herd-stars: landed in toque as `f2100c5` + `4fdb374`, pushed to toque's `orig...](/yaks/yaks/yaks-4f4d.md#2026-11-08t011939z)
- yaks-6919: TUI phase 7: the key pass (collisions, which-key, overrides)
  - [pickup: herd-stars: Joel's answer defers the leader/chord questions to yaks-7c6e; not...](/yaks/yaks/yaks-6919.md#2026-11-07t170426z)
- yaks-7c6e: Holistic key-binding rethink, after the unified-model phases land (hairy -> shaving)
  - [pickup: herd-stars: implementing the six answered points plus the {/} tab idea, after...](/yaks/yaks/yaks-7c6e.md#2026-11-07t170427z)
  - [Claim (yakherd herd-stars): worker `stars-keys`, to start once the starred TU...](/yaks/yaks/yaks-7c6e.md#2026-11-07t174032z)
  - [landed: 61cda058 yaks-7c6e: one key family, h/l never leave the pane, grouped...](/yaks/yaks/yaks-7c6e.md#2026-11-07t181919z)
  - [landed: a29187b1 yaks-7c6e: docs: key families and rules, the grouped ? popup...](/yaks/yaks/yaks-7c6e.md#2026-11-07t181921z)
  - [herd-stars: landed `61cda058` (code, worker `4fc439e5`) and `a29187b1` (docs,...](/yaks/yaks/yaks-7c6e.md#2026-11-07t182444z)
- yaks-8fd1: yaks render: rasterize an SVG to a PNG with the embedded fonts (no browser) (hairy -> shorn)
  - [Claim (yakherd herd-stars): worker after yaks-4f4d lands in toque. Open point...](/yaks/yaks/yaks-8fd1.md#2026-11-08t005211z)
  - [Done by herd-stars in `cddd0bcd`: `yaks render <svg> -o <png> --scale N` (src...](/yaks/yaks/yaks-8fd1.md#2026-11-08t012506z)
- yaks-a004: Unify "peek" across yaks + doks (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-peek`.](/yaks/yaks/yaks-a004.md#2026-11-07t170534z)
  - [landed: Landed by herd-stars as `e08cfa4f` (worker `9c9b9362`), by cherry-pick: `run_...](/yaks/yaks/yaks-a004.md#2026-11-07t174343z)
- yaks-a036: Consistent copy behavior across yaks/docs (hairy -> shorn)
  - [Done by herd-stars in `3e43d41e`: `y` in the text viewer with nothing selecte...](/yaks/yaks/yaks-a036.md#2026-11-07t173839z)
- yaks-ac6a: Recurring datetimes keep their wall-clock time across DST; a farm time zone (hairy -> shaving)
  - [ask: Three calls before this is built (herd-stars), my lean on each:](/yaks/yaks/yaks-ac6a.md#2026-11-07t170437z)
  - [pickup: herd-stars: building it as answered: farm-wide `timezone:`, chrono-tz embedded.](/yaks/yaks/yaks-ac6a.md#2026-11-08t004915z)
  - [Claim (yakherd herd-stars): worker `stars-tz`, per Joel's answers (2026-11-08...](/yaks/yaks/yaks-ac6a.md#2026-11-08t005209z)
  - [landed: 913e4f4a yaks-ac6a: farm time zone (timezone: + chrono-tz); recurring...](/yaks/yaks/yaks-ac6a.md#2026-11-08t011319z)
  - [herd-stars: landed `913e4f4a` (worker `cd5faac7`, clean squash); gate `cargo...](/yaks/yaks/yaks-ac6a.md#2026-11-08t011619z)
- yaks-c316: Dok conventions: dated doks, a created date, and folder layout (hairy -> shorn)
  - [pickup: herd-stars: doing it myself: dok new stamps created:, and the folders-follow-...](/yaks/yaks/yaks-c316.md#2026-11-07t170428z)
  - [Done by herd-stars in `2206248e`, per Joel's two answers.](/yaks/yaks/yaks-c316.md#2026-11-07t173840z)
- yaks-db66: Images and files in doks and session notes: a sibling-directory convention (hairy -> shaving)
  - [pickup: herd-stars: blessed sibling directory; a worker builds attach/mv/doctor/weigh...](/yaks/yaks/yaks-db66.md#2026-11-07t170428z)
  - [Claim (yakherd herd-stars): worker `stars-dokattach` builds the commands; her...](/yaks/yaks/yaks-db66.md#2026-11-07t170535z)
  - [landed: 4def079b yaks-db66: attach to doks and session notes via a sibling di...](/yaks/yaks/yaks-db66.md#2026-11-07t172841z)
  - [herd-stars: landed `4def079b` (worker `e4c2dd0d`, clean squash) and the skill...](/yaks/yaks/yaks-db66.md#2026-11-07t173838z)
- yaks-e5f9: Image zoom affordances (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-zoom`.](/yaks/yaks/yaks-e5f9.md#2026-11-07t170534z)
  - [landed: Landed by herd-stars as `04907036` (worker `c46e089a`), by cherry-pick: `docs...](/yaks/yaks/yaks-e5f9.md#2026-11-07t173745z)
- yaks-f4ea: YAKS_THREAD: a harness-neutral thread id for moves and session notes (hairy -> shaving)
  - [Claim (yakherd herd-stars): worker `stars-thread`.](/yaks/yaks/yaks-f4ea.md#2026-11-07t170535z)
  - [landed: Landed by herd-stars as `b7cebb1b` (worker `797b367a`), by cherry-pick: a con...](/yaks/yaks/yaks-f4ea.md#2026-11-07t173301z)

## Lessons

- Four parallel workers each editing `docs/tui.md`'s one giant `A`/`O` table row and `run_cmd`'s no-op arm produced cherry-pick conflicts at every landing; a single shared line is a collision point even when the briefs split files by symbol.
- A key-binding change silently broke docshot scenes that pressed the old key (`l` to open the pane): the scenes shot an empty list and nothing failed. Rasterize and look at every regenerated docshot.
- Parallel Rust workers fill a laptop disk (about 3–5 GB of `target/` each); the gate then fails with ENOSPC that looks like test failures. Check `df` before fanning out and clean landed workers' targets.
- `cargo fmt -- <file>` also rewrites `tests/commits.rs`; now listed in AGENTS.md.
