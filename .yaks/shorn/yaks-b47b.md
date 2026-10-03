---
id: yaks-b47b
title: No default or explicit herd should error in `yaks create`
type: bug
priority: 3
created: '2026-09-23T03:52:41Z'
updated: '2026-10-03T21:21:41Z'
labels:
- cli
- herds
---

This can be quite a weird foot-gun, because you end up falling back to `yak-`, and can end up accidentally creating a new, unintended herd.

---
▸ 2026-10-03T21:14:11Z [delta-lead]
TRIAL 1 (yaks-fe33), worker A 'trial1-haiku' (profile default model, Haiku 4.5). Claim committed on main before spawn. Scope: Farm::create in src/farm.rs and its tests; clap help for create in src/main.rs and docs/cli.md if behavior text changes. Gate: cargo test -p yaks. Also: a new test that fails before and passes after, and an attached transcript of the built binary in a throwaway farm with no herd configured (must error, non-zero) and with a default herd (must still work). Judge: coordinator re-runs the gate.

---
▸ 2026-10-03T21:15:48Z [trial1-haiku]
START: .yaks/shaving/yaks-b47b.md (status dir: shaving) at Sat Oct  3 21:15:48 UTC 2026. target/ did NOT exist before build; release build took 21.1s (real 0m21.097s).

FINDINGS (before code change): the fallback is in Farm::create (src/farm.rs). Route = explicit NewTask.prefix (CLI --herd) > self.pointer_prefix (.yaks pointer file 'herd:' or legacy 'prefix:', set in store::discover) > cfg.prefix (config.yaml top-level 'herd:'/legacy 'prefix:'). The last step is the bug: store::read_config seeds Config.prefix = "yak" when config.yaml is missing OR has no herd/prefix key, so None => cfg.prefix silently yields 'yak-'. read_config cannot tell 'default yak' from 'explicit herd: yak'. 'No default' therefore = no --herd AND no pointer herd AND no non-empty top-level herd:/prefix: line in config.yaml. 'yaks init' always writes 'herd: <p>' (store::init), so initialised farms are unaffected. Plan: Farm::create returns an Err (anyhow) instead of adding a CreateOutcome variant (a new variant would force edits in out-of-scope src/tui/handlers.rs; the TUI already surfaces Err as a notification). Detection is a small private helper in farm.rs since store.rs (where Config lives) is out of scope; follow-up could move it to a Config field.

---
▸ 2026-10-03T21:17:03Z [trial1-haiku]
BLOCKED on scope. Fix implemented in farm.rs (+ clap help in main.rs, docs/cli.md) and the new test create_with_no_explicit_or_default_herd_errors FAILS on unfixed code ('panicked at src/farm.rs:1545:14: create with no herd must error') and passes with the fix. But the gate 'cargo test -p yaks' then FAILS: 316 passed; 6 failed — tui::tests::live::{create_child_sets_parent, create_form_sets_priority_and_labels, create_form_source_is_saved, create_root_via_form, labels_edit_normalizes_commas_and_spaces, reload_preserving_selection_picks_up_external_add}. Cause: the TUI test helper temp_farm() in src/tui/test_support.rs builds a .yaks/ with NO config.yaml, so TUI creates there relied on the silent 'yak-' fallback — exactly what this yak removes. Needs a one-line out-of-scope change: in test_support::temp_farm add fs::write(root.join("config.yaml"), "herd: yak\n").unwrap(); (after the schema write). QUESTION: may I edit src/tui/test_support.rs for that? I reverted my code edits to keep the tree unbroken; the full WIP diff is saved at /var/folders/j3/ngc8z3h567v_l779rt0gtdlw0000gp/T/delta-terminal-.delta-fs-Xb1Nrt/yaks-b47b-wip.patch (git apply it). Not committed.

---
▸ 2026-10-03T21:18:19Z [trial1-haiku]
Scope extended to src/tui/test_support.rs by coordinator delta-lead (one-line temp_farm config.yaml); human may overrule.

![transcript](artifacts/yaks-b47b/transcript.txt)

---
▸ 2026-10-03T21:19:27Z [trial1-haiku]
SUMMARY. Farm::create (src/farm.rs) now bails when there is no --herd, no .yaks pointer herd, and no non-empty top-level herd:/prefix: in config.yaml (private helper config_declares_default_herd, since store::read_config seeds "yak" and cannot distinguish). Clap help (main.rs create --herd) and docs/cli.md updated. Scope extended (coordinator-approved) to src/tui/test_support.rs: temp_farm now writes config.yaml 'herd: yak' since 6 TUI tests relied on the silent yak- fallback. The existing farm test create_without_a_prefix_uses_the_config_default now writes 'herd: core'.
EVIDENCE. New test farm::tests::create_with_no_explicit_or_default_herd_errors, BEFORE fix: 'test farm::tests::create_with_no_explicit_or_default_herd_errors ... FAILED / panicked at src/farm.rs:1545:14: create with no herd must error / test result: FAILED. 0 passed; 1 failed'. AFTER fix: 'test farm::tests::create_with_no_explicit_or_default_herd_errors ... ok / test result: ok. 1 passed'. Gate cargo test -p yaks: exit 0; 'test result: ok. 322 passed; 0 failed; 3 ignored' (unit) and 'test result: ok. 28 passed; 0 failed' (cli). Without the test_support change the gate was 316 passed; 6 failed (tui::tests::live::create_* etc.). Binary transcript attached (transcript.txt): no-herd farm -> 'Error: no herd to create the yak in: pass `--herd <prefix>` or set a default herd (...)' exit=1 and .yaks/hairy stays empty; --herd proj works; init --herd core farm -> 'Created core-e741' exit=0; pointer herd -> 'Created ptr-e344'.
LEARNED. The error surfaces via anyhow in main as 'Error: ...' exit 1 (not the 'error:' style of other outcomes); a CreateOutcome variant would be tidier for the CLI but touches tui/handlers.rs. A follow-up could expose 'herd explicitly configured' on store::Config instead of the farm.rs helper.

---
▸ 2026-10-03T21:21:41Z [delta-lead]
COORDINATOR REVIEW (delta-lead). Re-ran in my own checkout: cargo test -p yaks 323 passed + 28 passed; built binary in a throwaway farm: no config -> 'Error: no herd to create the yak in...' rc=1 and nothing written; --herd proj and a config herd still create; after yaks init, create works. Review change, made as a follow-up commit so the worker's d2833d3 stays intact: replaced the worker's farm.rs helper that re-scanned config.yaml with a Config.prefix_declared flag set in store::read_config (one parser, so quoting and the legacy prefix: alias cannot drift; verified quoted herd and prefix: both still work). Also fixed README.md, which still said the herd defaulted to yak. Scope extension to src/tui/test_support.rs (one line) was my call as coordinator, not the human's; overrule if you disagree. Known rough edge: the error prints as 'Error: ...' (anyhow) and is ~150 chars in the TUI notification (handlers.rs surfaces Err as 'error: {e}').
