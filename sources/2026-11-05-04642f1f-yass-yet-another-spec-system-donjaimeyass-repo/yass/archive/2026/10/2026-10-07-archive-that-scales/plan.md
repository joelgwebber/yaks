# Archive that scales: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 changes where archives go.** `cmdArchive` ([`commands.go`](../../../internal/yass/commands.go)) writes `archived:` into the frontmatter and moves the change to `archive/<YYYY>/<MM>/<name>/` ([design §1–2](design.md)). The loader in [`repo.go`](../../../internal/yass/repo.go) (`load`, `resolve`, `check`, `checkDeps`, `checkQueues`) walks year and month folders as well as flat entries, so a half-migrated folder works. The hook ([`hook.go`](../../../internal/yass/hook.go)) learns that an archived change's folder is three levels under `archive/`, and accepts archiving into months ([design §7](design.md)). The yass folder README and `AGENTS.md` templates ([`templates/`](../../../internal/yass/templates/)), the top-level [`README.md`](../../../README.md) and [`docs/monorepo.md`](../../../docs/monorepo.md) describe the layout, the `archived:` field, the reworded rule 5, and splitting busy folders into team folders.

**M1 also adds migrations to `yass upgrade`.** A new `migrate.go` holds a list of steps ([design §5](design.md)); [`upgrade.go`](../../../internal/yass/upgrade.go) runs them for every yass folder after writing YASS's files, including folders outside the repo, using git in each folder's own repo. The first step moves a flat archive into months. The hook accepts that move. This repo's archive and Jaime's private plans folder are migrated with the released step, as its first real runs.

**M2 reads only names.** Archived changes load as names (and their pieces' names) without reading `change.md`; titles and frontmatter are read on first use, which only `status --archived`, `status <archived change>` and `yass decisions` need. [`tests/bench.sh`](../../../tests/bench.sh) adds 10,000 archived changes across its team folders and months, timed against the existing 1s `yass status` target. This touches the same loader as the in-flight `speed-at-scale` piece of `monorepos-at-scale-and-plans-repos`; both keep its bench passing.

**M3 adds `yass decisions`.** A new `decisions.go` reads the `## Decisions` section of each change's and piece's `change.md`, active and archived, in every yass folder, parsing who and date from each entry's last parenthesis ([design §6](design.md)). Each result carries the entry's date and the change's `created:` and `archived:`; filters use the entry's date, else `archived:`, else `created:`. Results are newest first, cut at `--limit`, with a line saying how many were left out. `--json` emits an array of objects with the same fields. The change templates, `yass-work` and the `AGENTS.md` section ask for dated entries; [`yass-log`](../../../kit/.agents/skills/yass-log/SKILL.md) and [`yass-shape`](../../../kit/.agents/skills/yass-shape/SKILL.md) use `yass decisions` (and `yass status --archived`) instead of grepping `archive/`. Playbook edits are made in `kit/` and reach this repo through `yass upgrade`.

**M4 adds `keep` and `yass evict`.** [`config.go`](../../../internal/yass/config.go) learns `archive: { keep: N }`. `yass status` and `yass archive` add a note (not a warning, so `--strict` still passes) when a folder is past `keep`. A new `yass evict` removes whole months and writes their manifests ([design §3–4](design.md)); the loader reads `.evicted` files so evicted names still resolve; `status --archived` summarizes them. The hook accepts evictions. The docs cover `keep`, eviction, clone size, and a scheduled CI recipe that runs `yass evict` and opens a PR.

**M5 reads evicted months from git, and finds citations.** `yass decisions --cites <sha>` searches boxes for `code:` citations of that commit, in the tree and, through the manifests, in evicted months; `yass-log`'s "starting from code" step uses it instead of grepping the yass folders. `yass decisions` reads only the manifests until a query's date range or `--change` reaches an evicted month; then it reads those months' `change.md` files from the manifest's commit with one `git cat-file --batch`. A missing commit (shallow clone) is reported with the months it covers and `git fetch --unshallow`; the other results still print.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a finished change, when you run `yass archive` on it, then its `change.md` frontmatter has `archived:` with the current UTC time, and nothing else in the folder changed beyond what `yass archive` already marks — verify: e2e
- [x] AC2 (R2) Given that archive, then the change is at `archive/<YYYY>/<MM>/<name>/` for the `archived:` date, with its pieces inside it, and the move is staged — verify: e2e
- [x] AC3 (R2) Given a change whose name's date is in an earlier month than today, when it's archived, then it lands in today's month — verify: e2e
- [x] AC4 (R2) Given a change with the same name already archived in some month, when you archive it, then `yass archive` refuses and names where the other one is — verify: e2e
- [x] AC5 (R3) Given a yass folder with some archived changes flat and some in months, then `yass status --archived` lists both, and `follows:`, `blocked:` and `queue.md` resolve names in either without warnings — verify: e2e
- [x] AC6 (R3) Given a change folder under `archive/<YYYY>/<MM>/`, then `yass status <name>` finds it, as with a flat one — verify: e2e
- [x] AC7 (R4, R21) Given a yass folder in git with a flat archive, when you run `yass upgrade`, then each archived change moves to the month of the commit that added its `change.md` under `archive/`, its `archived:` is that commit's date, and the output names the step and says to commit the move on its own — verify: e2e
- [x] AC8 (R4) Given a flat archive in a yass folder outside git (`yass.yaml` `path:` to a plain folder), when you run `yass upgrade` from the repo, then each change moves to the month of the date in its name, with `archived:` that date at 00:00 UTC — verify: e2e
- [x] AC9 (R4) Given a yass folder in a different git repo from the code (`yass.yaml` `path:`), when you run `yass upgrade`, then the dates come from that repo's history and the commit command it prints runs in that repo — verify: e2e
- [x] AC10 (R21) Given a yass folder already migrated, when you run `yass upgrade` again, then the migration step does nothing and says nothing — verify: e2e
- [x] AC11 (R21) Given the migration steps in the source, then each is one entry in a list with its own needed-check, and removing the archive step leaves the other steps and `yass upgrade` working — verify: go test (a step list without it runs clean)
- [x] AC12 (R4, R1) Given a migration commit or an archive into a month, when it's committed with the hook on, then the hook passes it; and given an edit, deletion or other move under `archive/`, then the hook still flags it — verify: e2e
- [x] AC13 (R5) Given this repo's flat archive and Jaime's private plans folder, when `yass upgrade` built from this piece runs on them (at the version their files are stamped with, so it writes no files and only migrates), then every archived change is in its month with `archived:` set, `yass status --strict` passes, and each migration is committed on its own — verify: manual: run it on both, check `yass status --archived` and the commits
- [x] AC14 (R19) Given a new `yass init`, then the yass folder README describes `archive/<YYYY>/<MM>/` — verify: e2e (the text); manual: review
- [x] AC15 (R19, R24) Given `README.md` and `docs/monorepo.md`, then they show the monthly layout and the `archived:` field, and recommend that a yass folder archiving more than about 1,000 changes a month split into team folders — verify: manual: review

### M2
- [x] AC16 (R6) Given archived changes whose `change.md` is unreadable, when you run `yass status` (no `--archived`), then it succeeds without errors about them, and `follows:` and `blocked:` naming them still resolve — verify: e2e (`chmod 000` on archived `change.md` files)
- [x] AC17 (R6) Given the same setup, when you run `yass status --archived`, then it reports the unreadable ones — verify: e2e
- [x] AC18 (R7) Given `tests/bench.sh`'s repo with 200 yass folders, 2,000 active changes and 10,000 archived changes across months, then `yass status` stays under 1s — verify: `tests/bench.sh`

### M3
- [x] AC19 (R8) Given active and archived changes in two yass folders with `## Decisions` entries, when you run `yass decisions`, then each entry prints once on its own line with its change, who, the decision, its own date if it has one, and the change's `created:` and `archived:` dates — verify: e2e
- [x] AC20 (R8) Given a piece's `## Decisions`, then its entries are listed under the piece's name — verify: e2e
- [x] AC21 (R22) Given an entry `- X - Y (Jaime, 2026-10-07)`, then who is `Jaime` and its date is 2026-10-07; given `- X - Y (claude, agreed by Jaime)`, then who is the whole credit and it has no date of its own — verify: go test
- [x] AC22 (R9) Given entries across several months, when you pass `--since` and `--until`, then only entries whose date (their own, else the change's `archived:`, else `created:`) falls in the range print — verify: e2e
- [x] AC23 (R9) Given `--about "squash merge"`, then only entries containing those words, case-insensitively, in the decision or the change's title print — verify: e2e
- [x] AC24 (R9) Given `--change B` where B follows A and C follows B, then entries from A, B and C print, and none from unrelated changes — verify: e2e
- [x] AC25 (R9) Given more matching entries than `--limit` (50 by default), then the newest print and a last line says how many were left out; with fewer, no such line — verify: e2e
- [x] AC26 (R9) Given no yass folder, or no decisions matching, then `yass decisions` says so and exits 0 — verify: e2e
- [x] AC27 (R23) Given `--json`, then the output is one JSON array whose objects have `change`, `title`, `who`, `decision`, `date`, `created` and `archived` (null when unknown), and the same filters apply — verify: e2e (parsed with `python3 -m json.tool`)
- [x] AC28 (R22) Given the change templates, `yass-work` and the `AGENTS.md` section, then they ask for `(<who>, <YYYY-MM-DD>)` on new entries — verify: e2e (template text); manual: review
- [x] AC29 (R10, R25) Given `yass-log` and `yass-shape`, then they look up past decisions and earlier work with `yass decisions` and `yass status --archived`, and nothing in the kit greps or walks `archive/` — verify: manual: review; `grep -rn "archive" kit/` shows no direct reads
- [x] AC30 (R10) Given `yass-log` asked why this repo's hook leaves existing hooks alone, then it finds the decision through `yass decisions` — verify: manual: ask it

### M4
- [x] AC31 (R11) Given `archive: { keep: 3 }` in a yass folder's `yass.yaml`, then that folder's limit is 3 and other folders keep 2,000; an unknown key under `archive:` warns like other unknown settings — verify: go test (config); e2e
- [x] AC32 (R14) Given a folder past `keep`, when you run `yass status` and `yass archive`, then each prints a note naming the folder, how far past it is, and `yass evict`; `yass status --strict` still exits 0; under `keep`, no note — verify: e2e
- [x] AC33 (R12) Given a folder 5 past `keep` whose oldest month holds 3 and next holds 4, when you run `yass evict`, then both months are deleted, the output names them with their counts, says to commit it on its own, and the current month is never touched — verify: e2e
- [x] AC34 (R12) Given a folder at or under `keep`, then `yass evict` changes nothing and says so — verify: e2e
- [x] AC35 (R12) Given a month with uncommitted changes, or a yass folder outside git, then `yass evict` refuses and says why — verify: e2e
- [x] AC36 (R13) Given an evicted month, then `archive/<YYYY>/<MM>.evicted` has the commit that last touched the month's folder, its path, and its change names sorted, as in design §3 — verify: e2e
- [x] AC37 (R13) Given two branches from the same commit that each run `yass evict` on the same month and commit, when one is merged into the other, then the merge has no conflicts — verify: e2e
- [x] AC38 (R13) Given a month evicted on a branch that was then squash-merged, then the manifest's commit is still reachable from main — verify: e2e (`git merge --squash`, then `git cat-file -e`)
- [x] AC39 (R15) Given an active change with `follows:` or `blocked:` naming an evicted change, then `yass status` resolves it without warnings, and reads no git history — verify: e2e (works with the evicted commit missing)
- [x] AC40 (R16) Given evicted months, when you run `yass status --archived`, then it lists what's in the working tree and ends with one line per yass folder giving the evicted count and months — verify: e2e
- [x] AC41 (R12, R13) Given an eviction commit with the hook on, then the hook passes it; and given a deletion under `archive/` without a matching `.evicted` file, then it still flags it — verify: e2e
- [x] AC48 (R19) Given a new `yass init`, then the yass folder README's rule 5 says the archive is append-only except for eviction by `yass evict`, and the AGENTS.md section says the same; `yass upgrade` brings existing ones in line — verify: e2e (the text)
- [x] AC42 (R20) Given `docs/monorepo.md`, then it explains `keep`, `yass evict`, that eviction bounds the working tree but not `.git` (partial clones or a plans repo for that), and gives a scheduled CI recipe that runs `yass evict` and opens a PR — verify: manual: review

### M5
- [x] AC43 (R17) Given an evicted month and a query whose dates reach it, when you run `yass decisions`, then decisions from its changes print as if they were in the tree — verify: e2e
- [x] AC44 (R17) Given `--change` naming an evicted change, or one following it, then its decisions print — verify: e2e
- [x] AC45 (R17) Given a query whose dates don't reach any evicted month, then it reads no git history: it succeeds with the same results in a clone where the evicted commits are missing — verify: e2e (shallow clone)
- [x] AC46 (R18) Given a shallow clone missing an evicted month's commit and a query that reaches it, then the other results print, a line names the months it couldn't read and `git fetch --unshallow`, and it exits 0 — verify: e2e
- [x] AC49 (R25) Given boxes citing a commit (`code: a1b2c3d`) in an active change, an archived one and a piece, when you run `yass decisions --cites a1b2c3d` (or a longer prefix of the same commit), then each box prints with its change, and nothing else does — verify: e2e
- [x] AC50 (R25) Given a box citing a commit in an evicted change, then `--cites` finds it from git, the way decisions from evicted months are read; and with that month's commit missing, it says so as AC46 does — verify: e2e
- [x] AC47 (R17) Given 1,000 evicted changes in range, then `yass decisions` reads them with a constant number of git processes, not one per change — verify: go test (counts git invocations)

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. **`2026-10-07-archive-into-months`** (AC1–AC6, AC12's archiving half, AC14, AC15). The layout everything else builds on, so it goes first.
2. **`2026-10-07-migrations-in-upgrade`** (AC7–AC11, AC12's migrating half, AC13). Needs piece 1's layout. AC13 runs on a local build, before release, so the migration lands on main with the code.
3. **`2026-10-07-archive-names-only`** (AC16–AC18). Needs piece 1's loader; can go alongside piece 2.
4. **`2026-10-07-yass-decisions`** (AC19–AC28, AC30). Needs piece 1's `archived:` dates; can go alongside pieces 2 and 3.
5. **`2026-10-07-keep-and-evict`** (AC31–AC42, AC48). Needs piece 3's names-only loader, which it extends with manifests.
6. **`2026-10-07-decisions-from-history`** (AC43–AC47, AC49, AC50, and AC29 once `yass-log` uses `--cites`). Needs pieces 4 and 5.

`monorepos-at-scale-and-plans-repos` (R13, citation checks in `yass archive`) and `stacked-prs-and-squash-merges` (M2, marking delivered criteria on archive) also change `cmdArchive`. They don't depend on each other; whichever lands second rebases.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- `go test ./...`, `tests/e2e.sh`, `tests/examples.sh` and `tests/bench.sh` pass, on macOS and Linux in CI.
- This repo and Jaime's private plans folder are migrated by `yass upgrade` built from source (AC13), and `yass status --strict` passes on both.
- A dry run of eviction on a generated folder with `keep: 100`: evict, commit, then ask `yass decisions` about an evicted month in a full clone and in a shallow one.
- `yass-log` answers a "why did we…" question in this repo through `yass decisions` (AC30).
