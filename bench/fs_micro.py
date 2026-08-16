#!/usr/bin/env python3
"""Micro-benchmark decomposing the fs cost model (follow-up to fs_bench.py).

Answers: (1) is readdir+stat really ~0.1s at 50k, and where does the time go;
(2) the raw read-I/O floor (proxy for a native-code rebuild); (3) JSON vs a
line-oriented index vs a single packed file; (4) why `ls` feels slow.

All numbers are WARM-CACHE on local SSD. Run from repo root:
    python3 bench/fs_micro.py
"""

from __future__ import annotations

import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

_here = Path(__file__).resolve().parent
sys.path.insert(0, str(_here))
sys.path.insert(0, str(_here.parent / "scripts"))

import fs_bench  # reuse corpus generation
from yaklib import model  # noqa: E402

SIZES = [50000]


def timeit(fn, repeat=1):
    best = float("inf")
    for _ in range(repeat):
        t0 = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - t0)
    return best


def sh_time(args, cwd, shell=False):
    t0 = time.perf_counter()
    subprocess.run(args, cwd=cwd, shell=shell,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return time.perf_counter() - t0


def micro(n: int) -> dict:
    tmp = Path(tempfile.mkdtemp(prefix="yakmicro-"))
    root = tmp / ".yaks"
    try:
        fs_bench.gen_corpus(root, n)
        status_dirs = [root / s for s in ("hairy", "shaving", "shorn")]
        files = [f for d in status_dirs for f in d.glob("*.md")]
        res: dict = {"n": n, "files": len(files)}

        # --- enumeration decomposition ---
        def scandir_only():
            c = 0
            for d in status_dirs:
                with os.scandir(d) as it:
                    for e in it:
                        c += e.name.endswith(".md")
            return c

        def scandir_stat():
            for d in status_dirs:
                with os.scandir(d) as it:
                    for e in it:
                        if e.name.endswith(".md"):
                            e.stat()

        res["scandir_only"] = timeit(scandir_only, 5)
        res["scandir_stat"] = timeit(scandir_stat, 5)

        # --- raw read I/O floor (no parse) ---
        total = 0
        for f in files:
            total += len(f.read_bytes())
        res["total_MB"] = total / 1e6

        def read_bytes():
            for f in files:
                f.read_bytes()

        def read_text():
            for f in files:
                f.read_text()

        res["read_bytes"] = timeit(read_bytes)
        res["read_text"] = timeit(read_text)

        # --- build compact records once (one full parse) ---
        records = {}
        for s in ("hairy", "shaving", "shorn"):
            d = root / s
            for f in d.glob("*.md"):
                st = f.stat()
                t = model.load_task(f)
                records[f.stem] = [
                    s, st.st_mtime_ns, st.st_size, t.get("type"),
                    t.get("priority"), t.get("updated"), t.get("title"),
                    t.get("labels") or [],
                ]

        # --- JSON index ---
        jpath = tmp / "idx.json"
        jpath.write_text(json.dumps(records))
        res["json_MB"] = jpath.stat().st_size / 1e6
        res["json_load"] = timeit(lambda: json.loads(jpath.read_text()), 5)

        # --- line-oriented (TSV) index ---
        tpath = tmp / "idx.tsv"
        with tpath.open("w") as fh:
            for stem, (s, m, sz, ty, pri, upd, title, labels) in records.items():
                fh.write(f"{stem}\t{s}\t{m}\t{sz}\t{ty}\t{pri}\t{upd}\t"
                         f"{','.join(labels)}\t{title}\n")
        res["tsv_MB"] = tpath.stat().st_size / 1e6

        def load_tsv():
            out = {}
            with tpath.open() as fh:
                for line in fh:
                    p = line.rstrip("\n").split("\t")
                    out[p[0]] = p
            return out

        res["tsv_load"] = timeit(load_tsv, 5)

        # --- single packed file: read + split once ---
        ppath = tmp / "packed.dat"
        with ppath.open("w") as fh:
            for f in files:
                fh.write(f.read_text())
                fh.write("\n\x00\n")
        res["packed_MB"] = ppath.stat().st_size / 1e6

        def read_packed():
            return len(ppath.read_text().split("\n\x00\n"))

        res["packed_read_split"] = timeit(read_packed, 5)

        # --- native `ls` variants + `cat` (on the big shorn dir) ---
        shorn = str(root / "shorn")
        res["ls_1"] = sh_time(["ls", "-1"], shorn)
        res["ls_1f"] = sh_time(["ls", "-1f"], shorn)   # unsorted
        res["ls_la"] = sh_time(["ls", "-la"], shorn)   # sorted + stat
        res["cat_all"] = sh_time("find . -name '*.md' -exec cat {} +", shorn, shell=True)
        return res
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    print(f"platform: {platform.platform()}  python: {platform.python_version()}")
    print("(warm-cache, local SSD; times in seconds unless noted)\n")
    rows = [micro(n) for n in SIZES]

    def line(label, key, fmt="{:.3f}"):
        cells = "  ".join(fmt.format(r[key]) if key in r else "  -  " for r in rows)
        print(f"{label:26s}{cells}")

    print(f"{'metric':26s}" + "  ".join(f"{r['n']:>9d}" for r in rows))
    print("-" * 48)
    line("files on disk", "files", "{:d}")
    line("corpus size (MB)", "total_MB", "{:.1f}")
    line("scandir only (no stat)", "scandir_only")
    line("scandir + stat", "scandir_stat")
    line("read all bytes (no parse)", "read_bytes")
    line("read all text (decode)", "read_text")
    line("cat all (native, shorn)", "cat_all")
    print()
    line("json index size (MB)", "json_MB", "{:.1f}")
    line("json load", "json_load")
    line("tsv index size (MB)", "tsv_MB", "{:.1f}")
    line("tsv load", "tsv_load")
    line("packed read+split", "packed_read_split")
    print()
    line("ls -1 (sorted)", "ls_1")
    line("ls -1f (unsorted)", "ls_1f")
    line("ls -la (sorted+stat)", "ls_la")


if __name__ == "__main__":
    main()
