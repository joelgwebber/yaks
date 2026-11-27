# Index storage prototype: SQLite FTS5 vs tantivy (yaks-2704)

Throwaway prototypes for the storage decision in section 9 of `/doks/design/one-document-model.md`. Source: the attached `idxproto-src/` (a scratch cargo workspace, nothing in the yaks repo). Raw numbers: `results-run1.txt`.

**Machine:** Apple M4 Max (14 cores), macOS 26.7.2, cargo 1.99.0, `--release` with the yaks profile (`opt-level 3`, thin LTO, strip). **Versions:** rusqlite 0.43 `bundled` (libsqlite3-sys 0.40, SQLite 3.45.3, FTS5 enabled by the crate's build script), tantivy 0.27.

## Bottom line

- Both stores do the whole job at this scale. Every query measured is under 3.5 ms median on a 10k-file farm, and a full build is 0.2-2.5 s of indexing.
- **Tantivy does not need a second store** for the four needs (positional postings, link edges, parsed fields, mtimes). By the letter of the 2026-11-27 rule it is eligible. It is clumsier at edges and mtimes, and it costs more binary, crates and compile time.
- **SQLite FTS5 is the smaller, simpler single store** (+1.8 MB binary, 7 new crates) and makes edges and mtimes ordinary tables. Its costs are a 5x larger index, a C compiler in the build of all five targets, and an `unsafe` FFI tokenizer for identifiers.
- The dominant cold-build cost is not the index: it is the `git log --name-only` pass for modified times (0.5 s on this farm, about 9 s on a 28k-commit synthetic farm). It is the same for both stores.

## What was built

Both prototypes share one harness (`common/`): scan `*.md`, parse frontmatter (`title`, `tags`) and `](target#anchor)` links, one `git log --name-only` pass for mtimes (commits with a `Mechanical:` trailer skipped, dirty files fall back to fs mtime), then index.

| Need | SQLite FTS5 | tantivy |
|---|---|---|
| positional text | contentless FTS5 table over `kind, title, tags, body`; `tokenize='unicode61 remove_diacritics 2'`; `prefix='2 3 4'` | schema with `title`, `tags`, `body` text fields, positions on, tokenizer = simple + lowercase + ascii folding |
| link edges | `edges(src, target, anchor)` + btree indexes on `target` and `src` | one multi-valued raw `links` field (`target`, and `target#anchor`), stored; backlinks = one `TermQuery` |
| parsed field values | `docs` columns, or FTS5 column filters (`tags:tui`) | typed fields, fast fields for sorting |
| mtimes | `docs.mtime`, btree-indexed | `mtime` i64 fast field (sort and list) |
| second store needed? | no | **no**, but see "Where tantivy is clumsier" |

## Measurements

Real farm: this farm, 1142 `.md` files, 5.1 MB of text, 3462 link edges, 3195 commits. Synthetic farm: 10 000 yak files (frontmatter, 1-3 KB bodies sampled from the real farm's text so word frequencies are natural, 2-7 links each), 24 MB of text, 66 868 edges, 28 380 commits including one mechanical commit that touches every file.

"Cold" means a freshly copied directory (`cp -R`) on a warm OS file cache: `sudo purge` is not available, so a true cold-disk read is not measured. On an SSD, reading 5-24 MB is small next to the rest of the work, and the first build after a copy also pays for `git status` refreshing the copied index.

### Build time (ms)

| | real: scan+parse | real: git log+status | real: **index** | synth: scan+parse | synth: git log+status | synth: **index** |
|---|---|---|---|---|---|---|
| SQLite, first build (fresh copy) | 186 | 1632 | **228** | 283 | 9508 | **1842** |
| SQLite, warm rebuild | | | 284 | | | 1747 |
| tantivy default writer | 277 | 1942 | **661** | 265 | 9292 | **769** |
| tantivy 1 writer thread | | | 189 | | | 499 |
| tantivy 2 threads | | | 188 | | | 373 |

Notes:

- The git phase varies a lot run to run (0.5-1.9 s on the real farm; the high end follows a fresh copy's index refresh). `git log --name-only --no-renames` over 28k commits is 8.4-9.5 s regardless of flags (`--invert-grep`, `--no-merges`, `--diff-merges=off` and a commit-graph all made no difference); a commit-graph with `--changed-paths` helps only per-path (`git log -1 -- file`: 0.25 s to 0.08 s).
- tantivy's default writer spawns a thread per core and then merges segments, which costs more on a small corpus (661 ms) than one thread (189 ms). Segment count after the build is 1 in every run; `garbage_collect_files` is needed to get the 2.2 MB figure (without it the directory held 6.8 MB of replaced segments).
- SQLite FTS5 `optimize` takes 54-350 ms of the build; skipping it leaves more segments (slower queries).

### Incremental update

| | SQLite | tantivy |
|---|---|---|
| upsert 1 file (reopen, parse, delete+insert, commit) | 0.5-5 ms (first call after open 5 ms) | 85-115 ms (one writer thread, includes waiting for merge threads on drop) |
| upsert 50 files, one commit | 6-24 ms | 78-88 ms |
| upsert 1000 files, one commit | 0.3-2.0 s (noisy; WAL checkpoint, FTS5 delete + prefix indexes) | 125-157 ms |
| list every `(path, mtime)` (staleness input), 10k rows | 1 ms | 18-55 ms (fast-field walk) |
| per-file `git log -1 -- path` | 30-60 ms real farm, 200-350 ms on the 28k-commit farm | same |

The per-file `git log` dominates a one-file update on both. An incremental refresh should ask git once for `<last indexed commit>..HEAD --name-only` rather than once per file.

### Size on disk

| | real farm | synthetic 10k |
|---|---|---|
| SQLite (prefix indexes, contentless) | 12.4 MB (1 file) | 62.3 MB |
| SQLite, no prefix indexes | | 28.3 MB |
| SQLite, stored (not contentless) | 18.2 MB | 98.7 MB |
| tantivy | 2.2 MB (10 files) | 10.6 MB (10 files) |

### Release binary and dependencies

Measured as `idx-*` bins with the yaks release profile; base = the harness without an index (510 KB), hello-world = 339 KB.

| | binary | added over harness | crates added vs the yaks `Cargo.lock` | clean compile of the new crates |
|---|---|---|---|---|
| SQLite (`rusqlite` bundled) | 2.34 MB | +1.83 MB | 7 | 18.6 s (compiling SQLite's C amalgamation) |
| tantivy default features | 5.85 MB | +5.34 MB | 54 | 1 m 55 s |
| tantivy `default-features = false, features = ["mmap","lz4-compression"]` | 4.94 MB | +4.43 MB | 49 | 47 s |

The yaks binary is 14.7 MB today. tantivy's defaults pull `zstd-sys` (a C library) and the `frostem` stemmer; the reduced feature set drops both and needs no C compiler. SQLite always needs one.

### Query latency (median over 200 runs, top-20 results ranked by BM25)

Synthetic farm (10 000 files); the real-farm figures are in `results-run1.txt` and are all under 0.6 ms for both.

| query | SQLite | tantivy |
|---|---|---|
| rare word `rkyv` | 0.04 ms | 0.02 ms |
| common word `task` | 0.83 ms | 0.03 ms |
| word + sort by mtime | 0.10 ms | 0.03 ms |
| prefix `ta*` (2 chars) | 3.0 ms | 0.53 ms |
| prefix `ind*` | 0.71 ms | 0.47 ms |
| prefix `docum*` | 0.80 ms | 0.49 ms |
| prefix `deriv*` | 0.27 ms | 0.48 ms |
| phrase "derived index" | 0.07 ms | 0.03 ms |
| phrase "the farm" (common words) | 3.0 ms | 0.73 ms |
| field `tags:tui` | 0.72 ms | 0.02 ms |
| field `tags:tui` + word | 0.18 ms | 0.02 ms |
| backlinks, most linked target (287 in) | 0.07 ms | 0.015 ms |
| backlinks, typical target (5 in) | 0.008 ms | 0.005 ms |

Both are fast enough to run per keystroke. Prefix: FTS5 with `prefix='2 3 4'` is a direct index lookup (fast when the prefix is rare, slower for 2-char prefixes with many hits); tantivy runs a regex over the term dictionary, about 0.5 ms regardless of the prefix. Without the FTS5 prefix indexes a 2-char prefix is 5.4 ms (vs 3.0) and 3-5 chars are 0.3-0.9 ms (about the same), and the index is 28 MB instead of 62 MB, so they buy little. The first query after opening is 0.1-3 ms slower on both (page cache).

### Identifier-aware pre-tokenizer

Wanted: `issues/123` → `issues`, `123`, `issues/123`; `yaks-ca59` → `yaks`, `ca59`, `yaks-ca59`.

Both prototypes do it and pass the same checks (`issues/123`, `yaks-ca59`, `ca59`, `issues`, `123`, `/yaks/yaks/yaks-ca59`, phrase `"issues 123"`, prefix `yaks-ca`, prefix `issues/12`).

- **tantivy**: a safe `Tokenizer`/`TokenStream` impl in ~110 lines (`tantivy/src/idtok.rs`, with a unit test). It emits parts plus the whole identifier at the position of the last part (`position_length` set), then `LowerCaser` and `AsciiFoldingFilter` stack on top. Plugging in is `index.tokenizers().register(name, analyzer)`. The same analyzer runs on the query side through tantivy's query parser (I wrote the queries by hand and ran the analyzer on the word myself).
- **SQLite**: a custom FTS5 tokenizer wrapping `unicode61` through the C API (`fts5_api`, `xCreateTokenizer`, `FTS5_TOKEN_COLOCATED`), ~170 lines of `unsafe` Rust (`sqlite/src/idtok.rs`). It needs `libsqlite3_sys` directly (rusqlite has no tokenizer API), a pointer-binding call to get `fts5_api`, and care over two modes: at index time it emits parts plus the whole identifier colocated; at query time (`FTS5_TOKENIZE_QUERY`) it emits only the whole identifier so `yaks-ca59` matches exactly. It must be registered on every connection, including read-only ones.
- Cost: SQLite index 62.3 to 68.2 MB, tantivy 10.6 to 12.7 MB. Without it, a `issues/510` query cannot match as one identifier: with the plain tokenizers SQLite finds 15 files (`issues` adjacent to `510`) and the tantivy prototype's word query degrades to `issues` alone (20 hits, capped at the top 20). With it both find the same 11 files, and `yaks-0001` becomes an exact-id lookup (8 hits, against 20 for the plain tantivy prototype). Counts are on the synthetic farm.

Easy in both; clearly easier and safer in tantivy.

### Release targets

Documented, not cross-compiled (the five targets are in `.github/workflows/release.yml`).

- **tantivy** states Linux, macOS and Windows support (README). With `default-features = false` it is pure Rust; with defaults it adds `zstd-sys` (C, built by `cc`). Nothing else here builds C.
- **rusqlite `bundled`** compiles SQLite with the `cc` crate on every target. rusqlite's README names `bundled` as the option for platforms where linking is complicated, "such as Windows".
  - `aarch64-unknown-linux-gnu`: `cc` looks for `aarch64-linux-gnu-gcc` (its target table has that entry); the workflow already installs `gcc-aarch64-linux-gnu` and sets the linker.
  - Both `*-apple-darwin` targets build on `macos-15` with clang, no extra setup.
  - `x86_64-pc-windows-msvc` needs `cl.exe`, which `windows-latest` provides.
  - So no workflow change is needed, but the comment in `release.yml` ("all deps are pure Rust ... no target C libraries") becomes false and `cargo build` on a machine without a C compiler stops working.
- I did not run any of the five release builds; the claims above are from the crates' docs and source plus the workflow file.

## Where tantivy is clumsier (all needs are met, none requires a second store)

- **Edges as terms**: backlinks work well (one term query). "Outgoing links of file X" (needed for rewrites) means reading a stored multi-valued field back; "all edges as rows" means a doc walk. In SQLite it is one indexed `SELECT`.
- **Metadata updates**: tantivy has no in-place update; changing an mtime or tag re-indexes the document (delete + add, plus a commit of 80-110 ms and segment churn). In SQLite it is a row `UPDATE`. In practice an mtime changes only when the file does, so this matters mostly for bulk metadata passes.
- **Listing all `(path, mtime)`** needs a fast-field walk (18-55 ms for 10k) rather than a table scan (1 ms). Fine at 10k, so this is a shape point, not a performance one.
- **No joins**: a query like "files matching `tui` that link to X" is two queries and an intersection in the caller; SQLite does it in one statement.
- **Many files**: the index is a directory of 10-60 files with a `meta.json`, a writer lock file (`LockBusy` when a second writer opens), and segment garbage to collect. SQLite is one file (plus WAL) with a busy timeout.

## Where SQLite is clumsier

- **5x larger index** (62 vs 10.6 MB at 10k files) and **slower bulk updates** (1000 files: 0.3-2 s vs 0.13 s; the prefix indexes and FTS5 deletes account for most of it).
- **The tokenizer is `unsafe` FFI** against `libsqlite3-sys` raw bindings, not a safe API.
- **A C toolchain** in every build (see Release targets).
- FTS5 phrase and prefix syntax needs query escaping by the caller (`"word"*`), which tantivy's typed queries avoid.

## Caveats

- One machine, warm file cache, no true cold-disk run.
- The synthetic farm samples real text, so word statistics are realistic, but its link graph is random (the "most linked" target has 287 inbound; the real farm's has 65) and its 28k-commit history is shaped by me.
- Bulk-update timings for SQLite are noisy (0.3-2.8 s for the same 1000-file batch across runs); the lower end is nearer the steady state.
- I did not measure memory use or concurrent readers during a rebuild.
- My queries are hand-built; neither a query parser nor ranking tuning is evaluated. Both use BM25 and agree on hit counts for the checks I ran (e.g. 7 hits for "derived index" on the real farm in both).
