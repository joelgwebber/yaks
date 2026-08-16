#!/usr/bin/env python3
"""Throwaway fs/perf benchmark for the yak storage model (yak-3fd4).

Generates synthetic corpora and measures the operations that dominate CLI and
TUI cost, so storage/index/cache decisions rest on numbers rather than
intuition. Run from the repo root:  python3 bench/fs_bench.py
"""

from __future__ import annotations

import json
import os
import platform
import random
import shutil
import string
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

import yaml  # noqa: E402

from yaklib import model  # noqa: E402

try:
    from yaml import CSafeLoader  # libyaml-backed, if present

    HAVE_C = True
except ImportError:
    HAVE_C = False

SIZES = [1000, 10000, 50000]
# Realistic mix: completed work dominates; active work is bounded.
MIX = {"hairy": 0.05, "shaving": 0.02, "shorn": 0.93}
CHILD_FRACTION = 0.10  # of tasks, made children of an earlier task
LOOKUP_SAMPLES = 300

WORDS = ("refactor login token cache retry parser tui filter view render "
         "index scan dep tree shard bench detail search label sort pin "
         "config migrate skill rollup status frontmatter timestamp").split()


def rand_title() -> str:
    return " ".join(random.choice(WORDS) for _ in range(random.randint(4, 9)))


def rand_body() -> str:
    n = random.randint(2, 8)
    return "\n".join(" ".join(random.choice(WORDS) for _ in range(random.randint(6, 14)))
                     for _ in range(n))


def task_text(tid: str) -> str:
    labels = random.sample(("ui", "perf", "agent", "search", "demo", "bug"),
                           k=random.randint(0, 3))
    fm = {
        "id": tid,
        "title": rand_title(),
        "type": random.choice(("task", "bug", "feature", "idea")),
        "priority": random.randint(1, 5),
        "created": "2026-08-15T10:00:00Z",
        "updated": f"2026-08-{random.randint(1, 28):02d}T{random.randint(0,23):02d}:00:00Z",
    }
    if labels:
        fm["labels"] = labels
    out = ["---\n", model.dump_yaml(fm), "---\n\n", rand_body(), "\n"]
    return "".join(out)


def gen_corpus(root: Path, n: int) -> list[str]:
    ids: list[str] = []
    for s in ("hairy", "shaving", "shorn", "dead"):
        (root / s).mkdir(parents=True, exist_ok=True)
    counts = {s: int(n * f) for s, f in MIX.items()}
    counts["shorn"] += n - sum(counts.values())
    existing: set[str] = set()

    def new_id() -> str:
        while True:
            tid = "yak-" + "".join(random.choices(string.hexdigits[:16], k=4))
            if tid not in existing:
                existing.add(tid)
                return tid

    for status, c in counts.items():
        for _ in range(c):
            if ids and random.random() < CHILD_FRACTION:
                parent = random.choice(ids)
                # avoid runaway nesting; only child of a top-level id
                if "." not in parent:
                    k = 1
                    tid = f"{parent}.{k}"
                    while tid in existing:
                        k += 1
                        tid = f"{parent}.{k}"
                    existing.add(tid)
                else:
                    tid = new_id()
            else:
                tid = new_id()
            (root / status / f"{tid}.md").write_text(task_text(tid))
            ids.append(tid)
    return ids


# --------------------------------------------------------------------------
# Measurement primitives
# --------------------------------------------------------------------------

def timeit(fn, repeat=1):
    best = float("inf")
    for _ in range(repeat):
        t0 = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - t0)
    return best


def naive_parse(text: str) -> dict:
    """Fast hand-rolled frontmatter parse (approximate; estimates a lower
    bound on parse cost vs PyYAML). Handles scalars + simple '- ' lists."""
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---", 4)
    if end < 0:
        return {}
    fm = text[4:end]
    d: dict = {}
    key = None
    for line in fm.splitlines():
        if line.startswith("- "):
            if not isinstance(d.get(key), list):
                d[key] = []
            d[key].append(line[2:].strip())
        elif ":" in line:
            k, _, v = line.partition(":")
            key = k.strip()
            v = v.strip()
            d[key] = v if v else None
    return d


def read_all(root: Path) -> list[Path]:
    files = []
    for s in ("hairy", "shaving", "shorn"):
        d = root / s
        if d.exists():
            files.extend(d.glob("*.md"))
    return files


def build_index(root: Path, path: Path) -> None:
    idx = {}
    for s in ("hairy", "shaving", "shorn", "dead"):
        d = root / s
        if not d.exists():
            continue
        for f in d.glob("*.md"):
            st = f.stat()
            t = model.load_task(f)
            idx[f.stem] = {
                "s": s, "m": st.st_mtime_ns, "sz": st.st_size,
                "title": t.get("title"), "type": t.get("type"),
                "priority": t.get("priority"), "updated": t.get("updated"),
                "labels": t.get("labels"),
            }
    path.write_text(json.dumps(idx))


def load_index_validated(root: Path, path: Path) -> tuple[int, int]:
    """Load cached index, then stat-validate every file (no parse). Returns
    (n_entries, n_stale) — stale = would need reparse."""
    idx = json.loads(path.read_text())
    stale = 0
    seen = set()
    for s in ("hairy", "shaving", "shorn", "dead"):
        d = root / s
        if not d.exists():
            continue
        with os.scandir(d) as it:
            for e in it:
                if not e.name.endswith(".md"):
                    continue
                stem = e.name[:-3]
                seen.add(stem)
                st = e.stat()
                rec = idx.get(stem)
                if rec is None or rec["m"] != st.st_mtime_ns or rec["sz"] != st.st_size:
                    stale += 1
    return len(idx), stale


# --------------------------------------------------------------------------

def run(n: int) -> dict:
    tmp = Path(tempfile.mkdtemp(prefix="yakbench-"))
    root = tmp / ".yaks"
    try:
        ids = gen_corpus(root, n)
        top_ids = [i for i in ids if "." not in i]
        parents_with_kids = list({i.rsplit(".", 1)[0] for i in ids if "." in i})
        sample_ids = random.sample(ids, min(LOOKUP_SAMPLES, len(ids)))
        res = {"n": n}

        # 1. Full scan + parse via the real code path (all_tasks).
        res["all_tasks"] = timeit(lambda: model.all_tasks(root))

        # 2. readdir + stat only (bulletproof validation cost, no parse).
        def validate():
            for s in ("hairy", "shaving", "shorn"):
                d = root / s
                if d.exists():
                    with os.scandir(d) as it:
                        for e in it:
                            e.stat()
        res["readdir_stat"] = timeit(validate, repeat=3)

        # 3. Parse-only comparison on the same file set (read cost included).
        files = read_all(root)

        def parse_pyyaml():
            for f in files:
                text = f.read_text()
                e = text.find("\n---", 4)
                yaml.safe_load(text[4:e])
        res["parse_pyyaml"] = timeit(parse_pyyaml)

        if HAVE_C:
            def parse_cyaml():
                for f in files:
                    text = f.read_text()
                    e = text.find("\n---", 4)
                    yaml.load(text[4:e], Loader=CSafeLoader)
            res["parse_cyaml"] = timeit(parse_cyaml)

        def parse_naive():
            for f in files:
                naive_parse(f.read_text())
        res["parse_naive"] = timeit(parse_naive)

        # 4. Single-id lookup (find_task_file) — exact-name stat probes.
        def find_ids():
            for tid in sample_ids:
                model.find_task_file(root, tid)
        res["find_task_file_us"] = timeit(find_ids) / len(sample_ids) * 1e6

        # 5. Children lookup (glob prefix) — O(entries) per show.
        if parents_with_kids:
            kids = random.sample(parents_with_kids, min(50, len(parents_with_kids)))

            def find_children():
                for p in kids:
                    model.find_children(root, p)
            res["find_children_ms"] = timeit(find_children) / len(kids) * 1e3

        # 6. Persistent index: build, then warm load + validate.
        idxpath = tmp / "index.json"
        res["index_build"] = timeit(lambda: build_index(root, idxpath))
        res["index_bytes"] = idxpath.stat().st_size
        res["index_load_validate"] = timeit(
            lambda: load_index_validated(root, idxpath), repeat=3)
        return res
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    print(f"platform: {platform.platform()}  python: {platform.python_version()}"
          f"  libyaml: {HAVE_C}")
    rows = [run(n) for n in SIZES]

    def col(key, fmt="{:.3f}"):
        return "  ".join(fmt.format(r[key]) if key in r else "   -  " for r in rows)

    hdr = "  ".join(f"{r['n']:>8d}" for r in rows)
    print(f"\n{'metric':28s}{hdr}")
    print("-" * (28 + len(hdr)))
    print(f"{'all_tasks (scan+parse) s':28s}{col('all_tasks')}")
    print(f"{'readdir+stat only s':28s}{col('readdir_stat')}")
    print(f"{'parse pyyaml s':28s}{col('parse_pyyaml')}")
    if HAVE_C:
        print(f"{'parse cyaml s':28s}{col('parse_cyaml')}")
    print(f"{'parse naive s':28s}{col('parse_naive')}")
    print(f"{'index build s':28s}{col('index_build')}")
    print(f"{'index load+validate s':28s}{col('index_load_validate')}")
    print(f"{'find_task_file (us/call)':28s}{col('find_task_file_us', '{:.2f}')}")
    print(f"{'find_children (ms/call)':28s}{col('find_children_ms', '{:.2f}')}")
    print(f"{'index size (bytes)':28s}{col('index_bytes', '{:d}')}")


if __name__ == "__main__":
    main()
