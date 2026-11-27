#!/bin/sh
# Reproduce the report's numbers.
#   cargo build --release -p common -p idx-sqlite -p idx-tantivy; CARGO_TARGET_DIR=t-tv cargo build --release -p idx-tv-lite
#   target/release/gen <real-farm-clone> data/synth 10000
#   ./run.sh                       # expects data/real-src and data/synth
set -e
cd "$(dirname "$0")"
W=${TMPDIR:-/tmp}/idxproto-run; rm -rf $W; mkdir -p $W
for F in real-src synth; do
  for B in sqlite tantivy tv-lite; do
    bin=target/release/idx-$B; [ $B = tv-lite ] && bin=t-tv/release/idx-tv-lite
    echo "######## $B on $F: build on a FRESH COPY (first build), then warm rebuild"
    rm -rf data/fresh-$F; cp -R data/$F data/fresh-$F 2>/dev/null || true
    $bin build data/fresh-$F $W/$B-$F 2>&1 | grep -v segments
    $bin build data/fresh-$F $W/$B-$F 2>&1 | grep -E 'TOTAL|index build'
    $bin bench data/fresh-$F $W/$B-$F 2>&1
    $bin batch data/fresh-$F $W/$B-$F 50 2>&1
    $bin batch data/fresh-$F $W/$B-$F 1000 2>&1 | grep upsert
  done
done
