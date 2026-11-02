//! Pages (yaks-9af0, design yaks-aae6): markdown documents in a farm that are
//! not yaks. A page lives under a wiki root, the farm-wide `wiki/` or one
//! herd's `<herd>/wiki/`, at any depth, and is addressed by its path from the
//! farm root (`wiki/topics/vector-search.md`).
//!
//! A page is frontmatter plus a markdown body, in OKF's shape (`type`,
//! `title`, `description`, `tags`, `sources`, `status`, `stale_after`, and
//! anything else). Frontmatter is a [`Fields`], so every key round-trips as
//! written and an edit rewrites only the key it sets. Pages have no notes log
//! and no `updated:` field: git says when a page changed.
//!
//! Untyped kinds need no schema: `note` (the default), `topic`, `howto`,
//! `summary`. Any other `type` is a typed kind, described by
//! `schemas/<type>.md` (yaks-8e75).

use crate::fields::{Fields, Value};
use crate::store;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// The directory name of a wiki root, farm-wide (`wiki/`) or per herd
/// (`<herd>/wiki/`).
pub const WIKI_DIR: &str = "wiki";

/// The kinds of page that need no schema.
pub const UNTYPED_KINDS: &[&str] = &["note", "topic", "howto", "summary"];

/// The kind a page has when its frontmatter names none.
pub const DEFAULT_KIND: &str = "note";

/// One page: its farm-relative path, frontmatter and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// From the farm root, `/`-separated: `wiki/topics/x.md`.
    pub path: String,
    pub fields: Fields,
    pub body: String,
    /// False when the file has no `---` frontmatter block at all (doctor
    /// reports it; the whole text is then the body).
    pub has_frontmatter: bool,
}

impl Page {
    /// Parse a page file's text.
    pub fn parse(path: &str, text: &str) -> Page {
        match store::split_frontmatter(text) {
            Some((front, body)) => Page {
                path: path.to_string(),
                fields: Fields::parse(front),
                body: body.to_string(),
                has_frontmatter: true,
            },
            None => Page {
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

    /// The page's kind: its `type:`, else `note`.
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

    /// The herd whose wiki holds it (`None` for the farm-wide wiki).
    pub fn herd(&self) -> Option<&str> {
        let (first, rest) = self.path.split_once('/')?;
        (first != WIKI_DIR && rest.starts_with("wiki/")).then_some(first)
    }
}

/// Why `rel` (a farm-relative path) is not a valid place for a new page, or
/// `None` when it is: `wiki/<parts>.md` or `<herd>/wiki/<parts>.md`, every
/// part lowercase letters, digits and `-` (a kebab name), at least one part
/// below the wiki root, ending in `.md`.
pub fn path_problem(rel: &str) -> Option<String> {
    let parts: Vec<&str> = rel.split('/').collect();
    let below = match parts.as_slice() {
        [w, rest @ ..] if *w == WIKI_DIR => rest,
        [herd, w, rest @ ..] if *w == WIKI_DIR && !herd.starts_with('.') => {
            if store::RESERVED_HERD_NAMES.contains(herd) {
                return Some(format!("`{herd}` is not a herd"));
            }
            rest
        }
        _ => return Some("a page lives under wiki/ or <herd>/wiki/".to_string()),
    };
    let Some((file, dirs)) = below.split_last() else {
        return Some("name the page file below the wiki directory".to_string());
    };
    let Some(stem) = file.strip_suffix(".md") else {
        return Some("a page file ends in .md".to_string());
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

/// Every wiki root of the farm at `root` that exists: `wiki/` and each
/// `<herd>/wiki/`, as `(farm-relative dir, absolute dir)`.
fn wiki_roots(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let farm_wide = root.join(WIKI_DIR);
    if farm_wide.is_dir() {
        out.push((WIKI_DIR.to_string(), farm_wide));
    }
    if let Ok(rd) = std::fs::read_dir(root) {
        let mut herds: Vec<String> = rd
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(String::from))
            .filter(|n| !n.starts_with('.') && !store::RESERVED_HERD_NAMES.contains(&n.as_str()))
            .collect();
        herds.sort();
        for h in herds {
            let dir = root.join(&h).join(WIKI_DIR);
            if dir.is_dir() {
                out.push((format!("{h}/{WIKI_DIR}"), dir));
            }
        }
    }
    out
}

/// Every page file of the farm at `root`, as farm-relative paths, sorted. Dot
/// files and directories are skipped; only `.md` files are pages (anything
/// else under a wiki root, an image, is a file a page links to).
pub fn page_paths(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (rel, dir) in wiki_roots(root) {
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

/// Read the page at farm-relative `rel`; `Ok(None)` when there is no such
/// page file.
pub fn load(root: &Path, rel: &str) -> Result<Option<Page>> {
    let rel = normalize(rel);
    let path = root.join(&rel);
    if !path.is_file() || !is_page_path(&rel) {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(Some(Page::parse(&rel, &text)))
}

/// Whether farm-relative `rel` names a page file by its shape alone (the
/// rule [`page_paths`] walks by): a `.md` file under `wiki/` or
/// `<herd>/wiki/`, with no empty, `..` or dot segment. Unlike
/// [`path_problem`] it accepts any file name, as existing pages may predate
/// the naming rule.
pub fn is_page_path(rel: &str) -> bool {
    let under_root = rel.starts_with("wiki/")
        || rel.split_once('/').is_some_and(|(h, rest)| {
            !store::RESERVED_HERD_NAMES.contains(&h) && rest.starts_with("wiki/")
        });
    rel.ends_with(".md")
        && under_root
        && !rel
            .split('/')
            .any(|p| p.starts_with('.') || p.is_empty() || p == "..")
}

/// Every page, sorted by path.
pub fn load_all(root: &Path) -> Result<Vec<Page>> {
    page_paths(root)
        .into_iter()
        .map(|rel| {
            let path = root.join(&rel);
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            Ok(Page::parse(&rel, &text))
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

/// Write `page` at its path (creating directories), atomically.
pub fn save(root: &Path, page: &Page) -> Result<PathBuf> {
    let path = root.join(&page.path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    store::write::atomic_write(&path, &page.render())?;
    Ok(path)
}

/// Build a new page at `rel`: `type` (default `note`), `title` (default from
/// the file name), any other `fields`, then `body`. Refuses a bad path or an
/// existing file. Does not write.
pub fn new_page(
    root: &Path,
    rel: &str,
    kind: Option<&str>,
    title: Option<&str>,
    fields: &[(String, Value)],
    body: &str,
) -> Result<Page> {
    let rel = normalize(rel);
    if let Some(why) = path_problem(&rel) {
        bail!("{rel}: {why}");
    }
    if root.join(&rel).exists() {
        bail!("{rel} already exists; edit it with `yaks page set` or your editor");
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
    Ok(Page {
        path: rel,
        fields: f,
        body: body.to_string(),
        has_frontmatter: true,
    })
}

/// One problem `doctor` finds with a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageIssue {
    pub path: String,
    pub message: String,
}

/// Integrity of every page: a file without frontmatter, frontmatter without a
/// `type:` or a `title:`, a path that `new_page` would refuse, and a
/// frontmatter key whose value doesn't parse.
pub fn check(root: &Path) -> Result<Vec<PageIssue>> {
    let mut out = Vec::new();
    for p in load_all(root)? {
        let mut push = |m: String| {
            out.push(PageIssue {
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

/// A `--where key=value` test against one page: the field's scalar equals
/// `value`, or its list holds it. `type` and `tags` read through the page's
/// defaults.
pub fn matches(page: &Page, key: &str, value: &str) -> bool {
    match key {
        "type" => page.kind() == value,
        "tags" | "tag" => page.tags().iter().any(|t| t == value),
        _ => match page.fields.get(key) {
            Some(Value::List(items)) => items.iter().any(|i| i.as_str() == Some(value)),
            Some(v) => v.as_str() == Some(value),
            None => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn farm(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("yaks-pages-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn paths_must_sit_under_a_wiki_root_in_kebab_case() {
        assert_eq!(path_problem("wiki/topics/vector-search.md"), None);
        assert_eq!(path_problem("yaks/wiki/design/links.md"), None);
        assert_eq!(path_problem("wiki/a.md"), None);
        assert!(path_problem("notes/x.md").is_some());
        assert!(path_problem("wiki/x.txt").is_some());
        assert!(path_problem("wiki/Topics/x.md").is_some());
        assert!(path_problem("wiki/x y.md").is_some());
        assert!(path_problem("wiki/").is_some());
        assert!(path_problem("sources/wiki/x.md").is_some());
    }

    #[test]
    fn discovery_finds_farm_and_herd_wikis_and_skips_the_rest() {
        let root = farm("disc");
        for (p, t) in [
            (
                "wiki/topics/a.md",
                "---\ntype: topic\ntitle: A\n---\n\nBody.\n",
            ),
            ("wiki/b.md", "no frontmatter\n"),
            ("wiki/img.png", "x"),
            ("wiki/.hidden.md", "x"),
            ("yaks/wiki/c.md", "---\ntitle: C\n---\n"),
            ("yaks/yaks/yaks-1a2b.md", "---\nid: yaks-1a2b\n---\n"),
            ("sources/wiki/x.md", "x"),
        ] {
            let f = root.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, t).unwrap();
        }
        assert_eq!(
            page_paths(&root),
            ["wiki/b.md", "wiki/topics/a.md", "yaks/wiki/c.md"]
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
                "wiki/b.md: no frontmatter (start the file with a --- block holding type: and title:)",
                "yaks/wiki/c.md: no type: (use note when unsure)",
            ]
        );
        assert!(
            load(&root, "/wiki/topics/a.md").unwrap().is_some(),
            "leading / is the farm root"
        );
        assert!(
            load(&root, "yaks/yaks/yaks-1a2b.md").unwrap().is_none(),
            "a yak is not a page"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn new_page_fills_type_and_title_and_refuses_existing_files() {
        let root = farm("new");
        let p = new_page(
            &root,
            "wiki/topics/heat-pump.md",
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
        assert!(new_page(&root, "wiki/topics/heat-pump.md", None, None, &[], "").is_err());
        assert!(new_page(&root, "wiki/Bad.md", None, None, &[], "").is_err());
        let back = load(&root, "wiki/topics/heat-pump.md").unwrap().unwrap();
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
    fn an_untouched_page_round_trips_byte_for_byte() {
        let text = "---\ntype: topic\ntitle: Form signals\ndescription: Signals about forms.\ntags: [subtext, signals, forms]\nsources:\n  - raw/sources/a/source.md\n---\n\nPart of [x](x.md).\n";
        assert_eq!(Page::parse("wiki/a.md", text).render(), text);
    }
}
