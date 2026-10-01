---
id: yaks-532e
title: Move yaks + toque + edtui from rocketsurgery-games to joelgwebber (GitHub) and @j15r (npm)
type: chore
priority: 1
created: '2026-10-01T17:09:52Z'
updated: '2026-10-01T17:14:54Z'
labels:
- github
- npm
- infra
---

---
▸ 2026-10-01T17:14:54Z [Joel Webber]
Moved all three repos out of rocketsurgery-games:
- joelgwebber/toque: new repo, pushed main (11 tags) + repointed Cargo.toml/README org refs, origin repointed.
- joelgwebber/edtui: real GitHub fork of preiter93/edtui (gh repo fork, async-polled ready), all 11 branches (main + yaks-integration + 9 yaks/*) pushed from local .edtui clone; origin repointed, upstream remote added back (preiter93/edtui).
- joelgwebber/yaks: new repo. Actions disabled BEFORE seeding (gh api .../actions/permissions enabled=false) to avoid firing 11 historical v0.0.* release workflows on tag push; pushed main+tags; verified zero runs fired; re-enabled; origin repointed.
- npm scope renamed @rocketsurgery -> @j15r (user's existing npm identity; j15r is maintainer of the live @rocketsurgery/* packages, @joelgwebber would need a new npm org, unscoped yaks is taken).
- Rewrote every rocketsurgery-games/@rocketsurgery reference repo-wide incl. historical .yaks/shorn notes (user's explicit call) and the include_str!-embedded skills/yaks/SKILL.md.
- HAZARD CAUGHT: plain  after changing the git URLs re-resolved toque to the new repo's branch tip (cc5d911, 2 commits past what yaks was pinned to) since the old lock entry's source no longer matched the new Cargo.toml URL. Hand-pinned Cargo.lock back to the exact pre-existing commits (toque 792624f, edtui 9fbee79d) under the new URLs; cargo build --release --locked succeeded, proving those exact commits exist in the new repos' history.
Evidence: cargo build --release --locked OK; cargo test --locked (YAKS_SKILLS_AUTOSYNC=0) -> 319+28 pass; docshots -- --ignored green; node npm/yaks/test-mapping.mjs -> ok; git grep rocketsurgery -> zero hits in all 3 repos; both joelgwebber/toque and joelgwebber/edtui confirmed public.
NOT done (needs Joel, documented in RELEASING.md hand-off): create NPM_TOKEN secret on joelgwebber/yaks for the bootstrap publish of 6 new @j15r/* packages, then configure Trusted Publisher (owner joelgwebber) on each, delete the token. Also: npm deprecate the old @rocketsurgery/* packages, and delete the 3 rocketsurgery-games repos once stable.
