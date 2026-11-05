# Example: a solo mobile app

Moodlog is a small iOS/Android/web mood journal built by one developer (Sam) and coding agents. It adopted YASS on day one, so its archive is the story of the whole app.

```
yass/
├── changes/
│   ├── 2026-09-14-offline-sync/          large, in progress
│   │   ├── change.md  prd.md  plan.md  design.md
│   │   ├── 2026-09-16-offline-queue/     piece, in progress (read its Log to see how an agent resumes)
│   │   └── 2026-09-24-sync-badge/        piece, blocked on a question for Sam
│   └── 2026-10-02-fix-double-tap-save/   small, from a bug report
├── queue.md                              what to tackle first: the crash, then sync
└── archive/2026/09/                      finished in September 2026
    ├── 2026-08-03-mood-logging/          the first feature, finished
    └── 2026-09-08-fix-history-sort/      a finished bug fix
```

Things to notice:
- **Ceremony scales.** The bug fix is one file. Offline sync has a PRD, a plan, a design doc for its one hard call, and two PR-sized pieces.
- **Progress lives next to the work.** Ticked boxes, Decisions with who made them, and a Log that ends with **Next:**.
- **An order to work in.** `queue.md` puts the crash fix ahead of offline sync, though it's newer; `yass status` lists it first.
- **Intent vs. progress.** The PRD, plan and design were committed on their own. The pieces tick plan boxes as they deliver them.

Try it: copy `yass/` into a git repo with YASS installed and run `yass status`, then `yass status offline-queue`.
