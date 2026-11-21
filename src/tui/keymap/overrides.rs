//! The user's key overrides (yaks-6919): a `keys:` map in the per-user
//! `$XDG_CONFIG_HOME/yaks/config.yaml` (default `~/.config/yaks/config.yaml`,
//! never the farm), from a command's dotted name to its keys:
//!
//! ```yaml
//! keys:
//!   view.next: ["Ctrl-L", "]"]
//!   farm.sync: Ctrl-U
//!   app.help:
//!     - F2
//! ```
//!
//! The same file picks the key family for the whole TUI (yaks-7c6e), `vim`
//! (the default) or `standard`: `keys: standard` as a scalar, or `family:
//! standard` inside the map above (it is not a command's name, so it cannot
//! clash with one). The legacy `vim_mode: false` is an alias for
//! `standard`. The user's `keys` family beats their `vim_mode`, which beats the
//! farm's `vim_mode`, which beats vim ([`Keymap::load_user`]).
//!
//! A key is written as [`Key::label`] prints it. An override replaces the
//! command's keys in every context it is bound in. Whatever cannot be used (an
//! unknown command, a key that does not parse, an override that would give two
//! commands one key in a context) is left out and reported once, at startup.
//! Editor keys are not in the table and are not touched.
//!
//! The reading, the family rules and the collision checks are toque's
//! (`toque::keys`, yaks-2e7c); this is where yaks keeps its file.

use super::*;

/// The user's keymap, from their config file; the table's keys in the
/// `default` family (the farm's `vim_mode`) when there is no file. The notice
/// says what was left out, in one line.
pub(crate) fn load_user(default: Family) -> (Keymap, Option<String>) {
    let path = views_store::config_home().join("yaks").join("config.yaml");
    Keymap::load(&path, default)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn key(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
    }

    #[test]
    fn every_key_in_the_table_parses_from_its_label() {
        let problems = toque::keys::rules::every_key_parses_from_its_label::<Cmd>();
        assert!(problems.is_empty(), "{problems:#?}");
        assert_eq!(Key::parse("Ctrl-"), None);
        assert_eq!(Key::parse("Hyper"), None);
        assert_eq!(Key::parse(""), None);
        assert_eq!(Key::parse("Left"), Some(k(Left)));
        assert_eq!(Key::parse("Ctrl--"), Some(ctrl('-')));
    }

    #[test]
    fn an_override_replaces_the_commands_keys_in_every_context() {
        let (km, problems) = Keymap::from_config(
            "keys:\n  view.next: [\"Ctrl-L\"]\n  move.down: [\"z\"]\n",
            Family::Vim,
        );
        assert!(problems.is_empty(), "{problems:?}");
        let yak = Some(Yak);
        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(km.lookup(Ctx::List, yak, &ctrl_l), Some(ViewNext));
        // The table's keys for it are gone; the other commands are untouched.
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(km.lookup(Ctx::List, yak, &tab), None);
        assert_eq!(km.lookup(Ctx::List, yak, &key('l')), Some(TreeRight));
        // move.down is bound in the list, the pane and the viewer.
        for ctx in [Ctx::List, Ctx::Detail, Ctx::Viewer] {
            assert_eq!(km.lookup(ctx, yak, &key('z')), Some(MoveDown), "{ctx:?}");
            assert_eq!(km.lookup(ctx, yak, &key('j')), None, "{ctx:?}");
        }
    }

    #[test]
    fn overrides_of_tags_and_type_are_by_command_name_not_by_key() {
        // yak.tags / yak.type moved from L / T to T / Y: an override names the
        // command, so it replaces the new default and the new letters go dead.
        let (km, problems) = Keymap::from_config(
            "keys:\n  yak.tags: [\"I\"]\n  yak.type: [\"L\"]\n",
            Family::Vim,
        );
        assert!(problems.is_empty(), "{problems:?}");
        let yak = Some(Yak);
        for ctx in [Ctx::List, Ctx::Detail] {
            assert_eq!(km.lookup(ctx, yak, &key('I')), Some(YakTags), "{ctx:?}");
            assert_eq!(km.lookup(ctx, yak, &key('L')), Some(YakType), "{ctx:?}");
            assert_eq!(km.lookup(ctx, yak, &key('T')), None, "{ctx:?}");
            assert_eq!(km.lookup(ctx, yak, &key('Y')), None, "{ctx:?}");
        }
    }

    #[test]
    fn the_reference_and_the_hint_show_the_effective_keys() {
        let (km, _) = Keymap::from_config(
            "keys:\n  view.next: [\"Ctrl-L\"]\n  filter.drawer: [\"Ctrl-F\"]\n",
            Family::Vim,
        );
        let all = text(&help_content(&km));
        assert!(all.contains("  Ctrl-L          Next view"), "{all}");
        assert!(!all.contains("Tab / ] / }"));
        let hint = hint_for(&km, Ctx::List, Some(Yak), false);
        assert!(hint.starts_with("Ctrl-L:view  j/k:move"), "{hint}");
        assert!(hint.contains("Ctrl-F:filter  /:search"), "{hint}");
        let popup: Vec<_> = km
            .popup_entries(Ctx::List, Some(Yak))
            .into_iter()
            .flat_map(|(_, e)| e)
            .collect();
        assert!(
            popup.contains(&("Ctrl-L".to_string(), "view.next")),
            "{popup:?}"
        );
    }

    #[test]
    fn what_cannot_be_used_is_reported_and_left_out() {
        let text = "keys:\n  view.nxt: [x]\n  view.next: [\"Ctrl-\", \"Ctrl-L\"]\n  view.prev: [Hyper]\n  tree.toggle: []\n  move.up: [\"l\"]\n";
        let (km, problems) = Keymap::from_config(text, Family::Vim);
        assert_eq!(
            problems,
            vec![
                "view.nxt: no such command".to_string(),
                "view.next: no such key \"Ctrl-\"".to_string(),
                "view.prev: no such key \"Hyper\"".to_string(),
                "tree.toggle: no keys".to_string(),
                "move.up: l is already tree.right's in the list".to_string(),
            ]
        );
        // The good key of a partly bad list still applies; a clash drops the
        // whole override, leaving the table's keys.
        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &ctrl_l), Some(ViewNext));
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &key('k')), Some(MoveUp));
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &key('l')), Some(TreeRight));
    }

    #[test]
    fn two_commands_may_trade_keys() {
        let (km, problems) =
            Keymap::from_config("keys:\n  move.down: [k]\n  move.up: [j]\n", Family::Vim);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &key('k')), Some(MoveDown));
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &key('j')), Some(MoveUp));
    }
    // -- the key family (yaks-7c6e) -----------------------------------------

    fn family(text: &str, farm: Family) -> (Family, Vec<String>) {
        let (km, problems) = Keymap::from_config(text, farm);
        (km.family, problems)
    }

    #[test]
    fn the_family_is_the_users_keys_scalar_else_their_vim_mode_else_the_farms() {
        use Family::{Standard, Vim};
        // No word from the user: the farm's `vim_mode` (the default is vim).
        assert_eq!(family("", Vim).0, Vim);
        assert_eq!(family("", Standard).0, Standard);
        assert_eq!(family("sync_interval: 5m\n", Standard).0, Standard);
        // `keys: <family>` as a scalar beats everything.
        assert_eq!(family("keys: standard\n", Vim), (Standard, vec![]));
        assert_eq!(family("keys: vim\n", Standard), (Vim, vec![]));
        assert_eq!(
            family("keys: \"standard\"\nvim_mode: true\n", Vim).0,
            Standard
        );
        // The legacy `vim_mode:` is an alias, and beats the farm's.
        assert_eq!(family("vim_mode: false\n", Vim).0, Standard);
        assert_eq!(family("vim_mode: true\n", Standard).0, Vim);
        // `family:` inside the overrides map, beside the overrides.
        let (f, problems) = family(
            "keys:\n  family: standard\n  view.next: [\"Ctrl-L\"]\n",
            Vim,
        );
        assert_eq!((f, problems), (Standard, vec![]));
        let (km, _) = Keymap::from_config(
            "keys:\n  family: standard\n  view.next: [\"Ctrl-L\"]\n",
            Vim,
        );
        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(
            km.lookup(Ctx::List, Some(Yak), &ctrl_l),
            Some(ViewNext),
            "the overrides kept"
        );
        assert_eq!(
            km.lookup(Ctx::List, Some(Yak), &key('j')),
            None,
            "and the family applied"
        );
        assert_eq!(
            family("keys:\n  family: standard\nvim_mode: true\n", Vim).0,
            Standard,
            "the keys family beats vim_mode"
        );
    }

    #[test]
    fn a_family_that_does_not_read_is_reported_and_the_next_source_wins() {
        let (f, problems) = family("keys: emacs\nvim_mode: false\n", Family::Vim);
        assert_eq!(f, Family::Standard, "vim_mode is next");
        assert_eq!(
            problems,
            vec!["keys: no such family \"emacs\" (vim, standard)".to_string()]
        );
        let (f, problems) = family("keys:\n  family: [vim, standard]\n", Family::Standard);
        assert_eq!(f, Family::Standard);
        assert_eq!(problems, vec!["family: one of vim, standard".to_string()]);
    }

    #[test]
    fn a_user_override_is_the_users_word_whatever_the_family() {
        // A vim-only key given to a command in standard mode stays on.
        let (km, problems) =
            Keymap::from_config("keys:\n  family: standard\n  move.down: [j]\n", Family::Vim);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(km.family, Family::Standard);
        assert_eq!(km.lookup(Ctx::List, Some(Yak), &key('j')), Some(MoveDown));
    }

    #[test]
    fn function_keys_read_and_write_as_f_n() {
        assert_eq!(Key::parse("F3"), Some(k(KeyCode::F(3))));
        assert_eq!(Key::parse("F(3)"), Some(k(KeyCode::F(3))));
        assert_eq!(Key::parse("Shift-F3"), Some(shift(KeyCode::F(3))));
        assert_eq!(k(KeyCode::F(1)).label(), "F1");
        // A plain `F` is still the key F.
        assert_eq!(Key::parse("F"), Some(c('F')));
        assert_eq!(
            Key::parse("Alt-F"),
            Some(Key {
                code: KeyCode::Char('f'),
                mods: KeyModifiers::ALT
            })
        );
        assert_eq!(
            Key::parse("Alt-F").map(|k| k.label()),
            Some("Alt-F".to_string())
        );
    }
}
