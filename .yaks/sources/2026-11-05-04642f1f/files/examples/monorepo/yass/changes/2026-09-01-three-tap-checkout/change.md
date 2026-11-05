---
platforms: [web, ios, android, api]
source: Q4 company bet
follows:
blocked:
---
# Three-tap checkout

## Goal
A returning buyer can buy anything in three taps from any surface: open the item, tap Buy, confirm. Product owns the PRD and the end-to-end acceptance; payments and web each own a piece of the work in their own folders.

## Decisions
- Returning buyers first - they are 70% of checkouts (@acme/product)
- Saved cards live in payments, not in web - card data stays behind the vault (@acme/payments, @acme/web)

## Log
### 2026-09-12 (claude)
- Did: plan agreed; team changes created in payments and web.
- Next: end-to-end flow once both team changes land.
