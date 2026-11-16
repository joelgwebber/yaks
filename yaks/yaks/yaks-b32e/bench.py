#!/usr/bin/env python3
"""Run a shell command R times, report min/median in ms. Usage: bench.py <label> <reps> <cwd> <shell-cmd>"""
import subprocess, sys, time, statistics, os
label, reps, cwd, cmd = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
env = dict(os.environ)
ts = []
out = b""
rc = 0
for i in range(reps):
    t0 = time.perf_counter()
    p = subprocess.run(["/bin/sh", "-c", cmd], cwd=cwd, env=env,
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    ts.append((time.perf_counter() - t0) * 1000)
    out, rc = p.stdout, p.returncode
print("%-34s reps=%d  min=%8.1fms  med=%8.1fms  max=%8.1fms  rc=%d  outbytes=%d" %
      (label, reps, min(ts), statistics.median(ts), max(ts), rc, len(out)))
