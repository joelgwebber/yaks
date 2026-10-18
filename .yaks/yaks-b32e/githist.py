"""Add N commits, each touching 1-3 random yak files (the real farm's shape:
~627 commits over ~480 yaks). Reports the per-commit add+commit cost."""
import subprocess, sys, time, os, random, statistics
farm, ncommits = sys.argv[1], int(sys.argv[2])
root = os.path.join(farm, ".yaks")
ids = [l.strip() for l in open(os.path.join(farm, "ids.txt"))]
random.seed(7)
def sh(cmd):
    t0 = time.perf_counter()
    p = subprocess.run(["/bin/sh","-c",cmd], cwd=farm, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return (time.perf_counter()-t0)*1000, p.stdout.decode()
adds, commits = [], []
for i in range(ncommits):
    touched = random.sample(ids, random.choice([1,1,2,3]))
    for yid in touched:
        p = os.path.join(root, yid + ".md")
        with open(p, "a") as fh:
            fh.write("\n---\n\n\u25b8 2026-10-%02dT12:00:00Z [scale-1]\n\nSynthetic note %d: progress on the thing.\n" % (1+i%28, i))
    a,_ = sh("git add -A .yaks"); adds.append(a)
    c,_ = sh("GIT_EDITOR=true git commit -q -m 'yaks: updated %s'" % " ".join(touched)); commits.append(c)
print("%-6s n=%d  add  med=%.0fms p90=%.0fms   commit med=%.0fms p90=%.0fms" % (
    os.path.basename(farm), ncommits,
    statistics.median(adds), sorted(adds)[int(len(adds)*0.9)],
    statistics.median(commits), sorted(commits)[int(len(commits)*0.9)]))
