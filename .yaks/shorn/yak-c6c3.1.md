---
id: yak-c6c3.1
title: 'Packaging & release: publish yakherder to PyPI'
type: task
priority: 1
created: '2026-07-25T20:48:14Z'
updated: '2026-07-25T21:16:55Z'
---

Make the repo publish-ready under a NEW PyPI distribution name (yaks is taken by an abandoned ADLINK/zenoh package). Keep the command name 'yaks'; publish as 'yakshave'. Tasks: rename [project.name] to yakshave; add readme/license/classifiers/project.urls; add a 'yakshave' console-script alias alongside 'yaks' (so 'uvx yakshave' works with no --from); unify version with the plugin manifests; verify 'uv build' and 'uvx --from ./dist/*.whl yaks ...'; add a GitHub Actions release workflow using PyPI trusted publishing on tag. NOTE: the actual first upload is user-gated (needs PyPI account / trusted-publisher config).

---
▸ 2026-07-25T21:05:05Z
Name locked: distribution = 'yakherder' (available on PyPI; 'yaks' is taken by a dead project). Command stays 'yaks'. Register BOTH console scripts (yaks + yakherder alias) at yaklib.cli:main so 'uvx yakherder' works with no --from. Docs/README should recommend 'uv tool install yakherder' so 'yaks' (esp. 'yaks tui') stays on PATH permanently.

---
▸ 2026-07-25T21:16:55Z
Done. pyproject: renamed dist to 'yakherder', version 0.1.0->0.1.78, added readme/license(Apache-2.0)/authors/keywords/classifiers/project.urls, added [build-system] (setuptools>=77.0.3), and a 'yakherder' console-script alias next to 'yaks'. Unified marketplace.json + .codex-plugin to 0.1.78. Added .github/workflows/publish.yml (PyPI Trusted Publishing on v* tags; env 'pypi'). Verified: 'uv build' produces sdist+wheel; 'uvx --from ./dist/*.whl yaks --help' and 'uvx --from ./dist/*.whl yakherder list' both run in isolated envs. HAND-OFF: owner must create a PyPI pending publisher for 'yakherder' (repo joelgwebber/yaks, workflow publish.yml, env pypi), then push a v0.1.78 tag to publish.
