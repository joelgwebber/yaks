# Three-tap checkout: plan

## Approach
Payments exposes a buyer's saved cards; web preselects the most recent one at checkout. No new storage.

## Acceptance
### M1
- [ ] AC1 (R1) Given a buyer with a saved card, when they reach checkout, then the card is preselected — verify: e2e checkout/saved_card (web)
- [ ] AC2 (R2) Given a buyer with a saved card, when they buy from the item page, then it takes three taps — verify: e2e checkout/three_taps (web)

## Pieces
Team changes, in their own folders:
1. payments: `services/payments/yass/changes/2026-09-10-saved-cards-api`, delivers the API for AC1.
2. web: `apps/web/yass/changes/2026-09-12-preselect-saved-card`, delivers AC1 and AC2 in the UI. Needs 1.

## Validation
Both e2e flows on staging, plus a manual run with a real test card.
