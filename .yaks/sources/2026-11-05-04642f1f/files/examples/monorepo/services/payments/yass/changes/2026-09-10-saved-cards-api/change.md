---
platforms: [api]
source: yass/changes/2026-09-01-three-tap-checkout (AC1)
follows:
blocked:
---
# Saved cards API

## Goal
`GET /buyers/{id}/cards` returns token ids, brand and last4 for a buyer's saved cards, so checkout can offer them. Part of three-tap checkout (root change, AC1).

## Acceptance
- [x] Given a buyer with two saved cards, when calling the endpoint, then both come back, newest first, with brand and last4 only — verify: contract test cards_list
- [ ] Given another buyer's id, when calling the endpoint, then it returns 403 — verify: contract test cards_list_forbidden

## Steps
- [x] Handler and query
- [x] Contract test cards_list
- [ ] Authorization check and its test
- [ ] Add the endpoint to docs/api.md

## Decisions
- Return brand and last4 only - enough for the UI, nothing sensitive (codex, approved by @acme/payments)
- Cache per buyer for 60s - the vault rate-limits listing (codex)

## Log
### 2026-10-02 (codex)
- Did: handler, query, cards_list passes.
- Next: authorization check, then document the endpoint.
