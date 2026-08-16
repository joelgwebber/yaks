#!/usr/bin/env python3
"""Measure index (re)serialization cost — answers 'does every mutation need a
full rewrite, and how expensive is a full rewrite?'. Synthetic records only,
no corpus needed. Run from repo root: python3 bench/idx_write.py
"""

from __future__ import annotations

import json
import os
import tempfile
import time
from pathlib import Path

SIZES = [50000, 100000]


def timeit(fn, repeat=5):
    best = float("inf")
    val = None
    for _ in range(repeat):
        t0 = time.perf_counter()
        val = fn()
        best = min(best, time.perf_counter() - t0)
    return best, val


def make_records(n: int) -> dict:
    recs = {}
    for i in range(n):
        stem = f"yak-{i:05x}"
        recs[stem] = [
            "shorn", 1700000000000000000 + i, 480 + (i % 200), "task",
            (i % 5) + 1, "2026-08-15T10:00:00Z",
            f"some representative task title number {i}",
            ["ui", "perf"] if i % 3 else [],
        ]
    return recs


def atomic_write(path: Path, data: str) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(data)
    os.replace(tmp, path)  # atomic rename over target


def tsv_dumps(recs: dict) -> str:
    out = []
    for stem, (s, m, sz, ty, pri, upd, title, labels) in recs.items():
        out.append(f"{stem}\t{s}\t{m}\t{sz}\t{ty}\t{pri}\t{upd}\t"
                   f"{','.join(labels)}\t{title}")
    return "\n".join(out) + "\n"


def tsv_loads(text: str) -> dict:
    out = {}
    for line in text.splitlines():
        p = line.split("\t")
        out[p[0]] = p
    return out


def run(n: int) -> None:
    recs = make_records(n)
    tmp = Path(tempfile.mkdtemp(prefix="idxw-"))
    try:
        jpath = tmp / "idx.json"
        tpath = tmp / "idx.tsv"

        t_jdump, jtext = timeit(lambda: json.dumps(recs))
        t_jwrite, _ = timeit(lambda: atomic_write(jpath, jtext))
        t_jfull, _ = timeit(lambda: atomic_write(jpath, json.dumps(recs)))
        t_jload, _ = timeit(lambda: json.loads(jpath.read_text()))

        t_tdump, ttext = timeit(lambda: tsv_dumps(recs))
        t_tfull, _ = timeit(lambda: atomic_write(tpath, tsv_dumps(recs)))
        t_tload, _ = timeit(lambda: tsv_loads(tpath.read_text()))

        print(f"\nn = {n}   json {jpath.stat().st_size/1e6:.1f}MB   "
              f"tsv {tpath.stat().st_size/1e6:.1f}MB")
        print(f"  json dumps           {t_jdump*1e3:7.1f} ms")
        print(f"  json write(atomic)   {t_jwrite*1e3:7.1f} ms")
        print(f"  json FULL rewrite    {t_jfull*1e3:7.1f} ms  (dumps+atomic write)")
        print(f"  json load            {t_jload*1e3:7.1f} ms")
        print(f"  tsv  dumps           {t_tdump*1e3:7.1f} ms")
        print(f"  tsv  FULL rewrite    {t_tfull*1e3:7.1f} ms")
        print(f"  tsv  load            {t_tload*1e3:7.1f} ms")
    finally:
        import shutil
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    import platform
    print(f"platform: {platform.platform()}  python: {platform.python_version()}")
    for n in SIZES:
        run(n)
