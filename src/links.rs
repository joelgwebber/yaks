//! The link grammar (yaks-101d, design yaks-ccb1): what a markdown link in a
//! yak or a dok points at, how it resolves, and how to read just the part it
//! names (`yaks peek`).
//!
//! A link is `<target>[@<rev>][#<anchor>]`:
//!
//! | Target | Form |
//! |---|---|
//! | a farm document | `/doks/topics/x.md`: from the farm root |
//! | below the linking dok | `setup/notes.md`: relative, downward only |
//! | a yak's attachment | `yaks-1a2b/log.txt`, from anywhere; in an attachment, a bare sibling `log.txt` |
//! | the code repo | `code:src/foo.rs` |
//! | a URL | `https://...` |
//! | one note of a yak | `yaks-1a2b#2026-11-04T22:13:03Z`: the note's timestamp |
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
    /// A yak's attachment, `<id>/<name>`. `name` may have directories in it
    /// (`proj/src/main.rs`, a file of an attached directory) or name a
    /// directory itself (`proj`, written `<id>/proj/`); never `.`, `..` or an
    /// empty segment.
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
    /// never relative to the linking dok (yaks-0e15). `note` names one of its
    /// notes by timestamp (`yaks-1a2b#2026-11-04T22:13:03Z`, yaks-c5bd); see
    /// [`note_anchor`].
    Yak {
        id: String,
        note: Option<String>,
    },
}

/// The document a link sits in, which decides what a relative link means.
#[derive(Debug, Clone, Copy)]
pub enum From<'a> {
    /// A yak: legacy `../` code links are valid; a relative link is
    /// relative to the farm root (an attachment `<id>/<file>`).
    Yak,
    /// A file attached to yak `<id>`, which sits beside its siblings: a bare
    /// `name` is another attachment of the same yak (so an evidence
    /// document's `![x](shot.png)` resolves in any markdown viewer and here),
    /// and the `<id>/<file>` form still works. Anything else reads as in a
    /// yak (`/farm/path`, `code:`, URLs, the legacy `../` code forms; any
    /// other relative path reads from the farm root). An attachment directory
    /// is flat, so a sibling is always a bare file name.
    Attachment(&'a str),
    /// A dok at this farm-relative path: a relative link is relative to its
    /// directory, downward only.
    Dok(&'a str),
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
        if matches!(from, From::Yak | From::Attachment(_)) {
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
        return Link::Yak {
            id: path.to_string(),
            note: frag.filter(|f| is_note_anchor(f)).map(String::from),
        };
    }
    if let Some(rest) = path.strip_prefix('/') {
        // A yak by its file, `/<herd>/yaks/<id>.md[#note]`, and its
        // attachment, `/<herd>/yaks/<id>/<file>`: the form that works on a
        // markdown host (a bare id does not), so it names the same yak or
        // attachment as the bare id and the `<id>/<file>` form do. Only when
        // the id is a real yak whose herd is the directory's, so a lookalike
        // path stays a plain farm file.
        if rev.is_none() {
            if let Some((id, tail)) = yak_file_parts(rest) {
                if is_yak(id) {
                    return match tail {
                        None => Link::Yak {
                            id: id.to_string(),
                            note: frag.filter(|f| is_note_anchor(f)).map(String::from),
                        },
                        Some(name) => Link::Attachment {
                            id: id.to_string(),
                            name: name.to_string(),
                            anchor,
                        },
                    };
                }
            }
        }
        return match clean(rest) {
            Some(path) => Link::Farm { path, rev, anchor },
            None => Link::Upward(dest.to_string()),
        };
    }
    if let Some((id, name)) = path.split_once('/') {
        if is_yak(id) {
            if let Some(name) = clean_attachment(name) {
                return Link::Attachment {
                    id: id.to_string(),
                    name,
                    anchor,
                };
            }
        }
    }
    if let From::Attachment(id) = from {
        // A sibling: one plain file name, no directory part.
        if let Some(name) = clean(path).filter(|n| rev.is_none() && !n.contains('/')) {
            return Link::Attachment {
                id: id.to_string(),
                name,
                anchor,
            };
        }
    }
    let base = match from {
        From::Dok(p) => p.rsplit_once('/').map_or("", |(d, _)| d),
        From::Yak | From::Attachment(_) | From::Cli => "",
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

/// `<herd>/yaks/<id>.md` -> `(id, None)` and `<herd>/yaks/<id>/<file>` ->
/// `(id, Some(file))`, when the id's herd is the directory's (`store::herd_of`).
/// The shape of a yak's own file and attachments in the farm (a path from the
/// farm root, no leading `/`); `None` for any other path.
fn yak_file_parts(path: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = path.split('/');
    let (herd, dir, third) = (parts.next()?, parts.next()?, parts.next()?);
    if dir != crate::store::YAKS_SUBDIR || herd.is_empty() {
        return None;
    }
    let rest: Vec<&str> = parts.collect();
    let (id, tail) = match (third.strip_suffix(".md"), rest.as_slice()) {
        (Some(id), []) => (id, None),
        // An attachment, or a file or directory inside an attached directory.
        (None, [first, ..]) if !first.is_empty() => {
            let start = herd.len() + dir.len() + third.len() + 3;
            (third, Some(path.get(start..)?.trim_end_matches('/')))
        }
        _ => return None,
    };
    if tail.is_some_and(|t| clean_attachment(t).is_none()) {
        return None;
    }
    (crate::store::herd_of(id) == herd).then_some((id, tail))
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

// ------------------------------------------------------------ note anchors
//
// A note is named by its timestamp, which a note keeps for life: removing
// another note (`pickup --drop-request`) shifts indexes but never a
// timestamp. Notes written in the same second (a shear note and its move)
// share one, so the later ones take a `-1`, `-2` suffix in file order, as
// GitHub numbers duplicate heading slugs. The first note of a second is the
// plain timestamp.
//
// The anchor written follows the body's note shape: the slug of the heading,
// `2026-11-04t221303z`, for the new shape (`#### <ts>`, which GitHub gives
// that id; a link with a `:` before any `/` is dropped as a URL scheme, so the
// slug is the form that survives a markdown host), the timestamp itself,
// `2026-11-04T22:13:03Z`, for the old one, which an older yaks can follow.
// Both forms are accepted everywhere, so a link of either age resolves.

/// GitHub's slug of a note timestamp: lowercased, with everything but letters,
/// digits, `-` and `_` dropped (`2026-11-04T22:13:03Z` -> `2026-11-04t221303z`).
fn ts_slug(ts: &str) -> String {
    ts.chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether `s` has the shape of a slugged timestamp: `yyyy-mm-ddt` and then
/// at least the six digits of a time.
fn is_slug_ts(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 17
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[10] == b't'
        && b[11..17].iter().all(u8::is_ascii_digit)
        && b[17..].iter().all(|c| c.is_ascii_alphanumeric())
}

/// Whether a fragment names a note: an RFC 3339 timestamp or its slug,
/// optionally with a `-<n>` suffix for the n-th later note of the same second.
pub fn is_note_anchor(frag: &str) -> bool {
    let (ts, _) = split_note_anchor(frag);
    (ts.contains('T') && chrono::DateTime::parse_from_rfc3339(ts).is_ok()) || is_slug_ts(ts)
}

/// `2026-11-04T22:13:03Z-2` or `2026-11-04t221303z-2` -> (the timestamp or
/// slug, 2); no suffix is 0.
fn split_note_anchor(frag: &str) -> (&str, usize) {
    if let Some((ts, n)) = frag.rsplit_once('-')
        && !n.is_empty()
        && n.bytes().all(|b| b.is_ascii_digit())
        && (chrono::DateTime::parse_from_rfc3339(ts).is_ok() || is_slug_ts(ts))
    {
        return (ts, n.parse().unwrap_or(0));
    }
    (frag, 0)
}

/// A note anchor in its slug form, `2026-11-04t221303z` (`-<n>` kept): what a
/// markdown host gives a `#### <timestamp>` heading. Accepts either form;
/// `None` for a fragment that names no note. The migration writes links with it.
pub fn slug_note_anchor(frag: &str) -> Option<String> {
    if !is_note_anchor(frag) {
        return None;
    }
    let (ts, n) = split_note_anchor(frag);
    let slug = ts_slug(ts);
    Some(if n == 0 { slug } else { format!("{slug}-{n}") })
}

/// The anchor naming `notes[i]` in a body of note shape `shape`: the slug of
/// its timestamp (new shape) or the timestamp (old), suffixed `-<n>` when `n`
/// earlier notes share it.
pub fn note_anchor(
    shape: crate::store::NoteShape,
    notes: &[crate::store::NoteEntry],
    i: usize,
) -> String {
    let ts = &notes[i].ts;
    let n = notes[..i].iter().filter(|e| &e.ts == ts).count();
    let name = match shape {
        crate::store::NoteShape::New => ts_slug(ts),
        crate::store::NoteShape::Old => ts.clone(),
    };
    if n == 0 { name } else { format!("{name}-{n}") }
}

/// The index of the note an anchor names, if the yak has it, by either form.
pub fn find_note(notes: &[crate::store::NoteEntry], anchor: &str) -> Option<usize> {
    let (ts, n) = split_note_anchor(anchor);
    let want = ts_slug(ts);
    notes
        .iter()
        .enumerate()
        .filter(|(_, e)| ts_slug(&e.ts) == want)
        .nth(n)
        .map(|(i, _)| i)
}

/// The link to a yak's latest note, or `None` without notes. A body in the new
/// shape gets the form a markdown host follows, `/<herd>/yaks/<id>.md#<slug>`;
/// an old-shape body keeps `<id>#<timestamp>`, which an older yaks can follow.
pub fn latest_note_link(id: &str, body: &str) -> Option<String> {
    let notes = crate::store::parse_notes(body);
    let shape = crate::store::body_shape(body).unwrap_or_default();
    if notes.is_empty() {
        return None;
    }
    Some(note_link(id, &note_anchor(shape, &notes, notes.len() - 1)))
}

/// The link to the note `anchor` (as [`note_anchor`] writes it) of yak `id`: a
/// slug anchor, which only a new-shape body has, goes with the file path a
/// markdown host follows; a timestamp anchor keeps the bare id.
pub fn note_link(id: &str, anchor: &str) -> String {
    if is_slug_ts(split_note_anchor(anchor).0) {
        format!("/{}/yaks/{id}.md#{anchor}", crate::store::herd_of(id))
    } else {
        format!("{id}#{anchor}")
    }
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
/// An attachment name as a link writes it after `<id>/`: one or more
/// segments, a trailing `/` allowed (a directory, dropped here), and no
/// empty, `.` or `..` segment. `None` when it is not one.
fn clean_attachment(name: &str) -> Option<String> {
    let name = name.strip_suffix('/').unwrap_or(name);
    (!name.is_empty() && name.split('/').all(|s| !matches!(s, "" | "." | "..")))
        .then(|| name.to_string())
}

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

/// The link as one short label: `doks/x.md#setup`, `code:src/a.rs:L3-L9@abc1234`.
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
        Link::Yak { id, note: None } => id.clone(),
        Link::Yak { id, note: Some(n) } => format!("{id}#{n}"),
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

/// One link from a farm document: who links (`from`, a yak id or a dok
/// path), on which line of that document, to what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub line: usize,
    pub dest: String,
    pub link: Link,
}

/// Whether an attachment name is a markdown document, whose links are read.
pub fn is_markdown_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
}

/// Every markdown link in every yak (description and notes), every markdown
/// attachment (`from` is `<id>/<file>`), every dok and every session note of
/// the farm at `root`,
/// in document order. Yak ids are links too (mentions), but those are
/// `refs`' business; this is the markdown-link half.
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
    for t in &tasks {
        for a in crate::farm::list_attachments(root, &t.id) {
            if !is_markdown_name(&a.name) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&a.path) else {
                continue;
            };
            for f in extract(&text) {
                out.push(Edge {
                    from: format!("{}/{}", t.id, a.name),
                    line: f.line,
                    link: parse(&f.dest, From::Attachment(&t.id), is_yak),
                    dest: f.dest,
                });
            }
        }
    }
    // Session notes cite yak notes and doks (yaks-17f5): their links count too.
    for p in crate::doks::load_all(root)?
        .into_iter()
        .chain(crate::sessions::load_all(root)?)
    {
        for f in extract(&p.body) {
            out.push(Edge {
                from: p.path.clone(),
                line: f.line,
                link: parse(&f.dest, From::Dok(&p.path), is_yak),
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
    // example, not a link: a destination is checked when it names a file
    // (has an extension) or sits in a namespace that holds directories and
    // files without one (`/sources/<id>`, `/sessions/...`, and the wrong
    // `/doks/sources/<id>` spelling); any other extensionless path is prose.
    let checkable = |p: &str| {
        let ns = |s: &str| matches!(s, "sources" | "sessions" | "schemas");
        let first = p.split('/').next().unwrap_or("");
        let below_doks = p.strip_prefix("doks/").and_then(|r| r.split('/').next());
        !p.contains(['{', '}', '<', '>'])
            && (p.rsplit('/').next().is_some_and(|f| f.contains('.'))
                || (p.contains('/') && (ns(first) || below_doks.is_some_and(ns))))
    };
    match link {
        Link::Farm { path, .. } if !checkable(path) => None,
        Link::Attachment { name: path, .. } if path.contains(['{', '}', '<', '>']) => None,
        Link::Farm { path, .. } => Some(
            root.join(&*crate::locate::farm_file_in(root, path))
                .exists(),
        ),
        // A file, or an attached directory (`<id>/proj/`, yaks-a57c).
        Link::Attachment { id, name, .. } => {
            Some(crate::farm::attachment_path(root, id, name).is_some_and(|p| p.exists()))
        }
        Link::Upward(_) => Some(false),
        // A note link is checked for its note; a plain yak id is `refs`' business.
        Link::Yak { id, note: Some(n) } => Some(
            crate::store::load_task_by_id(root, id)
                .ok()
                .flatten()
                .is_some_and(|t| find_note(&crate::store::parse_notes(&t.body), n).is_some()),
        ),
        Link::Url(_) | Link::Code { .. } | Link::Local(_) | Link::Yak { .. } => None,
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
    let mut out = git(base, &["show", &format!("{rev}:./{path}")])?;
    // A pin from before schema 8 names a commit where the dok was still under
    // `wiki/`: read it from there (only revs before the rename can miss). A pin
    // from before the layout move names a commit where the whole farm was in
    // `.yaks/` (`{rev}:.yaks/...` is from the repository top, not the cwd), so
    // each spelling is tried there too.
    let old_doks = pre_doks_path(path);
    let mut tries = vec![format!("{rev}:.yaks/{path}")];
    if let Some(old) = &old_doks {
        tries.push(format!("{rev}:./{old}"));
        tries.push(format!("{rev}:.yaks/{old}"));
    }
    for spec in tries {
        if out.status.success() {
            break;
        }
        let next = git(base, &["show", &spec])?;
        if next.status.success() {
            out = next;
        }
    }
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

/// The schema-7 spelling of a farm path under a doks root: `doks/x.md` was
/// `wiki/x.md` and `<herd>/doks/x.md` was `<herd>/wiki/x.md`. `None` for any
/// other path.
fn pre_doks_path(path: &str) -> Option<String> {
    if let Some(rest) = path.strip_prefix("doks/") {
        return Some(format!("wiki/{rest}"));
    }
    let (herd, rest) = path.split_once('/')?;
    let rest = rest.strip_prefix("doks/")?;
    Some(format!("{herd}/wiki/{rest}"))
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

/// One note of a yak's file `text`, as it reads in the file: its marker
/// line, metadata and text.
fn note_text(id: &str, text: &str, anchor: &str) -> Result<String> {
    let body = crate::store::split_frontmatter(text).map_or(text, |(_, b)| b);
    let notes = crate::store::parse_notes(body);
    let Some(i) = find_note(&notes, anchor) else {
        bail!(
            "{id} has no note at {anchor} (`yaks show {id}` lists its notes; the link is the \
             note's timestamp or its slug, `-1`, `-2` for later notes of the same second)"
        );
    };
    let n = &notes[i];
    if crate::store::body_shape(body) == Some(crate::store::NoteShape::New) {
        let meta: Vec<(&str, &str)> = n
            .meta
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let note = crate::store::format_note(
            crate::store::NoteShape::New,
            &n.ts,
            n.actor.as_deref(),
            &meta,
            &n.text,
        );
        return Ok(format!("{note}\n"));
    }
    let mut s = format!(
        "\u{25b8} {}\n",
        crate::store::note_head(&n.ts, n.actor.as_deref())
    );
    for (k, v) in &n.meta {
        s.push_str(&format!("{k}: {v}\n"));
    }
    s.push('\n');
    s.push_str(&n.text);
    s.push('\n');
    Ok(s)
}

/// Read what `link` names. `context` widens a line range.
pub fn peek(link: &Link, bases: &Bases, context: u32) -> Result<Peek> {
    let label = label(link);
    let text = match link {
        Link::Url(u) => bail!("{u}: a URL; open it in a browser"),
        Link::Upward(s) => {
            bail!("{s}: a link may not climb out with ../; link from the farm root (/doks/...)")
        }
        Link::Local(_) => bail!("an in-document anchor needs its document"),
        Link::Yak { id, note } => {
            let path = crate::store::task_path(bases.farm, id);
            let text = std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("{id}: {e}"))?;
            match note {
                None => summary("yak.md", &text),
                Some(n) => note_text(id, &text, n)?,
            }
        }
        Link::Farm { path, rev, anchor } => {
            // `/sources/<id>` names a directory: its `source.md` is the file.
            let path = &*crate::locate::farm_file_in(bases.farm, path);
            let text = match rev {
                // A farm path at a commit: the farm root is `.yaks/` in its
                // worktree, so read through git from there.
                Some(r) => read_at(bases.farm, path, Some(r))?,
                None => read_at(bases.farm, path, None)?,
            };
            span(path, &text, anchor, context)?
        }
        Link::Attachment { id, name, anchor } => {
            let dir = crate::farm::attachment_path(bases.farm, id, name).filter(|p| p.is_dir());
            if let Some(dir) = dir {
                // An attached directory: its entries, one per line.
                let mut names: Vec<String> = std::fs::read_dir(&dir)?
                    .flatten()
                    .map(|e| {
                        let n = e.file_name().to_string_lossy().into_owned();
                        if e.path().is_dir() {
                            format!("{n}/")
                        } else {
                            n
                        }
                    })
                    .collect();
                names.sort();
                return Ok(Peek {
                    label,
                    text: names.iter().map(|n| format!("{n}\n")).collect(),
                });
            }
            let Some(found) = crate::farm::locate_attachment(bases.farm, id, name) else {
                bail!("{id}/{name}: no such attachment");
            };
            let text = read_at(&crate::store::artifact_dir(bases.farm, id), name, None)?;
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
/// in a worktree holding only the farm, so there it is the code checkout: the
/// git top-level of `cwd` unless that is the farm worktree itself, else the
/// repository's main checkout (yaks-6c4f). `None` when nothing fits.
pub fn code_base(farm_root: &Path, cwd: Option<&Path>) -> Option<PathBuf> {
    let Some(fg) = crate::store::farm_git(farm_root) else {
        // A `YAKS_DIR` farm: the code checkout holds it as `.yaks/`.
        return farm_root.parent().map(Path::to_path_buf);
    };
    let farm_wt = Some(crate::gitrepo::canonical(&fg.top));
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

    /// The form a markdown host can follow (yaks-acc0): a yak by its file
    /// and its attachment by their path from the farm root. The same `Link`s as
    /// the bare id and the `<id>/<file>` forms give, so a farm that mixes the
    /// two reads alike.
    #[test]
    fn a_yak_and_its_attachment_by_path_from_the_farm_root() {
        let f = From::Dok("doks/topics/x.md");
        let yak_link = |id: &str, note: Option<&str>| Link::Yak {
            id: id.into(),
            note: note.map(String::from),
        };
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b.md", f, yak),
            yak_link("yaks-1a2b", None)
        );
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b.md", f, yak),
            parse("yaks-1a2b", f, yak)
        );
        // A note, by either anchor form.
        for frag in [
            "2026-11-04T22:13:03Z",
            "2026-11-04t221303z",
            "2026-11-04t221303z-1",
        ] {
            assert_eq!(
                parse(&format!("/yaks/yaks/yaks-1a2b.md#{frag}"), f, yak),
                yak_link("yaks-1a2b", Some(frag)),
                "{frag}"
            );
        }
        // An attachment, from a dok, a yak or an attachment alike.
        let att = Link::Attachment {
            id: "yaks-1a2b".into(),
            name: "shot.png".into(),
            anchor: None,
        };
        for from in [f, From::Yak, From::Attachment("yaks-9999")] {
            assert_eq!(parse("/yaks/yaks/yaks-1a2b/shot.png", from, yak), att);
        }
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b/shot.png", f, yak),
            parse("yaks-1a2b/shot.png", f, yak)
        );
        // A deeper path is a file of an attached directory (yaks-a57c).
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b/sub/x.png", f, yak),
            Link::Attachment {
                id: "yaks-1a2b".into(),
                name: "sub/x.png".into(),
                anchor: None
            }
        );
        // Not a yak: another herd's directory, an unknown id, a pin, and a
        // dok that happens to sit under a `yaks/` directory.
        assert_eq!(
            parse("/other/yaks/yaks-1a2b.md", f, yak),
            farm("other/yaks/yaks-1a2b.md", None, None)
        );
        assert_eq!(
            parse("/yaks/yaks/yaks-nope.md", f, |s| s == "yaks-1a2b"),
            farm("yaks/yaks/yaks-nope.md", None, None)
        );
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b.md@3f2a9c1", f, yak),
            farm("yaks/yaks/yaks-1a2b.md", Some("3f2a9c1"), None)
        );
        assert_eq!(
            parse("/doks/yaks/notes.md", f, yak),
            farm("doks/yaks/notes.md", None, None)
        );
    }

    #[test]
    fn the_grammar() {
        let pg = From::Dok("doks/topics/x.md");
        assert_eq!(
            parse("https://a.test/b#c", pg, yak),
            Link::Url("https://a.test/b#c".into())
        );
        assert_eq!(parse("/doks/y.md", pg, yak), farm("doks/y.md", None, None));
        assert_eq!(
            parse("/doks/y.md@91be0d2#set-up", pg, yak),
            farm(
                "doks/y.md",
                Some("91be0d2"),
                Some(Anchor::Heading("set-up".into()))
            )
        );
        assert_eq!(
            parse("sub/z.md", pg, yak),
            farm("doks/topics/sub/z.md", None, None)
        );
        assert_eq!(parse("z.md", pg, yak), farm("doks/topics/z.md", None, None));
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
            parse("/doks/a@b.md", pg, yak),
            farm("doks/a@b.md", None, None)
        );
        // A yak id is a yak from anywhere, never a dok-relative file
        // (yaks-0e15); a non-yak bare word still is.
        for from in [pg, From::Yak, From::Cli] {
            assert_eq!(
                parse("yaks-44b3", from, yak),
                Link::Yak {
                    id: "yaks-44b3".into(),
                    note: None
                }
            );
        }
        assert_eq!(
            parse("notes", pg, yak),
            farm("doks/topics/notes", None, None)
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
            parse("../src/foo.rs", From::Dok("doks/a.md"), yak),
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

    /// An attached directory (yaks-a57c): `<id>/proj/` names the directory
    /// and `<id>/proj/src/a.rs` a file in it, as the bare id form and the
    /// farm-root path form; nothing climbs out or hides.
    #[test]
    fn a_directory_attachment_and_its_files_are_attachments() {
        let yak = |s: &str| s == "yaks-1a2b";
        let att = |name: &str| Link::Attachment {
            id: "yaks-1a2b".into(),
            name: name.into(),
            anchor: None,
        };
        assert_eq!(parse("yaks-1a2b/proj/", From::Yak, yak), att("proj"));
        assert_eq!(
            parse("yaks-1a2b/proj/src/a.rs", From::Yak, yak),
            att("proj/src/a.rs")
        );
        assert_eq!(
            parse(
                "/yaks/yaks/yaks-1a2b/proj/src/a.rs",
                From::Dok("doks/x.md"),
                yak
            ),
            att("proj/src/a.rs")
        );
        assert_eq!(
            parse("/yaks/yaks/yaks-1a2b/proj/", From::Dok("doks/x.md"), yak),
            att("proj")
        );
        for bad in ["yaks-1a2b/proj/../x", "yaks-1a2b//x", "yaks-1a2b/./x"] {
            assert!(
                !matches!(parse(bad, From::Yak, yak), Link::Attachment { .. }),
                "{bad}"
            );
        }
    }

    /// In a file attached to a yak, a bare name is a sibling attachment of
    /// that yak: what lets an evidence document's `![x](shot.png)` resolve in
    /// any markdown viewer and here. The rest reads as in a yak.
    #[test]
    fn a_bare_name_in_an_attachment_is_a_sibling() {
        let yak = |s: &str| s == "yaks-1a2b" || s == "yaks-3c4d";
        let from = From::Attachment("yaks-1a2b");
        let att = |id: &str, name: &str| Link::Attachment {
            id: id.into(),
            name: name.into(),
            anchor: None,
        };
        assert_eq!(parse("shot.png", from, yak), att("yaks-1a2b", "shot.png"));
        assert_eq!(parse("./shot.png", from, yak), att("yaks-1a2b", "shot.png"));
        assert_eq!(
            parse("my%20shot.png", from, yak),
            att("yaks-1a2b", "my shot.png")
        );
        // The qualified form still works, for this yak and another.
        assert_eq!(
            parse("yaks-3c4d/x.png", from, yak),
            att("yaks-3c4d", "x.png")
        );
        assert_eq!(
            parse("yaks-1a2b/shot.png#top", from, yak),
            Link::Attachment {
                id: "yaks-1a2b".into(),
                name: "shot.png".into(),
                anchor: Some(Anchor::Heading("top".into()))
            }
        );
        // A yak id is still a yak, a farm path a farm path.
        assert_eq!(
            parse("yaks-3c4d", from, yak),
            Link::Yak {
                id: "yaks-3c4d".into(),
                note: None
            }
        );
        assert_eq!(
            parse("/doks/a.md", from, yak),
            Link::Farm {
                path: "doks/a.md".into(),
                rev: None,
                anchor: None
            }
        );
        assert_eq!(
            parse("https://a.test/x.png", from, yak),
            Link::Url("https://a.test/x.png".into())
        );
        // An attachment directory is flat: a deeper relative path is not a
        // sibling, and `../` climbs out (legacy code forms aside).
        assert!(matches!(parse("sub/x.png", from, yak), Link::Farm { .. }));
        assert_eq!(
            parse("../../etc/x", from, yak),
            Link::Upward("../../etc/x".into())
        );
        assert!(matches!(parse("../src/a.rs", from, yak), Link::Code { .. }));
        // A pinned name is a farm path, not a sibling; an in-document anchor
        // stays local.
        assert!(matches!(
            parse("a.md@91be0d2", from, yak),
            Link::Farm { .. }
        ));
        assert!(matches!(parse("#top", from, yak), Link::Local(_)));
        // In a yak or on the command line a bare name still means the farm root.
        for from in [From::Yak, From::Cli] {
            assert!(matches!(parse("shot.png", from, yak), Link::Farm { .. }));
        }
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
        let text = "See [a](/doks/a.md).\n\n![shot](yaks-1a2b/s.png)\n\n`[not](x.md)`\n\n```\n[no](y.md)\n```\n";
        assert_eq!(
            extract(text),
            [
                Found {
                    dest: "/doks/a.md".into(),
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

    /// A link pinned before schema 8 (`@sha` of a commit where the dok was
    /// under `wiki/`) still reads it through the `doks/` spelling the
    /// migration wrote; a rev from after the rename reads `doks/` directly.
    #[test]
    fn a_pin_from_before_the_doks_rename_reads_the_old_wiki_path() {
        let dir = std::env::temp_dir().join(format!("yaks-pin-doks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("wiki/topics")).unwrap();
        let run = |args: &[&str]| {
            let o = Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .output()
                .unwrap();
            assert!(
                o.status.success(),
                "{args:?}: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(dir.join("wiki/topics/a.md"), "before the rename\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "wiki"]);
        let old = run(&["rev-parse", "HEAD"]);
        // The migration: a real `git mv` of the directory.
        run(&["mv", "wiki", "doks"]);
        std::fs::write(dir.join("doks/topics/a.md"), "after the rename\n").unwrap();
        run(&["commit", "-qam", "doks"]);
        let new = run(&["rev-parse", "HEAD"]);

        assert_eq!(
            read_at(&dir, "doks/topics/a.md", Some(&old)).unwrap(),
            "before the rename\n"
        );
        assert_eq!(
            read_at(&dir, "doks/topics/a.md", Some(&new)).unwrap(),
            "after the rename\n"
        );
        assert_eq!(
            read_at(&dir, "doks/topics/a.md", None).unwrap(),
            "after the rename\n"
        );
        let err = read_at(&dir, "doks/topics/none.md", Some(&old))
            .unwrap_err()
            .to_string();
        assert!(err.contains("none.md"), "{err}");
        assert_eq!(
            pre_doks_path("yak/doks/x.md").as_deref(),
            Some("yak/wiki/x.md")
        );
        assert_eq!(pre_doks_path("sources/x.md"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A pin into a commit from before the layout move (yaks-6ffd): the farm
    /// was in `.yaks/` then, and the worktree top is now the farm root. The
    /// `.yaks/` spelling is tried after the plain one, for both the current
    /// and the `wiki/` names.
    #[test]
    fn a_pin_from_before_the_layout_move_reads_the_dot_yaks_path() {
        let dir = std::env::temp_dir().join(format!("yaks-pin-layout-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".yaks/doks/topics")).unwrap();
        std::fs::create_dir_all(dir.join(".yaks/wiki/topics")).unwrap();
        let run = |args: &[&str]| {
            let o = Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .output()
                .unwrap();
            assert!(
                o.status.success(),
                "{args:?}: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(dir.join(".yaks/doks/topics/a.md"), "old layout\n").unwrap();
        std::fs::write(dir.join(".yaks/wiki/topics/w.md"), "old layout wiki\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "old"]);
        let old = run(&["rev-parse", "HEAD"]);
        run(&["mv", ".yaks/doks", "doks"]);
        std::fs::write(dir.join("doks/topics/a.md"), "root layout\n").unwrap();
        run(&["commit", "-qam", "move"]);
        let new = run(&["rev-parse", "HEAD"]);

        assert_eq!(
            read_at(&dir, "doks/topics/a.md", Some(&old)).unwrap(),
            "old layout\n"
        );
        assert_eq!(
            read_at(&dir, "doks/topics/a.md", Some(&new)).unwrap(),
            "root layout\n"
        );
        // Both moves at once: a `doks/` link at a commit with `.yaks/wiki/`.
        assert_eq!(
            read_at(&dir, "doks/topics/w.md", Some(&old)).unwrap(),
            "old layout wiki\n"
        );
        let err = read_at(&dir, "doks/topics/none.md", Some(&old))
            .unwrap_err()
            .to_string();
        assert!(err.contains("none.md"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `code:` links resolve in the code checkout for a farm branch in either
    /// layout, never in the farm worktree itself (yaks-6ffd).
    #[test]
    fn code_base_is_the_code_checkout_for_both_layouts() {
        let base = std::env::temp_dir().join(format!("yaks-codebase-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let code = base.join("code");
        std::fs::create_dir_all(&code).unwrap();
        let run = |dir: &Path, args: &[&str]| {
            let o = Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(["-c", "user.email=t@t", "-c", "user.name=t"])
                .args(args)
                .output()
                .unwrap();
            assert!(o.status.success(), "{args:?}: {o:?}");
        };
        run(&code, &["init", "-q", "-b", "main"]);
        std::fs::write(code.join("a.txt"), "a\n").unwrap();
        run(&code, &["add", "."]);
        run(&code, &["commit", "-qm", "code"]);
        let code = code.canonicalize().unwrap();
        for (name, nested) in [("root", false), ("dotyaks", true)] {
            let wt = base.join(format!("farm-{name}"));
            run(
                &code,
                &[
                    "worktree",
                    "add",
                    "-q",
                    "--orphan",
                    "-b",
                    "yaks",
                    wt.to_str().unwrap(),
                ],
            );
            let farm = if nested { wt.join(".yaks") } else { wt.clone() };
            std::fs::create_dir_all(&farm).unwrap();
            std::fs::write(farm.join("config.yaml"), "herd: y\n").unwrap();
            run(&wt, &["add", "."]);
            run(&wt, &["commit", "-qm", "farm"]);
            let farm = farm.canonicalize().unwrap();
            // From the code checkout: itself.
            assert_eq!(code_base(&farm, Some(&code)), Some(code.clone()), "{name}");
            // From inside the farm worktree: the repository's main checkout.
            assert_eq!(code_base(&farm, Some(&farm)), Some(code.clone()), "{name}");
            run(
                &code,
                &["worktree", "remove", "--force", wt.to_str().unwrap()],
            );
            run(&code, &["branch", "-q", "-D", "yaks"]);
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn peek_reads_lines_sections_and_summaries() {
        let root = std::env::temp_dir().join(format!("yaks-links-peek-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("doks")).unwrap();
        std::fs::write(root.join("doks/a.md"), "---\ntype: topic\ntitle: A\n---\n\nFirst para\nstill.\n\nSecond.\n\n## Setup\n\nSteps.\n").unwrap();
        let b = Bases {
            farm: &root,
            code: Some(&root),
        };
        let p = peek(&parse("/doks/a.md", From::Cli, yak), &b, 0).unwrap();
        assert_eq!(
            p.text,
            "---\ntype: topic\ntitle: A\n---\n\nFirst para\nstill.\n"
        );
        let p = peek(&parse("/doks/a.md#setup", From::Cli, yak), &b, 0).unwrap();
        assert_eq!(p.text, "## Setup\n\nSteps.\n");
        let p = peek(&parse("code:doks/a.md#L6-L7", From::Cli, yak), &b, 1).unwrap();
        assert_eq!(p.text, "5  \n6  First para\n7  still.\n8  \n");
        assert!(peek(&parse("/doks/none.md", From::Cli, yak), &b, 0).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_latest_note_link_of_a_new_shape_body_is_a_host_path() {
        let body = "Desc.\n\n#### 2026-11-04T22:13:03Z\n\n*a*\n\nhello\n";
        assert_eq!(
            latest_note_link("yaks-1a2b", body).unwrap(),
            "/yaks/yaks/yaks-1a2b.md#2026-11-04t221303z"
        );
        assert!(latest_note_link("yaks-1a2b", "Desc.").is_none());
    }

    /// A note is named by its timestamp; later notes of the same second take
    /// `-1`, `-2` (yaks-c5bd). A heading-like fragment on a yak id is no note.
    #[test]
    fn a_note_link_names_one_note_by_its_timestamp() {
        let ts = "2026-11-04T22:13:03Z";
        assert_eq!(
            parse(&format!("yaks-1a2b#{ts}"), From::Cli, yak),
            Link::Yak {
                id: "yaks-1a2b".into(),
                note: Some(ts.into())
            }
        );
        assert_eq!(
            parse("yaks-1a2b#setup", From::Cli, yak),
            Link::Yak {
                id: "yaks-1a2b".into(),
                note: None
            }
        );
        assert!(is_note_anchor(&format!("{ts}-2")));
        assert!(!is_note_anchor("2026-11-04"));

        let root = std::env::temp_dir().join(format!("yaks-links-note-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = crate::store::task_path(&root, "yaks-1a2b");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut body = String::from("Desc.");
        body = crate::store::append_note(&body, "2026-11-04T22:13:02Z", Some("a"), "first");
        body = crate::store::append_note(&body, ts, Some("a"), "the shear note");
        body = crate::store::append_note_meta(&body, ts, Some("a"), &[("kind", "move")], "moved");
        std::fs::write(
            &path,
            format!("---\nid: yaks-1a2b\ntitle: T\nstatus: shorn\n---\n\n{body}\n"),
        )
        .unwrap();

        let notes = crate::store::parse_notes(&body);
        use crate::store::NoteShape;
        assert_eq!(note_anchor(NoteShape::Old, &notes, 1), ts);
        assert_eq!(note_anchor(NoteShape::Old, &notes, 2), format!("{ts}-1"));
        assert_eq!(note_anchor(NoteShape::New, &notes, 1), "2026-11-04t221303z");
        assert_eq!(
            note_anchor(NoteShape::New, &notes, 2),
            "2026-11-04t221303z-1"
        );
        // Either form finds the note, in a body of either shape.
        assert_eq!(find_note(&notes, "2026-11-04t221303z-1"), Some(2));
        assert_eq!(find_note(&notes, "2026-11-04t221303z"), Some(1));
        assert!(is_note_anchor("2026-11-04t221303z"));
        assert!(is_note_anchor("2026-11-04t221303z-2"));
        assert!(!is_note_anchor("2026-11-04t22"));
        assert!(!is_note_anchor("setup"));
        assert_eq!(find_note(&notes, &format!("{ts}-1")), Some(2));
        assert_eq!(
            latest_note_link("yaks-1a2b", &body).unwrap(),
            format!("yaks-1a2b#{ts}-1")
        );

        let b = Bases {
            farm: &root,
            code: None,
        };
        let p = peek(&parse(&format!("yaks-1a2b#{ts}"), From::Cli, yak), &b, 0).unwrap();
        assert_eq!(p.text, format!("\u{25b8} {ts} [a]\n\nthe shear note\n"));
        let p = peek(&parse(&format!("yaks-1a2b#{ts}-1"), From::Cli, yak), &b, 0).unwrap();
        assert_eq!(p.text, format!("\u{25b8} {ts} [a]\nkind: move\n\nmoved\n"));
        let missing = parse("yaks-1a2b#2026-01-01T00:00:00Z", From::Cli, yak);
        assert!(peek(&missing, &b, 0).is_err());
        assert_eq!(exists(&root, &missing), Some(false));
        assert_eq!(
            exists(&root, &parse(&format!("yaks-1a2b#{ts}"), From::Cli, yak)),
            Some(true)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn exists_checks_directories_and_extensionless_paths_but_not_placeholders() {
        let root = std::env::temp_dir().join(format!("yaks-exists-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("sources/2026-11-03-81364278")).unwrap();
        std::fs::write(root.join("sources/2026-11-03-81364278/source.md"), "x").unwrap();
        std::fs::create_dir_all(root.join("doks")).unwrap();
        let check = |dest: &str| exists(&root, &parse(dest, From::Cli, yak));
        // `/sources/<id>` is its source.md; a wrong id or a wrong namespace is broken.
        assert_eq!(check("/sources/2026-11-03-81364278"), Some(true));
        assert_eq!(check("/sources/nope"), Some(false));
        assert_eq!(check("/doks/sources/2026-11-03-81364278"), Some(false));
        assert_eq!(check("/doks/sessions/2026/11/x"), Some(false));
        // Prose placeholders and one bare word are examples, not links.
        assert_eq!(check("/doks/{name}"), None);
        assert_eq!(check("path"), None);
        assert_eq!(check("doks/path"), None);
        assert_eq!(check("<id>/<file>"), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
