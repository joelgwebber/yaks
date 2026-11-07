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
//! The file is read by a small pass over the YAML subset above, as the farm's
//! `config.yaml` is: only whole-line comments, and a punctuation key such as
//! `]` or `,` must be quoted.

use super::*;

/// One `name: keys` entry of the section: the command's name and its raw key
/// strings, or why the entry could not be read.
type Entry = (String, Result<Vec<String>, String>);

/// The entries of the top-level `keys:` section of `text`.
fn parse_section(text: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    let (mut in_keys, mut block_open) = (false, false);
    for line in text.lines() {
        let bare = line.trim();
        if bare.is_empty() || bare.starts_with('#') {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            in_keys = bare
                .strip_prefix("keys:")
                .is_some_and(|v| v.trim().is_empty());
            block_open = false;
            continue;
        }
        if !in_keys {
            continue;
        }
        if let Some(item) = bare.strip_prefix('-') {
            if let (true, Some((_, Ok(keys)))) = (block_open, out.last_mut()) {
                keys.push(unquote(item.trim()));
            }
            continue;
        }
        let Some((name, value)) = bare.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim().to_string(), value.trim());
        block_open = value.is_empty();
        let keys = match value.strip_prefix('[') {
            Some(list) => flow_list(list),
            None if block_open => Ok(Vec::new()),
            None => Ok(vec![unquote(value)]),
        };
        out.push((name, keys));
    }
    out
}

/// The family the config's top level names, and what it got wrong: the scalar
/// `keys: <family>`, else the legacy `vim_mode:`. (`family:` inside the `keys`
/// map is an [`Entry`], read by [`Keymap::from_config`].)
fn top_level_family(text: &str) -> (Option<Family>, Option<Family>, Vec<String>) {
    let (mut keys, mut vim_mode, mut problems) = (None, None, Vec::new());
    for line in text.lines() {
        let bare = line.trim();
        if bare.is_empty() || bare.starts_with('#') || line.starts_with([' ', '\t']) {
            continue;
        }
        let Some((name, value)) = bare.split_once(':') else {
            continue;
        };
        let value = unquote(value.trim());
        match name.trim() {
            "keys" if !value.is_empty() => match Family::from_name(&value) {
                Some(f) => keys = Some(f),
                None => problems.push(format!("keys: no such family \"{value}\" (vim, standard)")),
            },
            // As the farm's own `vim_mode` reads it.
            "vim_mode" => {
                vim_mode = Some(Family::from_vim_mode(matches!(
                    value.as_str(),
                    "true" | "True" | "yes" | "1"
                )))
            }
            _ => {}
        }
    }
    (keys, vim_mode, problems)
}

/// A scalar without its YAML quotes (`"x"`, or `'x'` with `''` for a quote).
fn unquote(s: &str) -> String {
    let mut it = s.chars();
    match (it.next(), it.next_back()) {
        (Some(q @ ('"' | '\'')), Some(end)) if q == end && s.len() >= 2 => {
            unescape(&s[1..s.len() - 1], q)
        }
        _ => s.to_string(),
    }
}

fn unescape(inner: &str, quote: char) -> String {
    let mut out = String::new();
    let mut it = inner.chars().peekable();
    while let Some(c) = it.next() {
        match (quote, c, it.peek()) {
            ('\'', '\'', Some('\'')) => {
                it.next();
                out.push('\'');
            }
            ('"', '\\', Some(_)) => out.extend(it.next()),
            _ => out.push(c),
        }
    }
    out
}

/// The items of a flow list whose `[` is already consumed: quoted or bare
/// strings, comma-separated, up to the closing `]`.
fn flow_list(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut rest = s;
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        let mut chars = rest.chars();
        match chars.next() {
            None => return Err("the list has no closing ]".into()),
            Some(']') => {
                return match rest[1..].trim() {
                    "" => Ok(out),
                    _ => Err("text after the list (quote a ] key: \"]\")".into()),
                };
            }
            Some(q @ ('"' | '\'')) => {
                // Up to the closing quote (a backslash, or a doubled single
                // quote, escapes one).
                let (mut end, mut esc) = (None, false);
                let mut it = rest.char_indices().skip(1).peekable();
                while let Some((i, c)) = it.next() {
                    if esc {
                        esc = false;
                    } else if q == '"' && c == '\\' {
                        esc = true;
                    } else if c == q {
                        if q == '\'' && it.peek().is_some_and(|(_, n)| *n == '\'') {
                            it.next();
                        } else {
                            end = Some(i);
                            break;
                        }
                    }
                }
                let end = end.ok_or("an unclosed quote")?;
                out.push(unescape(&rest[1..end], q));
                rest = &rest[end + 1..];
            }
            Some(_) => {
                let end = rest.find([',', ']']).unwrap_or(rest.len());
                out.push(rest[..end].trim().to_string());
                rest = &rest[end..];
            }
        }
    }
}

impl Keymap {
    /// The overrides in the `keys:` section of `text`, and what was left out;
    /// the family is the config's own, else `default` (the farm's).
    pub(crate) fn from_config(text: &str, default: Family) -> (Keymap, Vec<String>) {
        let mut km = Keymap::default();
        let (scalar, vim_mode, mut problems) = top_level_family(text);
        let mut family = scalar;
        for (name, raw) in parse_section(text) {
            if name == "family" {
                match raw.as_deref() {
                    Ok([f]) => match Family::from_name(f) {
                        Some(f) => family = Some(f),
                        None => {
                            problems.push(format!("family: no such family \"{f}\" (vim, standard)"))
                        }
                    },
                    _ => problems.push("family: one of vim, standard".into()),
                }
                continue;
            }
            let Some(cmd) = Cmd::from_name(&name) else {
                problems.push(format!("{name}: no such command"));
                continue;
            };
            let raw = match raw {
                Ok(raw) => raw,
                Err(why) => {
                    problems.push(format!("{name}: {why}"));
                    continue;
                }
            };
            if raw.is_empty() {
                problems.push(format!("{name}: no keys"));
                continue;
            }
            let mut keys: Vec<Key> = Vec::new();
            for r in &raw {
                match Key::parse(r) {
                    Some(key) if !keys.contains(&key) => keys.push(key),
                    Some(_) => {}
                    None => problems.push(format!("{name}: no such key \"{r}\"")),
                }
            }
            if !keys.is_empty() {
                km.overrides.insert(cmd, keys);
            }
        }
        km.family = family.or(vim_mode).unwrap_or(default);
        problems.extend(km.drop_collisions());
        (km, problems)
    }

    /// The user's keymap, from their config file; the table's keys in the
    /// `default` family (the farm's `vim_mode`) when there is no file. The
    /// notice says what was left out, in one line.
    pub(crate) fn load_user(default: Family) -> (Keymap, Option<String>) {
        let path = views_store::config_home().join("yaks").join("config.yaml");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return (
                Keymap {
                    family: default,
                    ..Keymap::default()
                },
                None,
            );
        };
        let (km, problems) = Keymap::from_config(&text, default);
        // The notice is one line, so a long list says how many more.
        let more = problems.len().saturating_sub(2);
        let more = if more > 0 {
            format!("; +{more} more")
        } else {
            String::new()
        };
        let notice = (!problems.is_empty()).then(|| {
            let shown = problems.iter().take(2).cloned().collect::<Vec<_>>();
            format!("keys in config.yaml: {}{more} (ignored)", shown.join("; "))
        });
        (km, notice)
    }

    /// Drop each override that gives two commands one key in a context and
    /// kind of row (the rule the table itself keeps), saying which.
    fn drop_collisions(&mut self) -> Vec<String> {
        let mut problems = Vec::new();
        while let Some((victim, key, other, ctx)) = self.collision() {
            self.overrides.remove(&victim);
            problems.push(format!(
                "{}: {} is already {}'s in the {ctx}",
                victim.name(),
                key.label(),
                other.name()
            ));
        }
        problems
    }

    /// One shared key to resolve: the overridden command to drop, the key, the
    /// command it clashes with, and the context's name. Two bindings clash when
    /// they share a key in a context and their rows overlap (the same kind, or
    /// one is for any kind), because the table lets a kind's binding shadow a
    /// general one, and a shadowed override would silently not work.
    fn collision(&self) -> Option<(Cmd, Key, Cmd, &'static str)> {
        for ctx in Ctx::ALL {
            let group: Vec<&Binding> = BINDINGS.iter().filter(|b| b.ctx == ctx).collect();
            for (i, a) in group.iter().enumerate() {
                for b in &group[i + 1..] {
                    let overlap = a.kind == b.kind || a.kind.is_none() || b.kind.is_none();
                    if a.cmd == b.cmd || !overlap {
                        continue;
                    }
                    let Some(key) = self.keys(a).find(|k| self.keys(b).any(|x| x == *k)) else {
                        continue;
                    };
                    let (victim, other) = if self.overrides.contains_key(&b.cmd) {
                        (b.cmd, a.cmd)
                    } else if self.overrides.contains_key(&a.cmd) {
                        (a.cmd, b.cmd)
                    } else {
                        continue;
                    };
                    return Some((victim, *key, other, ctx.title()));
                }
            }
        }
        None
    }
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
        for b in BINDINGS {
            for k in b.keys {
                assert_eq!(Key::parse(&k.label()), Some(*k), "{}", k.label());
            }
        }
        assert_eq!(Key::parse("Ctrl-"), None);
        assert_eq!(Key::parse("Hyper"), None);
        assert_eq!(Key::parse(""), None);
        assert_eq!(Key::parse("Left"), Some(k(Left)));
        assert_eq!(Key::parse("Ctrl--"), Some(ctrl('-')));
    }

    #[test]
    fn the_section_reads_flow_block_and_scalar_forms() {
        let text = "herd: yak\nkeys:\n  # a comment\n  view.next: [\"Tab\", \"]\", ',', x]\n  farm.sync: Ctrl-U\n  app.help:\n    - F(1)\n    - \"?\"\nother:\n  app.quit: z\n";
        let got = parse_section(text);
        let ok =
            |n: &str, v: &[&str]| (n.to_string(), Ok(v.iter().map(|s| s.to_string()).collect()));
        assert_eq!(
            got,
            vec![
                ok("view.next", &["Tab", "]", ",", "x"]),
                ok("farm.sync", &["Ctrl-U"]),
                ok("app.help", &["F(1)", "?"]),
            ]
        );
        assert!(parse_section("keys:\n  a.b: [Tab, ]]\n")[0].1.is_err());
        assert!(parse_section("keys:\n  a.b: [\"Tab\n")[0].1.is_err());
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
    fn the_reference_and_the_hint_show_the_effective_keys() {
        let (km, _) = Keymap::from_config(
            "keys:\n  view.next: [\"Ctrl-L\"]\n  filter.drawer: [\"Ctrl-F\"]\n",
            Family::Vim,
        );
        let all = text(&help_content(&km));
        assert!(all.contains("  Ctrl-L          Next view"), "{all}");
        assert!(!all.contains("Tab / ] / }"));
        let hint = km.hint_for(Ctx::List, Some(Yak), false);
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
