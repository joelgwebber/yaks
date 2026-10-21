# yaks-b32e scale study — raw results (scout scale-1)

Machine: AMD Ryzen 9 5950X (32 threads), 125 GB RAM, Linux 7.2.8 cachyos,
git 2.56.0, yaks 0.0.20.
Farms built on **btrfs** (`/var/tmp`, same fs as `/home`), NOT tmpfs.
All numbers are medians of 3-7 reps, warm cache. See "Not measured" at the end.

Synthetic farms mirror the real farm (`/home/joel/src/yaks-farm/.yaks`, 480 yaks):
bodies sampled from real yaks, observed distributions for status / type /
priority / labels / parent (64% have a parent), 16% with an attachment dir,
mean 3.5 KB per yak file. Each farm is a git repo with 201 commits
(1-3 yak files per commit, matching the real farm's 627-commits-over-480-yaks).

## yaks binary (warm)

| command | 1k | 10k | 50k |
|---|---|---|---|
| `yaks list` | 51 ms | 202 ms | 770 ms |
| `yaks show <id>` | 54 ms | 118 ms | 1,245 ms |
| `yaks next` | 52 ms | 127 ms | 995 ms |
| `yaks search <word>` | 34 ms | 191 ms | 1,176 ms |
| `yaks stats` | 24 ms | 108 ms | 523 ms |
| **TUI first frame (headless 120x40)** | **24 ms** | **2,095 ms** | **17,500-57,000 ms** |

TUI scaling detail (git-free farms, so this is pure yaks):

| n | wall | user CPU | sys CPU | fitted exponent |
|---|---|---|---|---|
| 1,000 | 24 ms | - | - | |
| 2,000 | 56 ms | - | - | 1.21 |
| 5,000 | 214 ms | 0.17 s | 0.04 s | 1.46 |
| 20,000 | 2,599 ms | 2.45 s | 0.15 s | 1.80 |
| 50,000 | 17,546 ms | 17.14 s | 0.33 s | 2.08 |

17.1 s user vs 0.33 s sys => CPU-bound, not filesystem-bound.
Bisect at n=20k: full 2.63 s; no `parent:` 1.85 s; no body id-mentions 2.59 s;
neither 1.56 s. Parent/tree build is ~1/3; the rest is unlocated.

## Plain tools

| tool | 1k | 10k | 50k |
|---|---|---|---|
| `ls` | 2.8 ms | 18 ms | 90 ms |
| `ls -l` | 7.8 ms | 55 ms | 287 ms |
| `ls -U` (no sort) | 1.9 ms | 4.7 ms | 18 ms |
| `ls *.md \| wc -l` | 5.8 ms | 44 ms | 228 ms |
| `find . -type f` | 3.3 ms | 24 ms | 123 ms |
| `grep -rl <word>` | 12 ms | 120 ms | 854 ms |
| `rg -l <word>` | 6.6 ms | 21 ms | 83 ms |
| `cat *.md > /dev/null` | 8.6 ms | 66 ms | 332 ms |

## git

| op | 1k | 10k | 50k |
|---|---|---|---|
| `git status --porcelain` | 5.9 ms | 25 ms | 111 ms |
| `git status` (full) | 5.8 ms | 26 ms | 115 ms |
| `git add -A` (1 changed file, steady state) | 6 ms | 26 ms | 107 ms |
| `git commit` (steady state) | 10 ms | 23 ms | 77 ms |
| `git log --oneline -- .yaks \| wc -l` | 46 ms | 366 ms | **1,978 ms** |
| `git log --oneline` (no pathspec) | - | - | 6 ms |
| `git ls-tree -r HEAD -- .yaks` | 4.4 ms | 9.0 ms | 28 ms |
| `git ls-files .yaks` | 2.5 ms | 4.3 ms | 11 ms |
| `git diff --stat HEAD~5` | 5.4 ms | 9.2 ms | 30 ms |
| initial `git add -A` (one-time) | 213 ms | 2,507 ms | 8,860 ms |
| `.yaks` tree object | 45 KB | 459 KB | 2.3 MB |

`git log -- .yaks` is the only git number that looks bad, and it is
history-depth x tree-size (201 commits here), not directory width: the same
log without a pathspec is 6 ms.

## Shell pipeline hazards (50k)

| pipeline | time | mode |
|---|---|---|
| `ls \| grep \| wc -l` | 94 ms | fine |
| `for f in *.md; do :; done` | 92 ms | fine |
| `ls *.md \| xargs grep -l` | 545 ms | fine |
| `find -name '*.md' \| xargs grep -l` | 383 ms | fine |
| `find -name '*.md' -exec grep -l {} +` | 468 ms | fine |
| `find -name '*.md' -exec grep -l {} \;` | 64,843 ms | LOUD (50k forks; 723 ms @1k, 8.5 s @10k) |
| `wc -c /abs/.yaks/*.md` | — | LOUD: E2BIG at ~41,310 abs paths |
| `ls *.md \| xargs wc -l \| tail -1` | — | **SILENT WRONG ANSWER** |

Silent-chunking detail: reports 96,966 vs the true 2,325,384 (24x off).
xargs splits at **~10,070 bare names / ~3,110 absolute paths** (128 KiB buffer).

## Web IDE (openvscode-server 1.109.5 + headless Chromium 142)

| metric | 1k | 10k | 50k |
|---|---|---|---|
| explorer ready | 1.48 s | 1.51 s | 1.51 s |
| expand `.yaks/` | 20 ms | 7 ms | 641 ms |
| **DOM rows rendered** | **39** | **39** | **39** |
| explorer DOM nodes | 442 | 442 | 442 |
| document DOM nodes | 1,301 | 1,394 | 1,394 |
| virtual scroll height | 25,256 px | 255,904 px | 1,275,930 px |
| scroll median / max | 33.3 / 33.4 ms | 33.3 / 45.3 ms | 33.3 / 83.7 ms |
| renderer JS heap | 58.7 MB | 91.6 MB | 157.4 MB |

Virtualization holds absolutely: 39 rows / 442 nodes at every size.
1,275,930 px / 22 px per row = 57,994 = exactly the farm's entry count.

Joel's real VS Code with f50k open, 14 processes: 2,147.6 MB RSS / 1,121.0 MB PSS.
Largest: renderer 364 MB PSS, gpu 137, main 110, copilot-runtime 107 (farm-independent).

## inotify: flat vs Joel's colocated-nested layout

Real `inotify_add_watch` counts (bin/watchcount.py):

| | flat | nested (colocated) | |
|---|---|---|---|
| 1k | 144 | 1,160 | nested 8.1x worse |
| 10k | 1,628 | 4,097 | nested 2.5x worse |
| 50k | 7,993 | 4,370 | nested 1.8x better |

flat = 1 + 0.16n (one watch per yak-with-attachments, unbounded).
nested = saturates at 4,370 = 1 + 1 + 16 + 256 + 4,096 buckets.
**Crossover at n = 27,306.** `max_user_watches` = 524,288, so flat does not
exhaust inotify until ~3.3M yaks; VS Code's watcher held 8,003 watches for f50k.

Nested bucket occupancy at 50k: 4,096 leaves, min 4 / median 15 / p90 22 / max 41.
Random hex ids balance fine with no hashing.

Nested does not fix the pipelines: xargs chunking is WORSE (11 invocations vs 7,
longer paths); `ls *.md` in `.yaks/` matches nothing; `rg -l` 55 ms vs 78 ms flat;
`find|xargs grep` 356 ms vs 367 ms flat. `yaks stats` reports "Total: 0".

## Not measured, and why

- **Cold cache.** Dropping caches needs root (`echo 3 > drop_caches`), which the
  brief forbids. Every number above is WARM. Cold would penalize flat and nested
  roughly in proportion to files touched, which is identical for both, so it does
  not change the comparison — but absolute first-run times on a cold machine will
  be higher, especially the 50k `git add` and `yaks list`.
- **Spinning disk / network FS.** btrfs on NVMe only. A farm on NFS or a slow
  external disk could behave very differently (per-file latency dominates), and
  that is where a nested layout's smaller directory blocks might matter more.
- **Other filesystems.** No ext4/XFS/ZFS comparison. btrfs and ext4 both use
  hashed directory indexes, so large-directory lookup should be comparable;
  untested.
- **macOS / Windows.** Linux only. macOS FSEvents watches recursively per-tree
  (not per-directory), so the inotify finding does not transfer; Windows
  `ReadDirectoryChangesW` is also recursive. If anything, Linux is the worst
  case for the watch metric and it is still fine.
- **Where the TUI's remaining O(n^2) lives.** Bisected to "~1/3 parent/tree
  build, rest unlocated". Needs a profiler against the source, which is outside
  a scout's remit.
- **Real VS Code baseline on a small farm.** Offered; judged unnecessary once
  the headless A/B showed only ~2 MB per 1,000 yaks of renderer growth.
