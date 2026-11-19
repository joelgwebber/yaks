---
id: yaks-1d2b
title: Links to a dok's or source's directory (x/sketches/, /doks/x/sketches/) are reported as ../ by doctor and refused by peek
status: shorn
type: bug
priority: 3
created: '2026-11-18T23:29:05Z'
updated: '2026-11-19T02:33:15Z'
tags:
- links
- doks
---

Found while designing yaks-93ee ([design dok](/doks/design/files-directories-archives.md)). `links::clean` refuses a trailing `/`, so a dok link to its sibling directory, `[sketches](x/sketches/)` or `/doks/x/sketches/`, parses as `Link::Upward`: `yaks doctor` reports it as climbing out with `../`, and `yaks peek` refuses it. The TUI opens it, because `tui/dirview.rs::farm_dir_of` asks the disk instead of the grammar. yaks-a57c fixed this only for attachment names.

Repro (scratch farm): `yaks dok new doks/x.md --body "[s](x/sketches/)"`, make `doks/x/sketches/a.txt`, then `yaks doctor` and `yaks peek /doks/x/sketches/`.

Fix: the grammar allows a farm path ending in `/` (a directory), as `clean_attachment` does; `exists` and `peek` accept a directory. The design dok folds this into its step 1 (one resolver that knows directories).

#### 2026-11-19T01:32:31Z

*delta:Review yaks-93ee for file and yak handli*

Folded into yaks-84fe (the refactor makes the grammar accept a farm directory path); shear when it lands.

#### 2026-11-19T02:33:14Z

*herd-files*

Fixed by yaks-84fe (herd-files): `links::parse` keeps a trailing `/` on a farm path, and `exists`/`peek` resolve it through `locate::dir_of`. On the repro farm: `yaks doctor` All clear, `yaks peek /doks/x/sketches/` lists the directory. Tests: `a_farm_path_may_name_a_directory`, `exists_and_peek_accept_a_farm_directory`.

#### 2026-11-19T02:33:15Z

*herd-files · kind: move · from: hairy · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shorn
