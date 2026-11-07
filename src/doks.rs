//! Doks (yaks-9af0, design yaks-aae6): markdown documents in a farm that are
//! not yaks. A dok lives under a doks root, the farm-wide `doks/` or one
//! herd's `<herd>/doks/`, at any depth, and is addressed by its path from the
//! farm root (`doks/topics/vector-search.md`).
//!
//! A dok is frontmatter plus a markdown body, in OKF's shape (`type`,
//! `title`, `description`, `tags`, `sources`, `status`, `stale_after`, and
//! anything else). Frontmatter is a [`Fields`], so every key round-trips as
//! written and an edit rewrites only the key it sets. Doks have no notes log
//! and no `updated:` field: git says when a dok changed.
//!
//! Untyped kinds need no schema: `note` (the default), `topic`, `howto`,
//! `summary`. Any other `type` is a typed kind, described by
//! `schemas/<type>.md` (yaks-8e75).

use crate::fields::{Fields, Value};
use crate::store;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// The directory name of a doks root, farm-wide (`doks/`) or per herd
/// (`<herd>/doks/`).
pub const DOKS_DIR: &str = "doks";

/// The kinds of dok that need no schema.
pub const UNTYPED_KINDS: &[&str] = &["note", "topic", "howto", "summary"];

/// The kind a dok has when its frontmatter names none.
pub const DEFAULT_KIND: &str = "note";

/// One dok: its farm-relative path, frontmatter and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dok {
    /// From the farm root, `/`-separated: `doks/topics/x.md`.
    pub path: String,
    pub fields: Fields,
    pub body: String,
    /// False when the file has no `---` frontmatter block at all (doctor
    /// reports it; the whole text is then the body).
    pub has_frontmatter: bool,
}

impl Dok {
    /// Parse a dok file's text.
    pub fn parse(path: &str, text: &str) -> Dok {
        match store::split_frontmatter(text) {
            Some((front, body)) => Dok {
                path: path.to_string(),
                fields: Fields::parse(front),
                body: body.to_string(),
                has_frontmatter: true,
            },
            None => Dok {
                path: path.to_string(),
                fields: Fields::default(),
                body: text.to_string(),
                has_frontmatter: false,
            },
        }
    }

    /// The file text: frontmatter (always written, even when empty) then the
    /// body exactly as held.
    pub fn render(&self) -> String {
        let mut s = String::from("---\n");
        s.push_str(&self.fields.render());
        s.push_str("---\n");
        if !self.body.is_empty() {
            if !self.body.starts_with('\n') {
                s.push('\n');
            }
            s.push_str(&self.body);
            if !self.body.ends_with('\n') {
                s.push('\n');
            }
        }
        s
    }

    /// A scalar field as text (`None` when absent or not a scalar).
    pub fn str_field(&self, key: &str) -> Option<String> {
        self.fields.get(key)?.as_str().map(String::from)
    }

    /// The dok's kind: its `type:`, else `note`.
    pub fn kind(&self) -> String {
        self.str_field("type")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_KIND.to_string())
    }

    /// Its `title:`, else the first `# ` heading, else the file name.
    pub fn title(&self) -> String {
        if let Some(t) = self.str_field("title").filter(|s| !s.is_empty()) {
            return t;
        }
        if let Some(h) = self
            .body
            .lines()
            .find_map(|l| l.strip_prefix("# ").map(str::trim))
        {
            return h.to_string();
        }
        let stem = self.path.rsplit('/').next().unwrap_or(&self.path);
        stem.strip_suffix(".md").unwrap_or(stem).to_string()
    }

    /// Its `tags:` (a list, or one scalar).
    pub fn tags(&self) -> Vec<String> {
        self.fields
            .get("tags")
            .map(|v| {
                v.as_list()
                    .into_iter()
                    .filter_map(|i| i.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The herd whose doks holds it (`None` for the farm-wide doks).
    pub fn herd(&self) -> Option<&str> {
        let (first, rest) = self.path.split_once('/')?;
        (first != DOKS_DIR && rest.starts_with("doks/")).then_some(first)
    }
}

/// Why `rel` (a farm-relative path) is not a valid place for a new dok, or
/// `None` when it is: `doks/<parts>.md` or `<herd>/doks/<parts>.md`, every
/// part lowercase letters, digits and `-` (a kebab name), at least one part
/// below the doks root, ending in `.md`.
pub fn path_problem(rel: &str) -> Option<String> {
    let parts: Vec<&str> = rel.split('/').collect();
    let below = match parts.as_slice() {
        [w, rest @ ..] if *w == DOKS_DIR => rest,
        [herd, w, rest @ ..] if *w == DOKS_DIR && !herd.starts_with('.') => {
            if store::RESERVED_HERD_NAMES.contains(herd) {
                return Some(format!("`{herd}` is not a herd"));
            }
            rest
        }
        _ => return Some("a dok lives under doks/ or <herd>/doks/".to_string()),
    };
    let Some((file, dirs)) = below.split_last() else {
        return Some("name the dok file below the doks directory".to_string());
    };
    let Some(stem) = file.strip_suffix(".md") else {
        return Some("a dok file ends in .md".to_string());
    };
    let kebab = |s: &str| {
        !s.is_empty()
            && !s.starts_with('-')
            && !s.ends_with('-')
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    for part in dirs.iter().chain(std::iter::once(&stem)) {
        if !kebab(part) {
            return Some(format!(
                "`{part}` is not a lowercase-kebab name (a-z, 0-9 and -)"
            ));
        }
    }
    None
}

/// Every doks root of the farm at `root` that exists: `doks/` and each
/// `<herd>/doks/`, as `(farm-relative dir, absolute dir)`.
fn doks_roots(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let farm_wide = root.join(DOKS_DIR);
    if farm_wide.is_dir() {
        out.push((DOKS_DIR.to_string(), farm_wide));
    }
    if let Ok(rd) = std::fs::read_dir(root) {
        let mut herds: Vec<String> = rd
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|n| !n.starts_with('.') && !store::RESERVED_HERD_NAMES.contains(&n.as_str()))
            .collect();
        herds.sort();
        for h in herds {
            let dir = root.join(&h).join(DOKS_DIR);
            if dir.is_dir() {
                out.push((format!("{h}/{DOKS_DIR}"), dir));
            }
        }
    }
    out
}

/// Every dok file of the farm at `root`, as farm-relative paths, sorted. Dot
/// files and directories are skipped; only `.md` files are doks (anything
/// else under a doks root, an image, is a file a dok links to).
pub fn dok_paths(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (rel, dir) in doks_roots(root) {
        let mut stack = vec![(rel, dir)];
        while let Some((rel, dir)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in rd.flatten() {
                let Some(name) = e.file_name().to_str().map(String::from) else {
                    continue;
                };
                if name.starts_with('.') {
                    continue;
                }
                let path = e.path();
                let child = format!("{rel}/{name}");
                if path.is_dir() {
                    stack.push((child, path));
                } else if name.ends_with(".md") {
                    out.push(child);
                }
            }
        }
    }
    out.sort();
    out
}

/// Read the dok at farm-relative `rel`; `Ok(None)` when there is no such
/// dok file.
pub fn load(root: &Path, rel: &str) -> Result<Option<Dok>> {
    let rel = normalize(rel);
    let path = root.join(&rel);
    if !path.is_file() || !is_dok_path(&rel) {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(Some(Dok::parse(&rel, &text)))
}

/// A dok file's content hash (yaks-252c): the first 12 hex digits of the
/// sha256 of its bytes. `dok show` reports it and `dok set --if-hash` checks
/// it, so an edit built from a stale read is refused, not written over a
/// newer body.
pub fn text_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Like [`load`], with the [`text_hash`] of the file as it is on disk.
pub fn load_hashed(root: &Path, rel: &str) -> Result<Option<(Dok, String)>> {
    let rel = normalize(rel);
    let path = root.join(&rel);
    if !path.is_file() || !is_dok_path(&rel) {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(Some((Dok::parse(&rel, &text), text_hash(&text))))
}

/// Whether farm-relative `rel` names a dok file by its shape alone (the
/// rule [`dok_paths`] walks by): a `.md` file under `doks/` or
/// `<herd>/doks/`, with no empty, `..` or dot segment. Unlike
/// [`path_problem`] it accepts any file name, as existing doks may predate
/// the naming rule.
pub fn is_dok_path(rel: &str) -> bool {
    let under_root = rel.starts_with("doks/")
        || rel.split_once('/').is_some_and(|(h, rest)| {
            !store::RESERVED_HERD_NAMES.contains(&h) && rest.starts_with("doks/")
        });
    rel.ends_with(".md")
        && under_root
        && !rel
            .split('/')
            .any(|p| p.starts_with('.') || p.is_empty() || p == "..")
}

/// Every dok, sorted by path.
pub fn load_all(root: &Path) -> Result<Vec<Dok>> {
    dok_paths(root)
        .into_iter()
        .map(|rel| {
            let path = root.join(&rel);
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            Ok(Dok::parse(&rel, &text))
        })
        .collect()
}

/// A path as given on the command line, made farm-relative: a leading `/`
/// (the farm-root link form) and `./` dropped, `\` turned into `/`.
pub fn normalize(rel: &str) -> String {
    let s = rel.replace('\\', "/");
    let s = s.trim_start_matches('/');
    s.strip_prefix("./").unwrap_or(s).to_string()
}

/// Write `dok` at its path (creating directories), atomically.
pub fn save(root: &Path, dok: &Dok) -> Result<PathBuf> {
    let path = root.join(&dok.path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    store::write::atomic_write(&path, &dok.render())?;
    Ok(path)
}

/// Build a new dok at `rel`: `type` (default `note`), `title` (default from
/// the file name), any other `fields`, then `body`. Refuses a bad path or an
/// existing file. Does not write.
pub fn new_dok(
    root: &Path,
    rel: &str,
    kind: Option<&str>,
    title: Option<&str>,
    fields: &[(String, Value)],
    body: &str,
) -> Result<Dok> {
    let rel = normalize(rel);
    if let Some(why) = path_problem(&rel) {
        bail!("{rel}: {why}");
    }
    if root.join(&rel).exists() {
        bail!("{rel} already exists; edit it with `yaks dok set` or your editor");
    }
    let mut f = Fields::default();
    f.set(
        "type",
        &Value::Plain(kind.unwrap_or(DEFAULT_KIND).to_string()),
    );
    let default_title = {
        let stem = rel.rsplit('/').next().unwrap_or(&rel);
        let stem = stem.strip_suffix(".md").unwrap_or(stem);
        let mut t = stem.replace('-', " ");
        if let Some(c) = t.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        t
    };
    f.set(
        "title",
        &Value::Str(title.map(String::from).unwrap_or(default_title)),
    );
    for (k, v) in fields {
        if *v != Value::Null {
            f.set(k, v);
        }
    }
    Ok(Dok {
        path: rel,
        fields: f,
        body: body.to_string(),
        has_frontmatter: true,
    })
}

/// One problem `doctor` finds with a dok.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DokIssue {
    pub path: String,
    pub message: String,
}

/// Integrity of every dok: a file without frontmatter, frontmatter without a
/// `type:` or a `title:`, a path that `new_dok` would refuse, and a
/// frontmatter key whose value doesn't parse.
pub fn check(root: &Path) -> Result<Vec<DokIssue>> {
    let mut out = Vec::new();
    for p in load_all(root)? {
        let mut push = |m: String| {
            out.push(DokIssue {
                path: p.path.clone(),
                message: m,
            })
        };
        if !p.has_frontmatter {
            push(
                "no frontmatter (start the file with a --- block holding type: and title:)".into(),
            );
            continue;
        }
        if p.str_field("type").is_none_or(|s| s.is_empty()) {
            push("no type: (use note when unsure)".into());
        }
        if p.str_field("title").is_none_or(|s| s.is_empty()) {
            push("no title:".into());
        }
        if let Some(why) = path_problem(&p.path) {
            push(format!("path: {why}"));
        }
        for (k, v) in p.fields.iter() {
            if v.is_none() {
                push(format!(
                    "{k}: value uses YAML yaks does not read (kept as written)"
                ));
            }
        }
    }
    Ok(out)
}

/// A `--where key=value` test against one dok: the values the field holds
/// (a scalar is one value, a list its items; `type` and `tags` read through
/// the dok's defaults) tested by [`value_matches`].
pub fn matches(dok: &Dok, key: &str, expr: &str) -> bool {
    let values: Vec<String> = match key {
        "type" => vec![dok.kind()],
        "tags" | "tag" => dok.tags(),
        _ => match dok.fields.get(key) {
            Some(Value::List(items)) => items
                .iter()
                .filter_map(|i| i.as_str().map(String::from))
                .collect(),
            Some(v) => v.as_str().map(String::from).into_iter().collect(),
            None => Vec::new(),
        },
    };
    value_matches(&values, expr)
}

/// One `key=<expr>` test against the values a field holds (none when it is
/// missing). `expr` is a plain value (exact match), or starts with an
/// operator:
///
/// - `~text`: contains `text`, ignoring case;
/// - `<x`, `<=x`, `>x`, `>=x`: compares, numerically when both sides are
///   numbers, otherwise as text (so `YYYY-MM-DD` dates order correctly);
/// - `!x`: does not equal `x` (a missing field matches; a list matches when
///   it holds no `x`);
/// - empty: the field is missing or empty.
///
/// Every other form matches when any one value does.
pub fn value_matches(values: &[String], expr: &str) -> bool {
    use std::cmp::Ordering;
    let any = |f: &dyn Fn(&str) -> bool| values.iter().any(|v| f(v));
    if expr.is_empty() {
        return values.iter().all(|v| v.trim().is_empty());
    }
    if let Some(x) = expr.strip_prefix('!') {
        return !any(&|v| v == x);
    }
    if let Some(x) = expr.strip_prefix('~') {
        let x = x.to_lowercase();
        return any(&|v| v.to_lowercase().contains(&x));
    }
    let compare = |x: &str, ok: fn(Ordering) -> bool| {
        any(&|v| {
            let ord = match (v.trim().parse::<f64>(), x.trim().parse::<f64>()) {
                (Ok(a), Ok(b)) => a.partial_cmp(&b),
                _ => Some(v.cmp(x)),
            };
            ord.is_some_and(ok)
        })
    };
    if let Some(x) = expr.strip_prefix("<=") {
        return compare(x, Ordering::is_le);
    }
    if let Some(x) = expr.strip_prefix(">=") {
        return compare(x, Ordering::is_ge);
    }
    if let Some(x) = expr.strip_prefix('<') {
        return compare(x, Ordering::is_lt);
    }
    if let Some(x) = expr.strip_prefix('>') {
        return compare(x, Ordering::is_gt);
    }
    any(&|v| v == expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vals(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_plain_value_is_an_exact_match_on_any_item() {
        assert!(value_matches(&vals(&["a", "b"]), "b"));
        assert!(!value_matches(&vals(&["ab"]), "b"));
        assert!(!value_matches(&[], "b"));
    }

    #[test]
    fn contains_ignores_case_and_reads_any_item() {
        assert!(value_matches(&vals(&["Heat Pump"]), "~heat"));
        assert!(value_matches(&vals(&["x", "Pump"]), "~PUMP"));
        assert!(!value_matches(&vals(&["x"]), "~pump"));
        assert!(!value_matches(&[], "~pump"));
    }

    #[test]
    fn compare_is_numeric_for_numbers_and_textual_for_dates() {
        assert!(value_matches(&vals(&["9"]), "<10"), "9 < 10 numerically");
        assert!(!value_matches(&vals(&["9"]), ">10"));
        assert!(value_matches(&vals(&["10.5"]), ">=10.5"));
        assert!(value_matches(&vals(&["10.5"]), "<=10.50"));
        assert!(value_matches(&vals(&["2026-11-30"]), "<2026-12-01"));
        assert!(!value_matches(&vals(&["2026-12-01"]), "<2026-12-01"));
        assert!(value_matches(&vals(&["2026-12-01"]), "<=2026-12-01"));
        assert!(value_matches(&vals(&["2027-01-02"]), ">2026-12-31"));
        assert!(value_matches(&vals(&["a", "2027-01-02"]), ">2026-12-31"));
        assert!(!value_matches(&[], ">2026-12-31"), "missing never compares");
    }

    #[test]
    fn not_equal_matches_a_missing_field_and_a_list_without_the_value() {
        assert!(value_matches(&vals(&["no"]), "!yes"));
        assert!(!value_matches(&vals(&["yes"]), "!yes"));
        assert!(value_matches(&[], "!yes"));
        assert!(!value_matches(&vals(&["a", "yes"]), "!yes"));
    }

    #[test]
    fn empty_means_missing_or_empty() {
        assert!(value_matches(&[], ""));
        assert!(value_matches(&vals(&[""]), ""));
        assert!(value_matches(&vals(&[" "]), ""));
        assert!(!value_matches(&vals(&["x"]), ""));
        assert!(!value_matches(&vals(&["", "x"]), ""));
    }

    #[test]
    fn dok_fields_read_through_the_same_tests() {
        let d = Dok::parse(
            "doks/a.md",
            "---\ntype: appliance\ntitle: A\nrenews: 2026-11-30\ntags: [home, hvac]\n---\n",
        );
        assert!(matches(&d, "renews", "<2026-12-01"));
        assert!(matches(&d, "type", "~appl"));
        assert!(matches(&d, "tags", "!old"));
        assert!(matches(&d, "reviewed", ""));
        assert!(!matches(&d, "renews", ""));
        assert!(matches(&d, "title", "A"));
    }

    fn farm(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("yaks-doks-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn paths_must_sit_under_a_doks_root_in_kebab_case() {
        assert_eq!(path_problem("doks/topics/vector-search.md"), None);
        assert_eq!(path_problem("yaks/doks/design/links.md"), None);
        assert_eq!(path_problem("doks/a.md"), None);
        assert!(path_problem("notes/x.md").is_some());
        assert!(path_problem("doks/x.txt").is_some());
        assert!(path_problem("doks/Topics/x.md").is_some());
        assert!(path_problem("doks/x y.md").is_some());
        assert!(path_problem("doks/").is_some());
        assert!(path_problem("sources/doks/x.md").is_some());
    }

    #[test]
    fn discovery_finds_farm_and_herd_doks_and_skips_the_rest() {
        let root = farm("disc");
        for (p, t) in [
            (
                "doks/topics/a.md",
                "---\ntype: topic\ntitle: A\n---\n\nBody.\n",
            ),
            ("doks/b.md", "no frontmatter\n"),
            ("doks/img.png", "x"),
            ("doks/.hidden.md", "x"),
            ("yaks/doks/c.md", "---\ntitle: C\n---\n"),
            ("yaks/yaks/yaks-1a2b.md", "---\nid: yaks-1a2b\n---\n"),
            ("sources/doks/x.md", "x"),
        ] {
            let f = root.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, t).unwrap();
        }
        assert_eq!(
            dok_paths(&root),
            ["doks/b.md", "doks/topics/a.md", "yaks/doks/c.md"]
        );
        let all = load_all(&root).unwrap();
        assert_eq!(all[1].kind(), "topic");
        assert_eq!(all[2].kind(), "note", "no type: reads as note");
        assert_eq!(all[2].herd(), Some("yaks"));
        assert_eq!(all[1].herd(), None);
        let issues = check(&root).unwrap();
        let msgs: Vec<String> = issues
            .iter()
            .map(|i| format!("{}: {}", i.path, i.message))
            .collect();
        assert_eq!(
            msgs,
            [
                "doks/b.md: no frontmatter (start the file with a --- block holding type: and title:)",
                "yaks/doks/c.md: no type: (use note when unsure)",
            ]
        );
        assert!(
            load(&root, "/doks/topics/a.md").unwrap().is_some(),
            "leading / is the farm root"
        );
        assert!(
            load(&root, "yaks/yaks/yaks-1a2b.md").unwrap().is_none(),
            "a yak is not a dok"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn new_dok_fills_type_and_title_and_refuses_existing_files() {
        let root = farm("new");
        let p = new_dok(
            &root,
            "doks/topics/heat-pump.md",
            None,
            None,
            &[("tags".into(), Value::from_cli("[home, hvac]"))],
            "Notes.",
        )
        .unwrap();
        assert_eq!(
            p.render(),
            "---\ntype: note\ntitle: Heat pump\ntags:\n- home\n- hvac\n---\n\nNotes.\n"
        );
        save(&root, &p).unwrap();
        assert!(new_dok(&root, "doks/topics/heat-pump.md", None, None, &[], "").is_err());
        assert!(new_dok(&root, "doks/Bad.md", None, None, &[], "").is_err());
        let back = load(&root, "doks/topics/heat-pump.md").unwrap().unwrap();
        assert_eq!(back.render(), p.render());
        assert_eq!(back.tags(), ["home", "hvac"]);
        assert!(
            matches(&back, "tags", "hvac")
                && matches(&back, "type", "note")
                && !matches(&back, "title", "x")
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_untouched_dok_round_trips_byte_for_byte() {
        let text = "---\ntype: topic\ntitle: Form signals\ndescription: Signals about forms.\ntags: [subtext, signals, forms]\nsources:\n  - raw/sources/a/source.md\n---\n\nPart of [x](x.md).\n";
        assert_eq!(Dok::parse("doks/a.md", text).render(), text);
    }
}
