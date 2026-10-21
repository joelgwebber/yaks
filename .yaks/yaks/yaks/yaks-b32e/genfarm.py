#!/usr/bin/env python3
"""Generate a synthetic yaks farm whose files mirror the shape of a real one.

Usage: genfarm.py <real-farm-dir> <out-root> <n> [--nested]

Writes <out-root>/.yaks/ (flat) or a 2-level hex-nested variant with --nested.
Bodies are sampled from the real farm so grep/search hit realistic text.
"""
import os, sys, random, re, shutil

real, out_root, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
nested = "--nested" in sys.argv
random.seed(20261018 + n)

# ---------- harvest the real farm ----------
real_mds = sorted(f for f in os.listdir(real) if f.endswith(".md"))
bodies = []          # (frontmatter-dict-ish lines we discard, body text)
for f in real_mds:
    txt = open(os.path.join(real, f), encoding="utf-8", errors="replace").read()
    if txt.startswith("---\n"):
        end = txt.find("\n---\n", 4)
        body = txt[end + 5:] if end != -1 else ""
    else:
        body = txt
    bodies.append(body)

# real attachment files: (size, is_text) sampled for replay
att_files = []
for d in os.listdir(real):
    p = os.path.join(real, d)
    if os.path.isdir(p):
        fs = [x for x in os.listdir(p) if os.path.isfile(os.path.join(p, x))]
        if fs:
            att_files.append([(x, os.path.getsize(os.path.join(p, x))) for x in fs])

# observed distributions (from the real farm, 480 yaks)
STATUS = ["shorn"] * 79 + ["hairy"] * 14 + ["dead"] * 5 + ["shaving"] * 2
TYPE   = ["task"] * 44 + ["feature"] * 33 + ["bug"] * 16 + ["idea"] * 5 + ["chore"] * 2
PRIO   = [1] * 6 + [2] * 36 + [3] * 53 + [4] * 4 + [5] * 1
LABELS = (["cli"] * 119 + ["ui"] * 98 + ["tui"] * 80 + ["delta"] * 55 + ["rust"] * 53 +
          ["skills"] * 49 + ["agent"] * 25 + ["git"] * 23 + ["eval"] * 23 +
          ["markdown"] * 22 + ["docs"] * 20 + ["parity"] * 15 + ["sheds"] * 13 +
          ["phase2"] * 12 + ["edtui"] * 12 + ["release"] * 11 + ["upstream"] * 10 +
          ["meta"] * 10 + ["herds"] * 7 + ["design"] * 7)
TITLES = []
for f in real_mds:
    txt = open(os.path.join(real, f), encoding="utf-8", errors="replace").read()
    m = re.search(r"^title: (.*)$", txt, re.M)
    if m:
        TITLES.append(m.group(1).strip().strip("'\""))

# ---------- unique ids: 4-hex while the pool is sparse, then 5, then 6 ----------
ids, seen = [], set()
def mint(width):
    while True:
        s = "yaks-%0*x" % (width, random.getrandbits(width * 4))
        if s not in seen:
            seen.add(s); return s
for i in range(n):
    # keep 4-hex occupancy under ~25% (16384), then 5-hex under ~25% (262144)
    w = 4 if i < 16384 else (5 if i < 16384 + 262144 else 6)
    ids.append(mint(w))

root = os.path.join(out_root, ".yaks")
if os.path.exists(out_root):
    shutil.rmtree(out_root)
os.makedirs(root)

def yak_dir(yid):
    if not nested:
        return root
    h = yid.split("-")[1]
    d = os.path.join(root, h[0], h[1])
    return d

MONTHS = ["2026-07", "2026-08", "2026-09", "2026-10"]
def ts(i):
    mo = MONTHS[i % len(MONTHS)]
    return "%s-%02dT%02d:%02d:%02dZ" % (mo, 1 + (i * 7) % 28, i % 24, (i * 13) % 60, (i * 29) % 60)

made_dirs = set()
n_att = 0; n_attfiles = 0; total_md = 0
for i, yid in enumerate(ids):
    d = yak_dir(yid)
    if d not in made_dirs:
        os.makedirs(d, exist_ok=True); made_dirs.add(d)
    body = bodies[random.randrange(len(bodies))]
    # a quarter of yaks get a second body appended (long-tail sizes)
    if random.random() < 0.25:
        body += "\n\n---\n\n" + bodies[random.randrange(len(bodies))]
    # remap yak-id mentions in the body to ids that exist in THIS farm
    body = re.sub(r"yaks-[0-9a-f]{4,6}", lambda m: ids[random.randrange(n)], body)
    st = random.choice(STATUS)
    fm = ["---", "id: %s" % yid,
          "title: '%s'" % TITLES[random.randrange(len(TITLES))].replace("'", "''"),
          "status: %s" % st,
          "type: %s" % random.choice(TYPE),
          "priority: %d" % random.choice(PRIO),
          "created: '%s'" % ts(i),
          "updated: '%s'" % ts(i + 1)]
    if random.random() < 0.64 and i > 0:
        fm.append("parent: %s" % ids[random.randrange(i)])
    if random.random() < 0.70:
        k = random.choice([1, 1, 1, 2, 2, 3])
        ls = sorted(set(random.choice(LABELS) for _ in range(k)))
        fm.append("labels:")
        fm += ["- %s" % x for x in ls]
    fm.append("---")
    att_links = ""
    if random.random() < 0.16 and att_files:          # 16% have attachments (real: 78/480)
        group = random.choice(att_files)
        ad = os.path.join(d, yid) if not nested else os.path.join(d, yid)
        os.makedirs(ad, exist_ok=True)
        n_att += 1
        links = []
        for (name, size) in group:
            stem, ext = os.path.splitext(name)
            fname = "%s-%s%s" % (yid, stem[:20], ext)
            with open(os.path.join(ad, fname), "wb") as fh:
                # realistic-ish filler at the real size
                blob = ("evidence for %s: headless frame capture line\n" % yid).encode()
                fh.write((blob * (size // len(blob) + 1))[:size])
            links.append("[%s](%s/%s)" % (stem[:20], yid, fname))
            n_attfiles += 1
        att_links = "\n\n" + "\n\n".join(links) + "\n"
    out = "\n".join(fm) + "\n\n" + body + att_links
    p = os.path.join(d, yid + ".md")
    with open(p, "w", encoding="utf-8") as fh:
        fh.write(out)
    total_md += len(out.encode())

shutil.copy(os.path.join(real, "config.yaml"), os.path.join(root, "config.yaml"))
shutil.copy(os.path.join(real, "schema"), os.path.join(root, "schema"))
# the synthetic farm is standalone: drop the live_branch redirect
cfg = open(os.path.join(root, "config.yaml")).read().replace("live_branch: farm", "# live_branch removed for the synthetic farm")
open(os.path.join(root, "config.yaml"), "w").write(cfg)

print("farm=%s n=%d nested=%s md_bytes=%d att_dirs=%d att_files=%d" %
      (out_root, n, nested, total_md, n_att, n_attfiles))
with open(os.path.join(out_root, "ids.txt"), "w") as fh:
    fh.write("\n".join(ids) + "\n")
