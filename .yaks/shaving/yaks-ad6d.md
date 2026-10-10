---
id: yaks-ad6d
title: 'yaks sheds: list the current shed too (marked), and let changes/--shed/answer name it'
type: feature
priority: 2
created: '2026-10-10T17:21:10Z'
updated: '2026-10-10T19:05:06Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

Joel (2026-10-10): `yaks sheds` omits the shed you are in, so you cannot see where you are, and `changes <name>`, `--shed <name>` and `answer <id>@<name>` cannot name your own shed (from inside this clone `yaks changes <its own name>` fails; from the primary `main` was mis-resolved, yaks-3344). Direction agreed (A): list the current shed as a row, marked, and let every selector accept it. The semantics of what "changes" are measured against, per shed, are the open design discussion (see the notes) and decide the code: do not start before that is settled.

---
▸ 2026-10-10T19:05:06Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-10T19:05:06Z [delta-lead]
Claim (coordinator). Owner: here-1. Binding: the 2026-10-10 Decisions on yaks-ee0a (read them: 1, 2, 3, 4, 7), and yaks-e9bb (landed, f03a81c: one fork-point baseline per shed from every viewer).
Why now: after yaks-e9bb the PRIMARY's row, seen from a clone, is measured from the first entry of the primary's own reflog and shows 154 new yaks, 23 moved, 172 notes (real machine, 2026-10-10). That is noise until the primary has its own baseline.

Goal (all of it, one rule: `cwd` never changes an answer, only the DEFAULT shed):
1. `yaks sheds` lists EVERY checkout including the one you are in, which is marked (text: a `(here)` tag on the `name:` line and a `*` in the row's first column or equivalent; JSON `current: true`, false elsewhere). Each row is measured like any other shed: fork point, working tree. The table rows are no longer "the others": say so in the `--help`/docs.
2. The PRIMARY's baseline is `origin/main` (Decision 4): its farm at `origin/main` (the remote-tracking ref `refs/remotes/origin/main`, falling back to `origin/HEAD`'s target) against its working tree: "what is not published". Use it wherever the primary is measured, from any viewer. When there is no usable `origin` ref (no remote, never fetched), fall back to the fork point and say `(vs fork)`-style honestly; record the choice as a Decision. A Delta clone's `local/main` is NOT used for the primary's own row (the primary IS local); it only matters to what a clone treats as "the primary's committed state", which this yak does not need.
3. Every selector accepts the current shed: `changes <name>`, `--shed <name>`, `answer <id>@<name>`, `main` (the viewer IS the primary: `main` now resolves to it; the yaks-3344 guard becomes: `main` resolves to the primary whether or not it is the current shed, and errors only when no primary is visible, e.g. from a clone on a shared-thread machine). `yaks changes` with NO argument means the current shed. `--shed <current>` is a no-op.
4. Commands that mean the OTHER sheds keep meaning that: `inbox --sheds`, `show --sheds` ("same in N sheds"), `sheds`'s ahead/behind notes. Audit every caller of `farm.sheds(...)` in src/main.rs and src/farm.rs and exclude the current shed where the command's meaning is "elsewhere"; list your audit (each caller, kept or excluded, why) in a note.
Scope: src/sheds.rs (`discover` keeps the viewer instead of `found.remove(top)`, `Shed` gets `current`, the primary baseline in `inspect`/`GitState`, `resolve`, render, `to_json`, tests), src/main.rs (`Changes { shed: Option }`, the callers above, help), src/farm.rs only for callers. Docs: docs/cli.md (sheds/changes/discover-related rows), README.md, `.agents/skills/yaks/SKILL.md` (sheds/changes rows and the `--shed` text), `Sheds`/`Changes` help in src/main.rs.
Out of scope: yak_changes and its counts (another worker, edit-2 on yaks-047a, owns them and will land first or second: do not touch the `edited` rule), `landed`/`stale` (yaks-ed5e, yaks-9277), the TUI, any write beyond what exists today.
Evidence (tests FAIL first): from three viewers (primary, a sibling git worktree, a Delta clone fixture) `yaks sheds --json` lists the same set of checkouts, the same per-shed `farm` for every shed and exactly one `current: true` that is the viewer; the primary row measured from `origin/main` shows only unpublished farm changes (a fixture primary with an origin ref one commit behind its HEAD whose farm changed); `yaks changes` with no argument equals `yaks changes <own name>`; `main` from the primary; `inbox --sheds`/`show --sheds` still exclude the viewer. Real machine: build your own binary, run `./target/release/yaks sheds` in your clone (read-only) and attach the output; the primary row must no longer show 154 new / 172 notes. Judge: coordinator.
