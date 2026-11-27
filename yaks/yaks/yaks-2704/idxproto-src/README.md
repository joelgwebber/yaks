# yaks-2704 index storage prototypes (throwaway)

Scratch cargo workspace comparing SQLite FTS5 (`sqlite/`, rusqlite `bundled`) with tantivy (`tantivy/`,
default features; `tv-lite/` is the same code with `default-features = false`). Not for the yaks repo.

- `common/`: file scan + frontmatter/link parse, one-pass `git log --name-only` mtimes (skips commits with a
  `Mechanical:` trailer), bench harness (`build`, `update`, `batch`, `bench`, `show`), and `gen`, which writes a
  synthetic farm of N yak files with a git history (text sampled from a real farm).
- `sqlite/src/idtok.rs`, `tantivy/src/idtok.rs`: identifier-aware tokenizers (`IDTOK=1`).
- `base/`, `hello/`: binary-size baselines. `run.sh`: reproduces `results-run1.txt` (expects `data/real-src` and `data/synth`).

    cargo build --release -p common -p idx-sqlite -p idx-tantivy -p idx-base -p hello
    CARGO_TARGET_DIR=t-tv cargo build --release -p idx-tv-lite
    target/release/gen <real-farm-clone> data/synth 10000
    ./run.sh

Env knobs: `IDTOK=1` (identifier tokenizer), `STORED=1` / `NOPREFIX=1` (sqlite), `TV_THREADS=n` (tantivy writer threads).
