//! The link grammar (yaks-101d, design yaks-ccb1): what a markdown link in a
//! yak or a page points at, how it resolves, and how to read just the part it
//! names (`yaks peek`).
//!
//! A link is `<target>[@<rev>][#<anchor>]`:
//!
//! | Target | Form |
//! |---|---|
//! | a farm document | `/wiki/topics/x.md`: from the farm root |
//! | below the linking page | `setup/notes.md`: relative, downward only |
//! | a yak's attachment | `yaks-1a2b/log.txt`, from anywhere |
//! | the code repo | `code:src/foo.rs` |
//! | a URL | `https://...` |
//!
//! `@<rev>` (7-40 hex digits) pins a path to a commit: a farm path to a
//! farm-branch commit, a `code:` path to a code commit. `#L12` / `#L12-L40`
//! name lines; any other `#anchor` a heading, by GitHub's slug rule.
//!
//! Yaks keep two older code forms valid forever: `../src/foo.rs` and the
//! pinned `../blob/<sha>/src/foo.rs` (yaks-da5e). Anywhere else a `../` link
//! climbs out of its directory, which the grammar refuses: [`Link::Upward`],
//! which `doctor` reports.
//!
//! Parsing is pure ([`parse`]). Resolution ([`Resolved`]) and reading
//! ([`peek`]) touch the disk and git.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where in a target a link points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Anchor {
    /// 1-based, inclusive.
    Lines(u32, u32),
    /// A heading's slug (GitHub's rule, see [`slug`]).
    Heading(String),
}

/// What a link points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    Url(String),
    /// A farm document by its farm-relative path.
    Farm {
        path: String,
        rev: Option<String>,
        anchor: Option<Anchor>,
    },
    /// A yak's attachment, `<id>/<name>`.
    Attachment {
        id: String,
        name: String,
        anchor: Option<Anchor>,
    },
    /// A file in the code repository, by its path from the repo root.
    Code {
        path: String,
        rev: Option<String>,
        anchor: Option<Anchor>,
    },
    /// A `../` link that is not one of the legacy code forms: it climbs out
    /// of its directory, which the grammar refuses (link from the farm root
    /// instead). Holds the link as written.
    Upward(String),
    /// An in-document anchor (`#section`).
    Local(Anchor),
    /// A yak, by its id (`[yaks-1a2b](yaks-1a2b)`): absolute from anywhere,
    /// never relative to the linking page (yaks-0e15).
    Yak(String),
}

/// The document a link sits in, which decides what a relative link means.
#[derive(Debug, Clone, Copy)]
pub enum From<'a> {
    /// A yak: legacy `../` code links are valid; a relative link is
    /// relative to the farm root (an attachment `<id>/<file>`).
    Yak,
    /// A page at this farm-relative path: a relative link is relative to its
    /// directory, downward only.
    Page(&'a str),
    /// Typed on the command line: relative means from the farm root.
    Cli,
}

/// Classify a link destination written in `from`. `is_yak` says whether a
/// token is a yak id (attachments are `<id>/<name>`).
pub fn parse(dest: &str, from: From, is_yak: impl Fn(&str) -> bool) -> Link {
    let dest = dest.trim();
    if dest.contains("://") || dest.starts_with("mailto:") {
        return Link::Url(dest.to_string());
    }
    let (body, frag) = match dest.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (dest, None),
    };
    let anchor = frag.and_then(parse_anchor);
    if body.is_empty() {
        return match anchor {
            Some(a) => Link::Local(a),
            None => Link::Upward(dest.to_string()),
        };
    }
    let body = percent_decode(body);
    if let Some(rest) = body.strip_prefix("code:") {
        let (path, rev) = split_rev(rest);
        return match clean(path.trim_start_matches('/')) {
            Some(path) => Link::Code { path, rev, anchor },
            None => Link::Upward(dest.to_string()),
        };
    }
    if let Some(rest) = body.strip_prefix("../") {
        if matches!(from, From::Yak) {
            // Legacy code links: `../path` and `../blob/<sha>/path`.
            let (rev, path) = match rest.strip_prefix("blob/").and_then(|r| r.split_once('/')) {
                Some((sha, path)) if is_rev(sha) => (Some(sha.to_string()), path),
                _ => (None, rest),
            };
            if let Some(path) = clean(path) {
                return Link::Code { path, rev, anchor };
            }
        }
        return Link::Upward(dest.to_string());
    }
    let (path, rev) = split_rev(&body);
    if rev.is_none() && !path.contains('/') && is_yak(path) {
        return Link::Yak(path.to_string());
    }
    if let Some(rest) = path.strip_prefix('/') {
        return match clean(rest) {
            Some(path) => Link::Farm { path, rev, anchor },
            None => Link::Upward(dest.to_string()),
        };
    }
    if let Some((id, name)) = path.split_once('/') {
        if is_yak(id) && !name.is_empty() && !name.contains('/') {
            return Link::Attachment {
                id: id.to_string(),
                name: name.to_string(),
                anchor,
            };
        }
    }
    let base = match from {
        From::Page(p) => p.rsplit_once('/').map_or("", |(d, _)| d),
        From::Yak | From::Cli => "",
    };
    let joined = if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}/{path}")
    };
    match clean(&joined) {
        Some(path) => Link::Farm { path, rev, anchor },
        None => Link::Upward(dest.to_string()),
    }
}

/// `path@<rev>` -> `(path, Some(rev))` when what follows the last `@` is 7-40
/// hex digits; anything else is all path.
fn split_rev(s: &str) -> (&str, Option<String>) {
    match s.rsplit_once('@') {
        Some((p, r)) if is_rev(r) && !p.is_empty() => (p, Some(r.to_string())),
        _ => (s, None),
    }
}

fn is_rev(s: &str) -> bool {
    (7..=40).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `L12`, `L12-L40` (reordered if reversed), else a heading slug. Empty is none.
pub fn parse_anchor(frag: &str) -> Option<Anchor> {
    if frag.is_empty() {
        return None;
    }
    let lines = frag.strip_prefix('L').and_then(|r| {
        let (a, b) = r.split_once("-L").unwrap_or((r, r));
        let (a, b): (u32, u32) = (a.parse().ok()?, b.parse().ok()?);
        (a > 0 && b > 0).then(|| (a.min(b), a.max(b)))
    });
    Some(match lines {
        Some((a, b)) => Anchor::Lines(a, b),
        None => Anchor::Heading(frag.to_lowercase()),
    })
}

/// Drop `.` and empty segments; `None` for `..`, a trailing `/` or nothing.
fn clean(p: &str) -> Option<String> {
    if p.ends_with('/') {
        return None;
    }
    let mut parts = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => return None,
            s => parts.push(s),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Percent-encode what a markdown link destination cannot hold bare in a
/// path: spaces, `%`, `(`, `)`, `<`, `>` and control characters. Everything
/// else (including `/` and non-ASCII) is kept, so a plain name is unchanged.
/// The inverse of [`percent_decode`] for these.
pub fn percent_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == ' ' || c == '%' || c == '(' || c == ')' || c == '<' || c == '>' || c.is_control() {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Decode `%XX` escapes (a link destination as written) back to the text.
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// GitHub's heading slug: lowercase; letters, digits, `-` and `_` kept;
/// spaces become `-`; everything else dropped.
pub fn slug(heading: &str) -> String {
    heading
        .trim()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

/// The link as one short label: `wiki/x.md#setup`, `code:src/a.rs:L3-L9@abc1234`.
pub fn label(link: &Link) -> String {
    let anchor = |a: &Option<Anchor>| match a {
        Some(Anchor::Lines(x, y)) if x == y => format!(":L{x}"),
        Some(Anchor::Lines(x, y)) => format!(":L{x}-L{y}"),
        Some(Anchor::Heading(h)) => format!("#{h}"),
        None => String::new(),
    };
    let rev = |r: &Option<String>| {
        r.as_ref()
            .map(|r| format!("@{}", &r[..r.len().min(7)]))
            .unwrap_or_default()
    };
    match link {
        Link::Url(u) => u.clone(),
        Link::Farm {
            path,
            rev: r,
            anchor: a,
        } => format!("{path}{}{}", anchor(a), rev(r)),
        Link::Code {
            path,
            rev: r,
            anchor: a,
        } => format!("code:{path}{}{}", anchor(a), rev(r)),
        Link::Attachment {
            id,
            name,
            anchor: a,
        } => format!("{id}/{name}{}", anchor(a)),
        Link::Upward(s) => s.clone(),
        Link::Local(a) => anchor(&Some(a.clone())),
        Link::Yak(id) => id.clone(),
    }
}

/// One link found in a document's markdown: its destination as written, the
/// 1-based line it sits on, and whether it is an image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub dest: String,
    pub line: usize,
    pub image: bool,
}

/// Every markdown link and image in `text`, in order (code spans and blocks
/// are not links).
pub fn extract(text: &str) -> Vec<Found> {
    use pulldown_cmark::{Event, Parser, Tag};
    let starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_of = |off: usize| starts.partition_point(|&s| s <= off);
    Parser::new_ext(text, crate::markdown_lint::options())
        .into_offset_iter()
        .filter_map(|(e, r)| match e {
            Event::Start(Tag::Link { dest_url, .. }) => Some(Found {
                dest: dest_url.to_string(),
                line: line_of(r.start),
                image: false,
            }),
            Event::Start(Tag::Image { dest_url, .. }) => Some(Found {
                dest: dest_url.to_string(),
                line: line_of(r.start),
                image: true,
            }),
            _ => None,
        })
        .collect()
}

/// Rewrite link destinations in markdown `text`: `replace(dest)` returns the
/// new destination for a link to change, `None` to leave it. Only the
/// destination's bytes change (inline `[x](dest)` and `![x](dest)` links,
/// optionally with a title); code spans and blocks are untouched. Returns the
/// new text and how many links changed.
pub fn rewrite(text: &str, replace: impl Fn(&str) -> Option<String>) -> (String, usize) {
    use pulldown_cmark::{Event, LinkType, Parser, Tag};
    let mut edits: Vec<(std::ops::Range<usize>, String)> = Vec::new();
    for (e, r) in Parser::new_ext(text, crate::markdown_lint::options()).into_offset_iter() {
        let (dest, inline) = match &e {
            Event::Start(Tag::Link {
                dest_url,
                link_type,
                ..
            })
            | Event::Start(Tag::Image {
                dest_url,
                link_type,
                ..
            }) => (dest_url.to_string(), *link_type == LinkType::Inline),
            _ => continue,
        };
        if !inline {
            continue;
        }
        let Some(new) = replace(&dest) else { continue };
        // The destination sits inside the last `(...)` of the link's source.
        let src = &text[r.clone()];
        let Some(open) = src.rfind("](") else {
            continue;
        };
        let inner_start = r.start + open + 2;
        let Some(pos) = text[inner_start..r.end].find(dest.as_str()) else {
            continue;
        };
        let start = inner_start + pos;
        edits.push((start..start + dest.len(), new));
    }
    let n = edits.len();
    let mut out = text.to_string();
    for (range, new) in edits.into_iter().rev() {
        out.replace_range(range, &new);
    }
    (out, n)
}

/// The headings of a markdown document: `(level, slug, byte range of the
/// section)`, a section running to the next heading of the same or a higher
/// level. Duplicate slugs get `-1`, `-2` (GitHub's rule).
pub fn sections(text: &str) -> Vec<(u32, String, std::ops::Range<usize>)> {
    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
    let mut heads: Vec<(u32, String, usize)> = Vec::new();
    let mut current: Option<(u32, usize, String)> = None;
    for (e, r) in Parser::new_ext(text, crate::markdown_lint::options()).into_offset_iter() {
        match e {
            Event::Start(Tag::Heading { level, .. }) => {
                let n = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current = Some((n, r.start, String::new()));
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, _, s)) = current.as_mut() {
                    s.push_str(&t);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((n, start, s)) = current.take() {
                    heads.push((n, s, start));
                }
            }
            _ => {}
        }
    }
    let mut seen: std::collections::HashMap<String, usize> = Default::default();
    let mut out = Vec::new();
    for (i, (level, text_, start)) in heads.iter().enumerate() {
        let base = slug(text_);
        let n = seen.entry(base.clone()).or_insert(0);
        let s = if *n == 0 {
            base.clone()
        } else {
            format!("{base}-{n}")
        };
        *n += 1;
        let end = heads[i + 1..]
            .iter()
            .find(|(l, _, _)| l <= level)
            .map_or(text.len(), |(_, _, s)| *s);
        out.push((*level, s, *start..end));
    }
    out
}

// ------------------------------------------------------------- the farm graph

/// One link from a farm document: who links (`from`, a yak id or a page
/// path), on which line of that document, to what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub line: usize,
    pub dest: String,
    pub link: Link,
}

/// Every markdown link in every yak (description and notes) and page of the
/// farm at `root`, in document order. Yak ids are links too (mentions), but
/// those are `refs`' business; this is the markdown-link half.
pub fn edges(root: &Path) -> Result<Vec<Edge>> {
    let tasks = crate::store::load(
        root,
        &[
            crate::model::Status::Hairy,
            crate::model::Status::Shaving,
            crate::model::Status::Shorn,
            crate::model::Status::Dead,
        ],
    )?;
    let ids: std::collections::HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
    let is_yak = |s: &str| ids.contains(s);
    let mut out = Vec::new();
    for t in &tasks {
        for f in extract(&t.body) {
            out.push(Edge {
                from: t.id.clone(),
                line: f.line,
                link: parse(&f.dest, From::Yak, is_yak),
                dest: f.dest,
            });
        }
    }
    for p in crate::pages::load_all(root)? {
        for f in extract(&p.body) {
            out.push(Edge {
                from: p.path.clone(),
                line: f.line,
                link: parse(&f.dest, From::Page(&p.path), is_yak),
                dest: f.dest,
            });
        }
    }
    Ok(out)
}

/// Whether `link` names something that exists in the farm at `root` (its
/// current file; a pinned link is checked only for its path, to stay cheap).
/// URLs, code links and in-document anchors are not checked here: `None`.
pub fn exists(root: &Path, link: &Link) -> Option<bool> {
    // Placeholder syntax in prose (`[x]({id}/{name})`, `[a](path)`) is an
    // example, not a link: only a destination naming a file (with an
    // extension) is checked.
    let looks_like_file = |p: &str| {
        !p.contains(['{', '}', '<', '>']) && p.rsplit('/').next().is_some_and(|f| f.contains('.'))
    };
    match link {
        Link::Farm { path, .. } | Link::Attachment { name: path, .. } if !looks_like_file(path) => {
            None
        }
        Link::Farm { path, .. } => Some(root.join(path).exists()),
        Link::Attachment { id, name, .. } => {
            Some(crate::farm::locate_attachment(root, id, name).is_some())
        }
        Link::Upward(_) => Some(false),
        Link::Url(_) | Link::Code { .. } | Link::Local(_) | Link::Yak(_) => None,
    }
}

/// The farm-relative path a link names, for matching incoming links to a
/// document (`None` for anything that is not a farm file).
pub fn farm_path(link: &Link) -> Option<String> {
    match link {
        Link::Farm { path, .. } => Some(path.clone()),
        _ => None,
    }
}

// ------------------------------------------------------------------ reading

/// The text a link names, for `peek`: a header line saying what it is, then
/// the span (the lines, the section, or a short summary of the document).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peek {
    pub label: String,
    pub text: String,
}

/// Where to read from: the farm root and, for `code:` links, the code
/// checkout (`None` when there is none).
pub struct Bases<'a> {
    pub farm: &'a Path,
    pub code: Option<&'a Path>,
}

fn git(dir: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

/// Read `path` under `base`, or as it was at `rev` in the git repository
/// holding `base` (`git show <rev>:./<path>`, run in `base`).
pub fn read_at(base: &Path, path: &str, rev: Option<&str>) -> Result<String> {
    let Some(rev) = rev else {
        let file = base.join(path);
        return match std::fs::read(&file) {
            Ok(b) => Ok(String::from_utf8(b)
                .unwrap_or_else(|e| format!("(binary file, {} bytes)\n", e.into_bytes().len()))),
            Err(_) => bail!("{path}: no such file"),
        };
    };
    let out = git(base, &["show", &format!("{rev}:./{path}")])?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!(
            "{path}@{}: {}",
            &rev[..rev.len().min(7)],
            err.lines().next().unwrap_or("not found")
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Lines `a..=b` of `text`, widened by `context` either side, each prefixed
/// with its number.
fn lines(text: &str, a: u32, b: u32, context: u32) -> Result<String> {
    let all: Vec<&str> = text.lines().collect();
    if a as usize > all.len() {
        bail!("line {a} is past the end ({} lines)", all.len());
    }
    let lo = a.saturating_sub(context).max(1) as usize;
    let hi = ((b + context) as usize).min(all.len());
    let w = hi.to_string().len();
    let mut out = String::new();
    for (i, l) in all.iter().enumerate().take(hi).skip(lo - 1) {
        out.push_str(&format!("{:>w$}  {l}\n", i + 1));
    }
    Ok(out)
}

/// The section of markdown `text` under the heading slugged `anchor`.
fn section(text: &str, anchor: &str) -> Result<String> {
    match sections(text).into_iter().find(|(_, s, _)| s == anchor) {
        Some((_, _, r)) => Ok(text[r].trim_end().to_string() + "\n"),
        None => {
            let have: Vec<String> = sections(text).into_iter().map(|(_, s, _)| s).collect();
            bail!(
                "no heading #{anchor} (headings: {})",
                if have.is_empty() {
                    "none".into()
                } else {
                    have.join(", ")
                }
            )
        }
    }
}

/// A document with no anchor: frontmatter and the first paragraph of the
/// body (a markdown file), else the first 20 lines.
fn summary(path: &str, text: &str) -> String {
    if !path.ends_with(".md") {
        let first: Vec<&str> = text.lines().take(20).collect();
        let more = text.lines().count().saturating_sub(20);
        let mut s = first.join("\n") + "\n";
        if more > 0 {
            s.push_str(&format!("... ({more} more lines)\n"));
        }
        return s;
    }
    let (front, body) = match crate::store::split_frontmatter(text) {
        Some((f, b)) => (Some(f), b),
        None => (None, text),
    };
    let para: Vec<&str> = body
        .trim_start()
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .collect();
    let mut s = String::new();
    if let Some(f) = front {
        s.push_str("---\n");
        s.push_str(f);
        s.push_str("\n---\n\n");
    }
    s.push_str(&para.join("\n"));
    s.push('\n');
    s
}

fn span(path: &str, text: &str, anchor: &Option<Anchor>, context: u32) -> Result<String> {
    match anchor {
        Some(Anchor::Lines(a, b)) => lines(text, *a, *b, context),
        Some(Anchor::Heading(h)) => section(text, h),
        None => Ok(summary(path, text)),
    }
}

/// Read what `link` names. `context` widens a line range.
pub fn peek(link: &Link, bases: &Bases, context: u32) -> Result<Peek> {
    let label = label(link);
    let text = match link {
        Link::Url(u) => bail!("{u}: a URL; open it in a browser"),
        Link::Upward(s) => {
            bail!("{s}: a link may not climb out with ../; link from the farm root (/wiki/...)")
        }
        Link::Local(_) => bail!("an in-document anchor needs its document"),
        Link::Yak(id) => {
            let path = crate::store::task_path(bases.farm, id);
            let text = std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("{id}: {e}"))?;
            summary("yak.md", &text)
        }
        Link::Farm { path, rev, anchor } => {
            let text = match rev {
                // A farm path at a commit: the farm root is `.yaks/` in its
                // worktree, so read through git from there.
                Some(r) => read_at(bases.farm, path, Some(r))?,
                None => read_at(bases.farm, path, None)?,
            };
            span(path, &text, anchor, context)?
        }
        Link::Attachment { id, name, anchor } => {
            let Some(found) = crate::farm::locate_attachment(bases.farm, id, name) else {
                bail!("{id}/{name}: no such attachment");
            };
            let text = read_at(found.path.parent().unwrap_or(bases.farm), name, None)?;
            span(name, &text, anchor, context)?
        }
        Link::Code { path, rev, anchor } => {
            let Some(code) = bases.code else {
                bail!("code:{path}: no code checkout to read it from (run from one)");
            };
            let text = read_at(code, path, rev.as_deref())?;
            span(path, &text, anchor, context)?
        }
    };
    Ok(Peek { label, text })
}

/// The checkout `code:` links (and legacy `../` ones) resolve in. For a farm
/// reached through `YAKS_DIR` that is the farm's parent. A farm branch lives
/// in a worktree holding only `.yaks/`, so there it is the code checkout: the
/// git top-level of `cwd` unless that is the farm worktree itself, else the
/// repository's main checkout (yaks-6c4f). `None` when nothing fits.
pub fn code_base(farm_root: &Path, cwd: Option<&Path>) -> Option<PathBuf> {
    let own = farm_root.parent().map(Path::to_path_buf);
    if crate::store::farm_branch(farm_root).is_none() {
        return own;
    }
    let farm_wt = own.as_deref().map(crate::gitrepo::canonical);
    let top = |dir: &Path| {
        git(dir, &["rev-parse", "--show-toplevel"])
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                crate::gitrepo::canonical(Path::new(String::from_utf8_lossy(&o.stdout).trim()))
            })
    };
    if let Some(t) = cwd.and_then(top).filter(|t| Some(t) != farm_wt.as_ref()) {
        return Some(t);
    }
    crate::gitrepo::host(farm_root)
        .and_then(|h| crate::gitrepo::host_checkout(&h))
        .filter(|t| Some(t) != farm_wt.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yak(s: &str) -> bool {
        s.starts_with("yaks-")
    }

    fn farm(path: &str, rev: Option<&str>, anchor: Option<Anchor>) -> Link {
        Link::Farm {
            path: path.into(),
            rev: rev.map(String::from),
            anchor,
        }
    }

    #[test]
    fn the_grammar() {
        let pg = From::Page("wiki/topics/x.md");
        assert_eq!(
            parse("https://a.test/b#c", pg, yak),
            Link::Url("https://a.test/b#c".into())
        );
        assert_eq!(parse("/wiki/y.md", pg, yak), farm("wiki/y.md", None, None));
        assert_eq!(
            parse("/wiki/y.md@91be0d2#set-up", pg, yak),
            farm(
                "wiki/y.md",
                Some("91be0d2"),
                Some(Anchor::Heading("set-up".into()))
            )
        );
        assert_eq!(
            parse("sub/z.md", pg, yak),
            farm("wiki/topics/sub/z.md", None, None)
        );
        assert_eq!(parse("z.md", pg, yak), farm("wiki/topics/z.md", None, None));
        assert_eq!(parse("../y.md", pg, yak), Link::Upward("../y.md".into()));
        assert_eq!(
            parse("../../raw/a.png", pg, yak),
            Link::Upward("../../raw/a.png".into())
        );
        assert_eq!(
            parse("yaks-1a2b/log.txt#L3", pg, yak),
            Link::Attachment {
                id: "yaks-1a2b".into(),
                name: "log.txt".into(),
                anchor: Some(Anchor::Lines(3, 3))
            }
        );
        assert_eq!(
            parse("code:src/foo.rs@3f2a9c1#L40-L12", pg, yak),
            Link::Code {
                path: "src/foo.rs".into(),
                rev: Some("3f2a9c1".into()),
                anchor: Some(Anchor::Lines(12, 40))
            }
        );
        assert_eq!(
            parse("#Setup", pg, yak),
            Link::Local(Anchor::Heading("setup".into()))
        );
        // `@` that is not a sha is part of the path.
        assert_eq!(
            parse("/wiki/a@b.md", pg, yak),
            farm("wiki/a@b.md", None, None)
        );
        // A yak id is a yak from anywhere, never a page-relative file
        // (yaks-0e15); a non-yak bare word still is.
        for from in [pg, From::Yak, From::Cli] {
            assert_eq!(parse("yaks-44b3", from, yak), Link::Yak("yaks-44b3".into()));
        }
        assert_eq!(
            parse("notes", pg, yak),
            farm("wiki/topics/notes", None, None)
        );
    }

    #[test]
    fn legacy_code_links_are_valid_only_in_yaks() {
        assert_eq!(
            parse("../src/foo.rs#L12", From::Yak, yak),
            Link::Code {
                path: "src/foo.rs".into(),
                rev: None,
                anchor: Some(Anchor::Lines(12, 12))
            }
        );
        assert_eq!(
            parse("../blob/abc1234/src/foo.rs", From::Yak, yak),
            Link::Code {
                path: "src/foo.rs".into(),
                rev: Some("abc1234".into()),
                anchor: None
            }
        );
        assert_eq!(
            parse("../../etc/x", From::Yak, yak),
            Link::Upward("../../etc/x".into())
        );
        assert_eq!(
            parse("../src/foo.rs", From::Page("wiki/a.md"), yak),
            Link::Upward("../src/foo.rs".into())
        );
        assert_eq!(
            parse("yaks-1a2b/shot.png", From::Yak, yak),
            Link::Attachment {
                id: "yaks-1a2b".into(),
                name: "shot.png".into(),
                anchor: None
            }
        );
    }

    #[test]
    fn slugs_and_sections_follow_github() {
        assert_eq!(slug("Set up: the `yaks` CLI!"), "set-up-the-yaks-cli");
        assert_eq!(slug("Ünïcode words"), "ünïcode-words");
        let doc =
            "# Top\n\nintro\n\n## Setup\n\na\n\n### Deeper\n\nb\n\n## Setup\n\nc\n\n## Last\n";
        let s: Vec<(u32, String)> = sections(doc).into_iter().map(|(l, s, _)| (l, s)).collect();
        assert_eq!(
            s,
            [
                (1, "top".into()),
                (2, "setup".into()),
                (3, "deeper".into()),
                (2, "setup-1".into()),
                (2, "last".into())
            ]
        );
        assert_eq!(
            section(doc, "setup").unwrap(),
            "## Setup\n\na\n\n### Deeper\n\nb\n"
        );
        assert_eq!(section(doc, "setup-1").unwrap(), "## Setup\n\nc\n");
        assert!(
            section(doc, "nope")
                .unwrap_err()
                .to_string()
                .contains("headings: top, setup")
        );
    }

    #[test]
    fn extract_finds_links_with_lines_and_skips_code() {
        let text = "See [a](/wiki/a.md).\n\n![shot](yaks-1a2b/s.png)\n\n`[not](x.md)`\n\n```\n[no](y.md)\n```\n";
        assert_eq!(
            extract(text),
            [
                Found {
                    dest: "/wiki/a.md".into(),
                    line: 1,
                    image: false
                },
                Found {
                    dest: "yaks-1a2b/s.png".into(),
                    line: 3,
                    image: true
                },
            ]
        );
    }

    #[test]
    fn peek_reads_lines_sections_and_summaries() {
        let root = std::env::temp_dir().join(format!("yaks-links-peek-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("wiki")).unwrap();
        std::fs::write(root.join("wiki/a.md"), "---\ntype: topic\ntitle: A\n---\n\nFirst para\nstill.\n\nSecond.\n\n## Setup\n\nSteps.\n").unwrap();
        let b = Bases {
            farm: &root,
            code: Some(&root),
        };
        let p = peek(&parse("/wiki/a.md", From::Cli, yak), &b, 0).unwrap();
        assert_eq!(
            p.text,
            "---\ntype: topic\ntitle: A\n---\n\nFirst para\nstill.\n"
        );
        let p = peek(&parse("/wiki/a.md#setup", From::Cli, yak), &b, 0).unwrap();
        assert_eq!(p.text, "## Setup\n\nSteps.\n");
        let p = peek(&parse("code:wiki/a.md#L6-L7", From::Cli, yak), &b, 1).unwrap();
        assert_eq!(p.text, "5  \n6  First para\n7  still.\n8  \n");
        assert!(peek(&parse("/wiki/none.md", From::Cli, yak), &b, 0).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
