# Example: many teams, one monorepo

Acme runs a marketplace from one repository. Three folders have a `yass/` folder. `yass status` finds every `yass/` folder in the repo, with no config.

| Folder | Who | What's in it |
|---|---|---|
| `yass/` | product | a cross-team change: *three-tap checkout*, with the PRD and the end-to-end acceptance |
| `services/payments/yass/` | payments team | *saved cards API*, a change payments owns |
| `apps/web/yass/` | web team | *preselect a saved card*, a change web owns |

`services/search/` has no `yass/` folder. That's fine: teams opt in by running `yass init <their folder>`.

Things to notice:
- **Cross-team work is just links.** The root change's plan lists which team change delivers each criterion; each team change names the root criterion it delivers. Nobody copies anyone's plan.
- **Web waits on payments.** The web change's `blocked:` names the payments change, so `yass status` shows `waiting on: 2026-09-10-saved-cards-api` until payments ticks its last box. Nobody has to remember to unblock it.
- **Payments adopted YASS late.** Its `README.md` and `docs/api.md` describe what exists today. Its `yass/` folder only describes what it's changing.
- **Ownership is your repo's business.** Use CODEOWNERS on `*/yass/` folders if you want reviews from the owning team.

Try it: copy this folder into a git repo with YASS installed and run `yass status`.
