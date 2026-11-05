#!/usr/bin/env bash
# Every example must show a clean `yass status --strict` once dropped into a repo with YASS installed.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
W="${1:-$(mktemp -d)}"; fail=0
if [ -z "${YASS_BIN:-}" ]; then
  mkdir -p "$W/bin"; (cd "$ROOT" && go build -o "$W/bin/yass" ./cmd/yass) || exit 1; YASS_BIN="$W/bin/yass"
fi
export YASS_BIN; export PATH="$(dirname "$YASS_BIN"):$PATH"
for ex in "$ROOT"/examples/*/; do
  name="$(basename "$ex")"; d="$W/example-$name"; rm -rf "$d"; mkdir -p "$d"; cd "$d"
  git init -q -b main && git config user.email t@t && git config user.name Tester
  yass init >/dev/null
  cp -R "$ex"/. .
  git add -A >/dev/null && git commit -q -m "chore: example"
  if out="$(yass status --strict 2>&1)"; then echo "ok   $name: status is clean"; else echo "FAIL $name"; echo "$out"; fail=1; fi
  yass status --archived >/dev/null 2>&1 && echo "ok   $name: archive lists" || { echo "FAIL $name: archive"; fail=1; }
  if out="$(yass hook --range "$(git rev-list --max-parents=0 HEAD)..HEAD" --strict 2>&1)"; then echo "ok   $name: hook clean"; else echo "FAIL $name: hook"; echo "$out"; fail=1; fi
done
exit $fail
