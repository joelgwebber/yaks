---
id: yak-cae6
title: Crash on unescaped colons in frontmatter
type: bug
priority: 1
created: '2026-08-15T17:24:32Z'
updated: '2026-08-16T18:40:45Z'
---

Any unescaped colon in yak frontmatter dies with this stack:

```
Traceback (most recent call last):
  File "/Users/joel/.local/bin/yaks", line 10, in <module>
    sys.exit(main())
             ^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaklib/cli.py", line 21, in main
    curses.wrapper(tui_main)
  File "/Users/joel/.local/share/uv/python/cpython-3.12.11-macos-aarch64-none/lib/python3.12/curses/__init__.py", line 94, in wrapper
    return func(stdscr, *args, **kwds)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 1196, in main
    tui.run()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 464, in run
    self._check_fs_changes()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 393, in _check_fs_changes
    self._reload_preserving_position()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 359, in _reload_preserving_position
    self.reload()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 280, in reload
    self._rebuild_detail()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/tui.py", line 409, in _rebuild_detail
    self.detail_lines = build_detail_lines(self.root, task, status, width, reverse_deps=self.reverse_deps)
                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaktui/detail.py", line 112, in build_detail_lines
    children = find_children(root, task["id"])
               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaklib/model.py", line 299, in find_children
    task = load_task(f)
           ^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaklib/model.py", line 197, in load_task
    task = yaml.safe_load(fm) or {}
           ^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/__init__.py", line 125, in safe_load
    return load(stream, SafeLoader)
           ^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/__init__.py", line 81, in load
    return loader.get_single_data()
           ^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/constructor.py", line 49, in get_single_data
    node = self.get_single_node()
           ^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/composer.py", line 36, in get_single_node
    document = self.compose_document()
               ^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/composer.py", line 55, in compose_document
    node = self.compose_node(None, None)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/composer.py", line 84, in compose_node
    node = self.compose_mapping_node(anchor)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/composer.py", line 127, in compose_mapping_node
    while not self.check_event(MappingEndEvent):
              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/parser.py", line 98, in check_event
    self.current_event = self.state()
                         ^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/parser.py", line 428, in parse_block_mapping_key
    if self.check_token(KeyToken):
       ^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/scanner.py", line 116, in check_token
    self.fetch_more_tokens()
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/scanner.py", line 223, in fetch_more_tokens
    return self.fetch_value()
           ^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yakherder/lib/python3.12/site-packages/yaml/scanner.py", line 577, in fetch_value
    raise ScannerError(None, None,
yaml.scanner.ScannerError: mapping values are not allowed here
  in "<unicode string>", line 2, column 88:
     ... s-matched entities; add messages:
```

---
▸ 2026-08-16T18:40:45Z
Fixed. Root cause: load_task called the strict YAML loader on frontmatter and let yaml.YAMLError (ScannerError on an unescaped colon) propagate, so a single bad file crashed both full scans and point reads. Post-3fd4 the index's except-skip stopped the scan crash but then silently DROPPED the file (yak vanished, incl. from its parent's children). Fix at the load_task chokepoint (covers every caller at once): on YAMLError, recover via new _lenient_frontmatter (never raises; extracts top-level key: value scalars and key:/- item lists, coerces priority to int for sort-safety) — which usually recovers the intended value anyway since YAML only choked on the second colon; force id = path.stem (filename is authoritative; the index + find_task_file key on it); set a private _error flag. save_task now strips _-prefixed keys so the flag (and any future private field) never gets written back. detail pane shows a '⚠ Unparseable frontmatter (...)' line so the recovered yak visibly signals it needs a fix. Net: no crash on scans OR point reads; malformed yaks stay visible, attached, and editable; fixing + saving clears the flag. Tests: tests/test_malformed.py (recovery-not-raise, stays-visible-in-scans, point-read, id-from-filename, save strips flag, CLI show/list survive). Full suite 143 pass.
