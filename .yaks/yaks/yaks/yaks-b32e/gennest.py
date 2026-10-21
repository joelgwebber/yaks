#!/usr/bin/env python3
"""Build Joel's proposed layout from an existing flat farm:

    .yaks/<herd>/3/a/8/           <- bucket dir, last hex digit TRUNCATED
                   yaks-3a81.md   <- yak file
                   yaks-3a81-frame.txt  <- its attachments, COLOCATED (no per-yak dir)

So each bucket holds ~16 yaks and all their attachments. Attachment name
conflicts are avoided by prefixing the yak id (which yaks already does).

Usage: gennest.py <flat-farm-root> <out-root>
"""
import os, sys, shutil, re

src, dst = sys.argv[1], sys.argv[2]
sroot = os.path.join(src, ".yaks")
droot = os.path.join(dst, ".yaks")
if os.path.exists(dst): shutil.rmtree(dst)
os.makedirs(droot)

n_yaks = n_att = 0
for name in os.listdir(sroot):
    p = os.path.join(sroot, name)
    if not name.endswith(".md"):
        continue
    yid = name[:-3]                       # yaks-3a81
    herd, hexpart = yid.split("-", 1)
    # 3 levels from the hex, LAST digit truncated (it names the file, not a dir)
    lv = hexpart[:3]
    bucket = os.path.join(droot, herd, *lv)
    os.makedirs(bucket, exist_ok=True)
    txt = open(p, encoding="utf-8").read()
    # attachments move beside the yak file; rewrite links <id>/<file> -> <file>
    adir = os.path.join(sroot, yid)
    if os.path.isdir(adir):
        for f in os.listdir(adir):
            shutil.copy2(os.path.join(adir, f), os.path.join(bucket, f))
            n_att += 1
        txt = txt.replace("(%s/" % yid, "(")
    open(os.path.join(bucket, name), "w", encoding="utf-8").write(txt)
    n_yaks += 1

for f in ("config.yaml", "schema"):
    shutil.copy2(os.path.join(sroot, f), os.path.join(droot, f))
shutil.copy2(os.path.join(src, "ids.txt"), os.path.join(dst, "ids.txt"))

dirs = sum(1 for _ in os.walk(droot))
print("nested: yaks=%d attachments=%d dirs=%d" % (n_yaks, n_att, dirs))
