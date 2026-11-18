# How YASS compares

*As of October 2026. Corrections welcome.*

**Short version:** YASS is a thin, file-based project-management layer: change folders whose ceremony scales with scope, progress and decisions recorded next to the code, and an archive that tells the story of how the system evolved. It deliberately leaves guardrails, approvals and verification to your repo. Most spec tools are built around a fixed multi-phase process. Others overlap with the code and documentation and create opportunity for drift.

| | Unit of work | Large work | Current-state spec | Process | Footprint |
|---|---|---|---|---|---|
| **OpenSpec** | a change folder: proposal, delta specs, design, tasks | one change per proposal; no layer above it | yes: `specs/`, updated when a change is archived | propose → apply → archive | CLI (Node) + slash commands for 30+ tools |
| **GitHub Spec Kit** | a feature: spec → plan → tasks | per feature | mostly spec-first | constitution + phased commands | CLI + templates |
| **Kiro** | a spec: requirements (EARS) → design → tasks | per spec | per spec | IDE-guided | Kiro IDE / CLI |
| **GSD** | phases from a project roadmap | project roadmap + state file | — | phase commands with verification | slash commands |
| **BMAD** | PRD → architecture → stories, via agent personas | yes, heavy | documents | role-based | many agents and templates |
| **YASS** | a change folder: one `change.md` for small work; `prd.md`, `plan.md`, `design.md` and PR-sized pieces for large work | the same folder, with pieces inside | no: the code and its docs are the current state | none imposed; five optional playbooks | small CLI (one Go binary), plain markdown, optional hook |

## Against OpenSpec, specifically

OpenSpec is the closest neighbor and very good at what it does. The differences are deliberate:
- **No current-state spec.** OpenSpec merges each change's delta into living `specs/`. YASS describes evolution only and leaves "what exists today" to the code and its docs, so there's no second description to keep in sync. If you want per-capability behavior specs, use OpenSpec for those; the two can live side by side.
- **Ceremony scales inside one folder.** A bug fix is a single file; a new system gets a PRD, a plan, a design doc and pieces, without a separate roadmap or epic layer.
- **Progress lives in the change.** Ticked boxes, decisions with who made them, and a log ending in **Next:** let any agent on any harness resume from the files alone.
- **Finishing is separate from merging.** A change can span many pull requests and plan revisions before it's archived.

## When not to use YASS
- A prototype you'll throw away. Use plan mode and tests.
- You want a maintained, per-capability description of current behavior. Use OpenSpec or similar.
- You want the tool to enforce approvals, verification or merge rules. YASS records them; your CI and review enforce them.
