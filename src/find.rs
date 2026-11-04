//! `yaks find`: one text search over everything in the farm that is prose
//! (yak files, doks, sources' `source.md` and `extracted.md`, and
//! `sessions/**.md`), yaks-bdcb.
//!
//! It reads the files directly each time; there is no index (a farm is small,
//! see `yaks weight`, and the scan is timed in the yak's notes). A match is a
//! line, reported as the farm-relative path, its line number in the file
//! (frontmatter included, so `path:line` opens at the line) and the line's text.
//!
//! Matching is case-insensitive unless asked otherwise. The pattern is a
//! literal substring, or with `regex` a small regular-expression dialect
//! (see [`Pattern::regex`]) run by a linear-time matcher: the `regex` crate is
//! only a transitive dependency, so it is not used here.

use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::{doks, model::Status, sources, store};

/// Where a match was found; also the values of `--kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Yaks,
    Doks,
    Sources,
    Sessions,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Yaks => "yaks",
            Kind::Doks => "doks",
            Kind::Sources => "sources",
            Kind::Sessions => "sessions",
        }
    }
}

/// The farm-root directory holding imported session transcripts.
pub const SESSIONS_DIR: &str = "sessions";

/// Longest text of a reported line, in characters.
const SNIPPET: usize = 160;

/// One matching line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Farm-relative, `/`-separated.
    pub path: String,
    pub kind: Kind,
    /// The yak's, dok's or source's title.
    pub title: String,
    /// 1-based line number in the file.
    pub line: usize,
    /// The line, trimmed and cut to about [`SNIPPET`] characters around the match.
    pub text: String,
}

pub struct Options {
    pub pattern: Pattern,
    /// Farm-relative path prefixes (see [`scope_matches`]); empty means everything.
    pub scopes: Vec<String>,
    /// Empty means every kind.
    pub kinds: Vec<Kind>,
}

/// Every matching line of the farm at `root`, in kind order (yaks, doks,
/// sources, sessions) and path order within a kind.
pub fn find(root: &Path, opts: &Options) -> Vec<Hit> {
    let mut hits = Vec::new();
    for (kind, rel, abs) in candidates(root, opts) {
        scan(root, kind, &rel, &abs, &opts.pattern, &mut hits);
    }
    hits
}

/// The files to read, in output order, after the `--kind` and `--scope` filters.
fn candidates(root: &Path, opts: &Options) -> Vec<(Kind, String, PathBuf)> {
    let wants = |k: Kind| opts.kinds.is_empty() || opts.kinds.contains(&k);
    let rel_of = |p: &Path| {
        p.strip_prefix(root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/")
    };
    let mut out: Vec<(Kind, String, PathBuf)> = Vec::new();
    if wants(Kind::Yaks) {
        for p in store::yak_files(root) {
            out.push((Kind::Yaks, rel_of(&p), p));
        }
    }
    if wants(Kind::Doks) {
        for rel in doks::dok_paths(root) {
            let abs = root.join(&rel);
            out.push((Kind::Doks, rel, abs));
        }
    }
    if wants(Kind::Sources) {
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join(sources::SOURCES_DIR))
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        dirs.sort();
        for d in dirs {
            if d.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            for f in ["source.md", "extracted.md"] {
                let p = d.join(f);
                if p.is_file() {
                    out.push((Kind::Sources, rel_of(&p), p));
                }
            }
        }
    }
    if wants(Kind::Sessions) {
        let mut files = Vec::new();
        walk_md(&root.join(SESSIONS_DIR), &mut files);
        files.sort();
        for p in files {
            out.push((Kind::Sessions, rel_of(&p), p));
        }
    }
    out.retain(|(_, rel, _)| scope_matches(&opts.scopes, rel));
    out
}

/// Every `.md` file under `dir`, skipping dot entries.
fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        if e.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            walk_md(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

/// Whether farm-relative `rel` is inside any of `scopes`. A scope is a path
/// prefix (`doks/topics/vec` matches `doks/topics/vector-search.md`); one
/// with no `/` names a directory (a herd, `doks`, `sources`), so `web` means
/// `web/` and never matches `webby/`. No scopes matches everything.
pub fn scope_matches(scopes: &[String], rel: &str) -> bool {
    scopes.is_empty()
        || scopes.iter().any(|s| {
            let s = s.trim_start_matches('/');
            if s.contains('/') {
                rel.starts_with(s)
            } else {
                rel.starts_with(&format!("{s}/"))
            }
        })
}

/// Scan one file; a file that is not UTF-8 text (or has a NUL byte) is skipped.
fn scan(root: &Path, kind: Kind, rel: &str, abs: &Path, pat: &Pattern, hits: &mut Vec<Hit>) {
    let Ok(bytes) = std::fs::read(abs) else {
        return;
    };
    if bytes.contains(&0) {
        return;
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return;
    };
    let mut title: Option<String> = None;
    for (i, line) in text.lines().enumerate() {
        let Some(at) = pat.find(line) else {
            continue;
        };
        let title = title
            .get_or_insert_with(|| title_of(root, kind, rel, &text))
            .clone();
        hits.push(Hit {
            path: rel.to_string(),
            kind,
            title,
            line: i + 1,
            text: snippet(line, at),
        });
    }
}

/// The title shown on a file's group: a yak's, a dok's or session's, and for
/// a source (`source.md` or `extracted.md`) the title in its `source.md`.
fn title_of(root: &Path, kind: Kind, rel: &str, text: &str) -> String {
    match kind {
        Kind::Yaks => store::parse_task(text, Status::Hairy)
            .map(|t| t.title)
            .unwrap_or_default(),
        Kind::Doks | Kind::Sessions => doks::Dok::parse(rel, text).title(),
        Kind::Sources => {
            let source_md = match rel.rsplit_once('/') {
                Some((dir, _)) => format!("{dir}/source.md"),
                None => rel.to_string(),
            };
            match std::fs::read_to_string(root.join(&source_md)) {
                Ok(t) => doks::Dok::parse(&source_md, &t).title(),
                Err(_) => String::new(),
            }
        }
    }
}

/// `line` trimmed, and when still longer than [`SNIPPET`] cut to a window
/// around the match at char offset `at` of the untrimmed line, with `…` on a
/// cut side.
fn snippet(line: &str, at: usize) -> String {
    let line = line.trim_end();
    let lead = line.chars().take_while(|c| c.is_whitespace()).count();
    let chars: Vec<char> = line.trim_start().chars().collect();
    if chars.len() <= SNIPPET {
        return chars.into_iter().collect();
    }
    let m = at.saturating_sub(lead).min(chars.len() - 1);
    let start = if m < SNIPPET - 40 { 0 } else { m - 40 };
    let end = (start + SNIPPET).min(chars.len());
    let mut s = String::new();
    if start > 0 {
        s.push('…');
    }
    s.extend(&chars[start..end]);
    if end < chars.len() {
        s.push('…');
    }
    s
}

/// Plain text: per file a header (`path  title`), then one `path:line: text`
/// line per match; groups are separated by a blank line.
pub fn render_text(hits: &[Hit]) -> String {
    let mut out = String::new();
    let mut last: Option<&str> = None;
    for h in hits {
        if last != Some(h.path.as_str()) {
            if last.is_some() {
                out.push('\n');
            }
            if h.title.is_empty() {
                out.push_str(&format!("{}\n", h.path));
            } else {
                out.push_str(&format!("{}  {}\n", h.path, h.title));
            }
            last = Some(&h.path);
        }
        out.push_str(&format!("{}:{}: {}\n", h.path, h.line, h.text));
    }
    out
}

/// `[{path, kind, title, line, text}]`, `kind` being a `--kind` value.
pub fn to_json(hits: &[Hit]) -> Value {
    Value::Array(
        hits.iter()
            .map(|h| {
                json!({
                    "path": h.path, "kind": h.kind.name(), "title": h.title,
                    "line": h.line, "text": h.text,
                })
            })
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// Pattern
// ---------------------------------------------------------------------------

/// What a line is tested against.
pub struct Pattern {
    matcher: Matcher,
}

enum Matcher {
    /// `needle` is already lowercased when `fold`.
    Literal {
        needle: String,
        fold: bool,
    },
    Regex {
        prog: Prog,
        fold: bool,
    },
}

impl Pattern {
    /// A literal substring. `fold` ignores case.
    pub fn literal(s: &str, fold: bool) -> Result<Pattern> {
        if s.is_empty() {
            bail!("the pattern is empty");
        }
        let needle = if fold {
            s.to_lowercase()
        } else {
            s.to_string()
        };
        Ok(Pattern {
            matcher: Matcher::Literal { needle, fold },
        })
    }

    /// A regular expression: literals, `.`, classes `[a-z]` `[^x]` (with
    /// `\d \w \s` inside), `\d \w \s \D \W \S`, `\b \B`, `^ $`, groups `( )`
    /// and `(?: )`, alternation `|`, and the repeats `* + ? {n} {n,} {n,m}`
    /// (a trailing `?` on a repeat is accepted and ignored: only whether a line
    /// matches matters). `\` escapes any other character. Matching a line is
    /// linear in the line and the pattern.
    pub fn regex(s: &str, fold: bool) -> Result<Pattern> {
        if s.is_empty() {
            bail!("the pattern is empty");
        }
        let prog = Prog::compile(s, fold)?;
        Ok(Pattern {
            matcher: Matcher::Regex { prog, fold },
        })
    }

    /// The char offset in `line` where a match starts, if there is one.
    pub fn find(&self, line: &str) -> Option<usize> {
        match &self.matcher {
            Matcher::Literal {
                needle,
                fold: false,
            } => line
                .find(needle.as_str())
                .map(|b| line[..b].chars().count()),
            Matcher::Literal { needle, fold: true } => {
                let lower = line.to_lowercase();
                lower
                    .find(needle.as_str())
                    .map(|b| lower[..b].chars().count())
            }
            Matcher::Regex { prog, fold } => {
                let chars: Vec<char> = line.chars().collect();
                prog.find(&chars, *fold)
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ClassItem {
    Range(char, char),
    Digit(bool),
    Word(bool),
    Space(bool),
}

impl ClassItem {
    /// `true` for the negated shorthands (`\D`) is stored in the bool.
    fn has(&self, c: char) -> bool {
        match *self {
            ClassItem::Range(a, b) => a <= c && c <= b,
            ClassItem::Digit(neg) => c.is_ascii_digit() != neg,
            ClassItem::Word(neg) => is_word(c) != neg,
            ClassItem::Space(neg) => c.is_whitespace() != neg,
        }
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn upper(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[derive(Debug, Clone, Copy)]
enum Assertion {
    Start,
    End,
    WordBoundary,
    NotWordBoundary,
}

#[derive(Debug, Clone)]
enum Node {
    Empty,
    Char(char),
    Any,
    Class(bool, Vec<ClassItem>),
    Assert(Assertion),
    Cat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat(Box<Node>, u32, Option<u32>),
}

#[derive(Debug, Clone)]
enum Inst {
    Char(char),
    Any,
    Class(bool, Vec<ClassItem>),
    Assert(Assertion),
    Split(usize, usize),
    Jmp(usize),
    Match,
}

/// A compiled pattern: a Thompson NFA run by lock-step simulation.
pub struct Prog {
    insts: Vec<Inst>,
}

/// Most instructions a pattern may compile to (counted repeats multiply).
const MAX_INSTS: usize = 20_000;

struct Parser {
    chars: Vec<char>,
    pos: usize,
    fold: bool,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn alt(&mut self) -> Result<Node> {
        let mut arms = vec![self.cat()?];
        while self.eat('|') {
            arms.push(self.cat()?);
        }
        Ok(if arms.len() == 1 {
            arms.pop().unwrap()
        } else {
            Node::Alt(arms)
        })
    }

    fn cat(&mut self) -> Result<Node> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            items.push(self.repeat()?);
        }
        Ok(match items.len() {
            0 => Node::Empty,
            1 => items.pop().unwrap(),
            _ => Node::Cat(items),
        })
    }

    /// Consume a one-character repeat operator, giving its bounds.
    fn op(&mut self, min: u32, max: Option<u32>) -> (u32, Option<u32>) {
        self.pos += 1;
        (min, max)
    }

    fn repeat(&mut self) -> Result<Node> {
        let mut node = self.atom()?;
        loop {
            let (min, max) = match self.peek() {
                Some('*') => self.op(0, None),
                Some('+') => self.op(1, None),
                Some('?') => self.op(0, Some(1)),
                Some('{') => match self.counts()? {
                    Some(r) => r,
                    // Not a count: `{` is a literal, read by the next atom.
                    None => break,
                },
                _ => break,
            };
            self.eat('?'); // lazy marker: irrelevant to whether a line matches
            node = Node::Repeat(Box::new(node), min, max);
        }
        Ok(node)
    }

    /// At `{`: parse `{n}`, `{n,}` or `{n,m}` and advance past the `}`; `None`
    /// (nothing consumed) when it is not that shape.
    fn counts(&mut self) -> Result<Option<(u32, Option<u32>)>> {
        let rest: String = self.chars[self.pos..].iter().collect();
        let Some(end) = rest.find('}') else {
            return Ok(None);
        };
        let inner = &rest[1..end];
        let num = |s: &str| s.parse::<u32>().ok();
        let parsed = match inner.split_once(',') {
            None => num(inner).map(|n| (n, Some(n))),
            Some((a, "")) => num(a).map(|n| (n, None)),
            Some((a, b)) => match (num(a), num(b)) {
                (Some(a), Some(b)) => Some((a, Some(b))),
                _ => None,
            },
        };
        let Some((min, max)) = parsed else {
            return Ok(None);
        };
        if max.is_some_and(|m| m < min) {
            bail!("invalid repeat {{{inner}}}: the upper bound is below the lower");
        }
        if min > 1000 || max.is_some_and(|m| m > 1000) {
            bail!("repeat {{{inner}}} is too large (limit 1000)");
        }
        self.pos += inner.chars().count() + 2;
        Ok(Some((min, max)))
    }

    fn atom(&mut self) -> Result<Node> {
        let Some(c) = self.peek() else {
            bail!("unexpected end of pattern");
        };
        self.pos += 1;
        Ok(match c {
            '.' => Node::Any,
            '^' => Node::Assert(Assertion::Start),
            '$' => Node::Assert(Assertion::End),
            '(' => {
                if self.peek() == Some('?') {
                    self.pos += 1;
                    if !self.eat(':') {
                        bail!("unsupported group syntax `(?`: only `(?:` is supported");
                    }
                }
                let inner = self.alt()?;
                if !self.eat(')') {
                    bail!("missing `)` in the pattern");
                }
                inner
            }
            ')' => bail!("unmatched `)` in the pattern"),
            '*' | '+' | '?' => bail!("nothing to repeat before `{c}` (escape it as `\\{c}`)"),
            '[' => self.class()?,
            '\\' => {
                let Some(e) = self.peek() else {
                    bail!("a pattern cannot end in `\\`");
                };
                self.pos += 1;
                match e {
                    'd' => Node::Class(false, vec![ClassItem::Digit(false)]),
                    'D' => Node::Class(false, vec![ClassItem::Digit(true)]),
                    'w' => Node::Class(false, vec![ClassItem::Word(false)]),
                    'W' => Node::Class(false, vec![ClassItem::Word(true)]),
                    's' => Node::Class(false, vec![ClassItem::Space(false)]),
                    'S' => Node::Class(false, vec![ClassItem::Space(true)]),
                    'b' => Node::Assert(Assertion::WordBoundary),
                    'B' => Node::Assert(Assertion::NotWordBoundary),
                    'n' => self.lit('\n'),
                    't' => self.lit('\t'),
                    _ => self.lit(e),
                }
            }
            c => self.lit(c),
        })
    }

    fn lit(&self, c: char) -> Node {
        Node::Char(if self.fold { lower(c) } else { c })
    }

    fn class(&mut self) -> Result<Node> {
        let neg = self.eat('^');
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let Some(c) = self.peek() else {
                bail!("missing `]` in the pattern");
            };
            self.pos += 1;
            if c == ']' && !first {
                break;
            }
            first = false;
            let lo = if c == '\\' {
                let Some(e) = self.peek() else {
                    bail!("missing `]` in the pattern");
                };
                self.pos += 1;
                match e {
                    'd' => {
                        items.push(ClassItem::Digit(false));
                        continue;
                    }
                    'D' => {
                        items.push(ClassItem::Digit(true));
                        continue;
                    }
                    'w' => {
                        items.push(ClassItem::Word(false));
                        continue;
                    }
                    'W' => {
                        items.push(ClassItem::Word(true));
                        continue;
                    }
                    's' => {
                        items.push(ClassItem::Space(false));
                        continue;
                    }
                    'S' => {
                        items.push(ClassItem::Space(true));
                        continue;
                    }
                    'n' => '\n',
                    't' => '\t',
                    e => e,
                }
            } else {
                c
            };
            // `a-z`, but a `-` right before `]` is a literal.
            if self.peek() == Some('-') && self.chars.get(self.pos + 1).is_some_and(|&n| n != ']') {
                self.pos += 1;
                let mut hi = self.peek().unwrap();
                self.pos += 1;
                if hi == '\\' {
                    hi = self.peek().unwrap_or('\\');
                    self.pos += 1;
                }
                if hi < lo {
                    bail!("invalid class range `{lo}-{hi}`");
                }
                items.push(ClassItem::Range(lo, hi));
            } else {
                items.push(ClassItem::Range(lo, lo));
            }
        }
        Ok(Node::Class(neg, items))
    }
}

impl Prog {
    fn compile(pattern: &str, fold: bool) -> Result<Prog> {
        let mut p = Parser {
            chars: pattern.chars().collect(),
            pos: 0,
            fold,
        };
        let node = p.alt()?;
        if p.pos < p.chars.len() {
            bail!("unmatched `)` in the pattern");
        }
        let mut insts = Vec::new();
        emit(&node, &mut insts)?;
        insts.push(Inst::Match);
        Ok(Prog { insts })
    }

    /// The char offset where the first match in `line` starts.
    fn find(&self, line: &[char], fold: bool) -> Option<usize> {
        let n = self.insts.len();
        let mut clist: Vec<(usize, usize)> = Vec::new();
        let mut nlist: Vec<(usize, usize)> = Vec::new();
        // `seen[pc] == step + 1` once pc is on the list being built for `step`.
        let mut seen = vec![0usize; n];
        for i in 0..=line.len() {
            // Threads carried over were added for position i already; the
            // thread starting here goes last, so older starts win.
            self.add(&mut clist, &mut seen, i + 1, 0, i, i, line);
            nlist.clear();
            let c = line.get(i).copied();
            for &(pc, start) in &clist {
                match &self.insts[pc] {
                    Inst::Match => return Some(start),
                    Inst::Char(want) => {
                        if c.is_some_and(|c| c == *want || (fold && lower(c) == *want)) {
                            self.add(&mut nlist, &mut seen, i + 2, pc + 1, start, i + 1, line);
                        }
                    }
                    Inst::Any => {
                        if c.is_some() {
                            self.add(&mut nlist, &mut seen, i + 2, pc + 1, start, i + 1, line);
                        }
                    }
                    Inst::Class(neg, items) => {
                        if let Some(c) = c {
                            let hit = |c: char| items.iter().any(|it| it.has(c));
                            let found = hit(c) || (fold && (hit(lower(c)) || hit(upper(c))));
                            if found != *neg {
                                self.add(&mut nlist, &mut seen, i + 2, pc + 1, start, i + 1, line);
                            }
                        }
                    }
                    _ => {}
                }
            }
            std::mem::swap(&mut clist, &mut nlist);
        }
        None
    }

    /// Put `pc` (following jumps, splits and assertions at position `pos`) on
    /// `list`, once per `stamp`.
    #[allow(clippy::too_many_arguments)]
    fn add(
        &self,
        list: &mut Vec<(usize, usize)>,
        seen: &mut [usize],
        stamp: usize,
        pc: usize,
        start: usize,
        pos: usize,
        line: &[char],
    ) {
        if seen[pc] == stamp {
            return;
        }
        seen[pc] = stamp;
        match &self.insts[pc] {
            Inst::Jmp(t) => self.add(list, seen, stamp, *t, start, pos, line),
            Inst::Split(a, b) => {
                self.add(list, seen, stamp, *a, start, pos, line);
                self.add(list, seen, stamp, *b, start, pos, line);
            }
            Inst::Assert(a) => {
                let before = pos.checked_sub(1).map(|p| line[p]);
                let after = line.get(pos).copied();
                let word = |c: Option<char>| c.is_some_and(is_word);
                let ok = match a {
                    Assertion::Start => pos == 0,
                    Assertion::End => pos == line.len(),
                    Assertion::WordBoundary => word(before) != word(after),
                    Assertion::NotWordBoundary => word(before) == word(after),
                };
                if ok {
                    self.add(list, seen, stamp, pc + 1, start, pos, line);
                }
            }
            _ => list.push((pc, start)),
        }
    }
}

fn emit(node: &Node, out: &mut Vec<Inst>) -> Result<()> {
    if out.len() > MAX_INSTS {
        bail!("the pattern is too large");
    }
    match node {
        Node::Empty => {}
        Node::Char(c) => out.push(Inst::Char(*c)),
        Node::Any => out.push(Inst::Any),
        Node::Class(neg, items) => out.push(Inst::Class(*neg, items.clone())),
        Node::Assert(a) => out.push(Inst::Assert(*a)),
        Node::Cat(items) => {
            for n in items {
                emit(n, out)?;
            }
        }
        Node::Alt(arms) => {
            // split L1, L2; L1: arm; jmp end; L2: split ... ; last arm
            let mut jumps = Vec::new();
            for (i, arm) in arms.iter().enumerate() {
                if i + 1 < arms.len() {
                    let split = out.len();
                    out.push(Inst::Split(split + 1, 0));
                    emit(arm, out)?;
                    jumps.push(out.len());
                    out.push(Inst::Jmp(0));
                    let next = out.len();
                    out[split] = Inst::Split(split + 1, next);
                } else {
                    emit(arm, out)?;
                }
            }
            let end = out.len();
            for j in jumps {
                out[j] = Inst::Jmp(end);
            }
        }
        Node::Repeat(inner, min, max) => {
            for _ in 0..*min {
                emit(inner, out)?;
            }
            match max {
                None => {
                    // L: split body, end; body; jmp L
                    let split = out.len();
                    out.push(Inst::Split(split + 1, 0));
                    emit(inner, out)?;
                    out.push(Inst::Jmp(split));
                    let end = out.len();
                    out[split] = Inst::Split(split + 1, end);
                }
                Some(max) => {
                    // (max - min) nested optionals, all skipping to the same end
                    let mut splits = Vec::new();
                    for _ in *min..*max {
                        splits.push(out.len());
                        out.push(Inst::Split(out.len() + 1, 0));
                        emit(inner, out)?;
                    }
                    let end = out.len();
                    for s in splits {
                        out[s] = Inst::Split(s + 1, end);
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn re(p: &str) -> Pattern {
        Pattern::regex(p, true).unwrap()
    }

    #[test]
    fn literal_ignores_case_by_default_and_reports_the_char_offset() {
        let p = Pattern::literal("Wörld", true).unwrap();
        assert_eq!(p.find("héllo wÖRLD"), Some(6));
        let exact = Pattern::literal("World", false).unwrap();
        assert_eq!(exact.find("hello world"), None);
        assert_eq!(exact.find("hello World"), Some(6));
        assert!(Pattern::literal("", true).is_err());
    }

    #[test]
    fn regex_basics() {
        assert_eq!(re("a.c").find("xxabc"), Some(2));
        assert_eq!(re("^abc$").find("abc"), Some(0));
        assert_eq!(re("^abc$").find("abcd"), None);
        assert_eq!(re("colou?r").find("my color"), Some(3));
        assert_eq!(re("a+b").find("caaab"), Some(1));
        assert_eq!(re("(foo|bar)baz").find("xbarbaz"), Some(1));
        assert_eq!(re("(?:ab)*c").find("ababc"), Some(0));
        assert_eq!(re("\\.md$").find("x.md"), Some(1));
        assert_eq!(re("a|b|c").find("zzc"), Some(2));
    }

    #[test]
    fn regex_classes_counts_and_boundaries() {
        assert_eq!(re("[a-c]+\\d{2,3}").find("zz ab123x"), Some(3));
        assert_eq!(re("[a-c]+\\d{2,3}").find("ab1"), None);
        assert_eq!(re("^\\d{4}-\\d{2}$").find("2026-11"), Some(0));
        assert_eq!(re("[^a-z ]").find("abc d1"), Some(5));
        assert_eq!(re("[-a]+").find("x-a"), Some(1));
        assert_eq!(re("\\bcat\\b").find("concat cat"), Some(7));
        assert_eq!(re("\\bcat\\b").find("concat"), None);
        assert_eq!(re("x{2}").find("xx"), Some(0));
        assert_eq!(re("x{2}").find("x"), None);
        assert_eq!(re("x{2,}y").find("xxxxy"), Some(0));
        // `{` that is not a count is a literal.
        assert_eq!(re("a{b").find("a{b"), Some(0));
        assert_eq!(re("\\w+@\\w+").find("mail bob@host"), Some(5));
    }

    #[test]
    fn regex_case_folding_follows_the_flag() {
        assert_eq!(re("ABC[d-f]").find("xabcE"), Some(1));
        assert_eq!(Pattern::regex("ABC", false).unwrap().find("xabc"), None);
        assert_eq!(Pattern::regex("a[b-d]", false).unwrap().find("xaB"), None);
        assert_eq!(
            Pattern::regex("a[b-d]", false).unwrap().find("xab"),
            Some(1)
        );
    }

    #[test]
    fn nested_repeats_stay_linear() {
        let line = "a".repeat(5000) + "!";
        assert_eq!(re("(a*)*b").find(&line), None);
        assert_eq!(re("(a+)+$").find(&line), None);
    }

    #[test]
    fn bad_patterns_are_errors() {
        for bad in ["(", ")", "[abc", "*a", "a\\", "x{3,1}", "(?=a)", "[z-a]"] {
            assert!(Pattern::regex(bad, true).is_err(), "{bad}");
        }
    }

    #[test]
    fn scopes_are_directory_names_or_path_prefixes() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(scope_matches(&[], "anything"));
        assert!(scope_matches(&s(&["web"]), "web/yaks/web-1.md"));
        assert!(!scope_matches(&s(&["web"]), "webby/yaks/x.md"));
        assert!(scope_matches(
            &s(&["doks/topics/vec"]),
            "doks/topics/vector.md"
        ));
        assert!(scope_matches(&s(&["/doks/"]), "doks/a.md"));
        assert!(scope_matches(
            &s(&["nope", "sources"]),
            "sources/x/source.md"
        ));
    }

    #[test]
    fn long_lines_are_cut_around_the_match() {
        let line = format!("{}NEEDLE{}", "a".repeat(300), "b".repeat(300));
        let s = snippet(&line, 300);
        assert!(s.contains("NEEDLE") && s.starts_with('…') && s.ends_with('…'));
        assert!(s.chars().count() <= SNIPPET + 2);
        assert_eq!(snippet("   short  ", 3), "short");
        let early = format!("NEEDLE{}", "b".repeat(300));
        assert!(snippet(&early, 0).starts_with("NEEDLE") && snippet(&early, 0).ends_with('…'));
    }
}
