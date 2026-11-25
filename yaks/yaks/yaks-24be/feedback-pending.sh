#!/bin/sh
# Print the feedback-to-yaks entries no ledger row covers (or updated since).
ledger=$(yaks dok query --rows --type topic --json | jq -c '[.[] | select(.dok=="doks/feedback-triage.md") | {(.entry): .triaged}] | add // {}')
for f in ~/.local/share/yaks/farms/*/doks/feedback-to-yaks.md ~/.local/share/yaks/farms/*/.yaks/doks/feedback-to-yaks.md; do
  [ -f "$f" ] || continue
  case "$f" in */tmp.*) continue;; esac
  farm=${f%/doks/feedback-to-yaks.md}; farm=${farm%/.yaks}
  repo=$(git -C "$farm" config --get remote.origin.url | sed -E 's#(\.git)?$##; s#.*[:/]([^/]+/[^/]+)$#\1#')
  awk -v repo="$repo" '
    /^## / { if (h) print h "\t" u; h = repo "#" substr($0, 4); u = ""; next }
    match($0, /\*\*Update [0-9-]+:/) { u = substr($0, RSTART + 9, 10) }
    END { if (h) print h "\t" u }' "$f" |
  while IFS="$(printf '\t')" read -r key upd; do
    t=$(printf '%s' "$ledger" | jq -r --arg k "$key" '.[$k] // ""')
    if [ -z "$t" ]; then echo "NEW      $key"
    elif [ -n "$upd" ] && [ "$upd" \> "$t" ]; then echo "UPDATED  $key (update $upd > triaged $t)"
    fi
  done
done
