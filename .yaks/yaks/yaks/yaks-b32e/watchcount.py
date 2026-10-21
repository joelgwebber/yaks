#!/usr/bin/env python3
"""Recursively inotify-watch a tree the way a file watcher does, and report
how many watches it actually costs + how long establishing them takes."""
import os, sys, time, ctypes, ctypes.util

libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)
fd = libc.inotify_init()
if fd < 0: sys.exit("inotify_init failed")
MASK = 0x00000002|0x00000100|0x00000200|0x00000040|0x00000080  # MODIFY|CREATE|DELETE|MOVED_FROM|MOVED_TO

root = sys.argv[1]
t0 = time.perf_counter()
n = fail = 0
for d, subdirs, files in os.walk(root):
    wd = libc.inotify_add_watch(fd, d.encode(), MASK)
    if wd < 0: fail += 1
    else: n += 1
el = (time.perf_counter()-t0)*1000
print("  %-46s watches=%-6d failed=%-3d setup=%.0fms" % (root, n, fail, el))
os.close(fd)
