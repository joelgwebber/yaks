---
id: yak-140e
title: 'Refactor: test harness for CLI behavior'
type: task
priority: 2
created: '2026-04-14T04:26:32Z'
updated: '2026-04-14T04:28:36Z'
commit: 17ecd77
---

Step 1 of the monolith split. Add pytest skeleton and golden-output CLI tests so subsequent refactor steps have a safety net.

Covers: yak create/list/show/attach/detach/dep/next/tangled/shave/shorn, using --json where available. Runs against temp .yaks/ fixtures.

### 2026-04-14T04:28:26Z
Added tests/ with conftest (yak runner + create_task helper) and three test modules: test_cli_basics.py (init/create/show/list/update), test_cli_workflow.py (shave/shorn/regrow, slaughter/revive, deps, next/tangled, child IDs, search), test_cli_artifacts.py (attach/detach/force/custom-name). 20 tests, all green in ~4s. Added pytest to dev deps; configured testpaths + pythonpath in pyproject.toml.
