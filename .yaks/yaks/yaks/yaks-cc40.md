---
id: yaks-cc40
title: 'Rename wiki pages to doks: the whole thing, one pass (schema 7)'
status: hairy
type: task
priority: 2
created: '2026-11-04T17:18:16Z'
updated: '2026-11-04T17:23:25Z'
tags:
- wiki
- cli
- tui
---

Joel's call (2026-11-03): rename what the farm calls its **wiki** and **pages** to **doks** (yaks, haks, doks), completely and at once. No aliases, no deprecation period, no old spelling left in the product: nobody but Joel uses these docs yet, so this is the cheap moment. Origin: [the haks research page](/doks/research/haks-orchestration-layer.md#naming-dok).

## Decisions made here (so the implementer doesn't re-open them)

- **A dok is a markdown document in the farm that is not a yak**: today's page. **Sources** and **schemas** keep their names: a source is saved outside material, a schema describes a kind of dok. The noun covers one thing.
- **`wiki/` becomes `doks/`**, farm-wide (`doks/topics/x.md`) and per herd (`<herd>/doks/`). Link form `/wiki/x.md` becomes `/doks/x.md`.
- **`yaks page` becomes `yaks dok`**, with the same subcommands (`list show new set query types`). `find --kind pages` becomes `--kind doks`, `promote --page` becomes `--dok`, and the doctor/check issue kinds follow (`page-problem` becomes `dok-problem`).
- **Schema type `Page Type` becomes `Dok Type`.**
- **The TUI's Wiki tab is the Doks tab.** It still lists sources too.
- **Hearth import keeps reading Hearth's `wiki/`** (that is Hearth's layout) but writes `doks/`.
- **Schema 6 becomes 7**, with a `yaks migrate` step. Every other command refuses a schema-6 farm and names `migrate`, as it already does for older ones.

## Scope

Use `grep -rniw -E 'wiki|pages?'` across `src/`, `tests/`, `docs/`, `.agents/skills/`, `README.md`, `AGENTS.md` and `RELEASING.md` to find every site; about 640 `wiki` hits in 78 files at the time of filing. Surfaces:

- **Layout and paths:** `RESERVED_HERD_NAMES` in [store.rs](code:src/store.rs#L402) (`wiki` becomes `doks`), `src/pages.rs` (module, `is_page_path`, wiki roots; rename the module to `doks.rs`), the doc classifier in `src/commit.rs` (so per-change commit messages say "doks"), `links.rs` and `refs.rs` (the `/wiki/` root), `find.rs` scope prefixes, `hearth_import.rs`, `mergedriver.rs`, `farm.rs` doctor/check text, `schemas.rs` (`Page Type`).
- **CLI:** the `Page` command and its help in `src/main.rs`, plus every doc comment and `--help` that says page or wiki (`find`, `promote`, `mv`, `source`, `check`, `refs`, `import-hearth`, `doctor`).
- **TUI:** `src/tui/wiki_tab.rs` becomes `doks_tab.rs`; `Cmd::WikiNew`/`WikiEdit` and their dotted names (`wiki.new`, `wiki.edit`, so a user's `keys:` override of them stops matching: say so in the changelog, there is no alias), `Ctx::Doc` rows, the tab label, hints, `?` help text, `follow.rs`, `viewmodel.rs`, `views_store.rs`, `history.rs`. Leave `DocPane` alone: it also shows attachments and images, not only doks.
- **Tests and snapshots:** rename the test files and fixtures (`tests/pages.rs` and others), then accept the insta snapshots after reading each diff: it should be wording only.
- **Docs and in-app help:** `docs/cli.md`, `docs/tui.md`, `docs/README.md`, `README.md`, `AGENTS.md`; the bundled skills `yaks` ("Pages" section, command table, `yaks page` rows) and `yaks-research` (and `yaks-coordinating`, `yaks-working`, `yaks-tracker` if they mention it); `src/brief.rs` notes line. The skill's own name `yaks-research` stays. Regenerate the docshots (`cargo test -p yaks docshots -- --ignored`), rename the `docs/assets/tui-wiki*.svg` files and their embeds.
- **This farm:** run the migration on it (below), retag the 34 yaks tagged `wiki` to `doks` (`yaks bulk`), and update the few design docs whose prose says wiki.

## The migration (the part that can go wrong)

`yaks migrate` gets a step that, for the farm root and for each herd:

- moves `wiki/` to `doks/` (`git mv` semantics, so history follows);
- rewrites farm-root links `/wiki/...` to `/doks/...` in yak descriptions and notes, in doks, and in sources' `source.md` link fields; leaves sources' saved originals and `extracted.md` untouched, since a source is a snapshot;
- rewrites `type: Page Type` in `schemas/*.md` to `Dok Type`;
- refuses a herd literally named `doks` (as the existing step does for new reserved names), and is idempotent.

**Pinned links need a decision.** `[x](/wiki/a.md@3f2a9c1)` reads the page as of that commit, when the path was still `wiki/a.md`. If the migration rewrites it to `/doks/a.md@3f2a9c1`, `yaks peek` finds nothing at that commit. Lean: rewrite them too, and teach the pin reader one rule: a `doks/` path that does not exist at the pinned rev is read from `wiki/` at that rev (only revs before the migration can hit it). Write a test with a real pre-migration commit. The alternative (leave pinned links on `wiki/`) keeps a second spelling in the farm forever.

Run it on a **copy** of this farm first (`YAKS_DIR=<copy> yaks migrate --dry-run`, then for real), then `yaks doctor --strict` and `yaks check` on the result: no broken links, no new findings.

## Consequences to handle, not hide

- **Every machine and agent must run the new binary** once the farm is migrated and synced: older yaks refuses a schema-7 farm. Land the code, `cargo install --path .`, then migrate, then `yaks sync` (Joel's call).
- **Haks** (Clint's, subtextdev/haks) tracks yaks `main` daily and its hub skill writes `wiki/digest/<date>.md` with `yaks page new`. That breaks on this change. Tell Clint before it lands, or hold the push until he can follow.
- Other threads shaving yaks that touch `wiki_tab.rs`, `pages.rs` or the docs (for example the starred TUI batch) will conflict. This is a mechanical rename: land it when those are quiet, or rebase them after.

## Evidence

- `cargo test --workspace` green, including the migration tests (a schema-6 fixture with wiki roots in the farm and in a herd, links in yaks and doks, a pinned link) and `cargo test -p yaks skills`.
- `grep -rniw wiki` over `src tests docs .agents README.md AGENTS.md` finds only Hearth-import code and its tests, and historical yak notes. `grep -rniw -E 'pages?'` leaves only "web page" (`source --url`), "login-page" style examples and prose that means a web page.
- A toque frame of the Doks tab and the `?` overlay, attached; the docshots regenerated and one rasterized (recipe in `docs/tui.md`).
- The migrated copy of this farm: `yaks doctor --strict`, `yaks check`, and `yaks peek` of a pinned link before and after.
- Update docs and in-app help in the same change (AGENTS.md rule), and `yaks scan-ids` clean on anything user-facing.

---
▸ 2026-11-04T17:23:25Z [source-fix]

Ordering (source-fix, with yaks-dedc): land yaks-dedc's tool and skill yaks before this one. They touch only `src/sources.rs`, the `source` help, `docs/cli.md`'s source section and the `yaks-research` skill, and this rename will sweep whatever they write (new help and skill prose say "pages"/"wiki" where they must). Doing it the other way round would make the dedc diff conflict with a ~78-file mechanical change.
