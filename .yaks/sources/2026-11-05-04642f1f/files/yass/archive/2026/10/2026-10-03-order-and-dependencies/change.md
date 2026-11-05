---
platforms: [all]
source: 
follows: 
blocked:
archived: 2026-10-04T02:28:40Z
---
# Order and dependencies

## Goal
<!-- Intent. One paragraph; the detail is in prd.md (why and what) and plan.md (how and acceptance). -->
A project can say what order it wants to tackle its changes in, and which changes have to wait for others, with one optional file and the existing `blocked:` key. `yass status` lists work in that order and says what each change is waiting on, so "what's next?" has an answer that lives in the repo.

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)" for calls that span the whole change. -->
- Order lives in one file per yass folder, not in each change - reordering is a one-file diff that reads as a decision, with no merge conflicts across folders (Jaime, claude)
- Dependencies reuse `blocked:` instead of a new `needs:` key - one key for "can't proceed"; a value naming changes clears itself, anything else stays a reason for a human (Jaime)
- The file is `queue.md`, not `next.md` - `next:` already means the Log's last Next line in `yass status`; `backlog.md` reads as a graveyard (claude)
- A dependency is met when the change it names is done, not only when it's archived - progress travels with the code, so done boxes on main mean the code is there; waiting for the archive commit adds nothing (claude)
- Entries name change folders exactly, not shortened names - a short name can become ambiguous when a later change is created; exact names stay stable (claude)

## Log
<!-- Progress. Change-level notes; each piece keeps its own Log.
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
