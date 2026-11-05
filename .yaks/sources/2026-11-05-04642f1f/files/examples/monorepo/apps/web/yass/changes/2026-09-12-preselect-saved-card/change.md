---
platforms: [web]
source: yass/changes/2026-09-01-three-tap-checkout (AC1, AC2)
follows:
blocked: services/payments/yass/changes/2026-09-10-saved-cards-api
---
# Preselect a saved card

## Goal
Checkout preselects the buyer's most recent saved card, and Buy goes straight to confirm. Delivers AC1 and AC2 of three-tap checkout in the web app.

## Acceptance
- [ ] Delivers AC1 and AC2 (root plan.md)
- [ ] Given a buyer with no saved card, when they reach checkout, then the card form shows as today — verify: e2e checkout/no_saved_card

## Steps
- [x] Checkout reads cards from the payments client (mocked)
- [ ] Switch to the real endpoint
- [ ] Buy → confirm in one tap when a card is preselected

## Decisions
- Preselect the most recent card, not a "default" flag - payments has no default flag (priya)

## Log
### 2026-09-20 (claude)
- Did: UI against a mocked client.
- Next: switch to the real endpoint once payments ships it.
