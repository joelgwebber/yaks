//! Frontmatter fields as data (yaks-1f34, design yaks-8384): an ordered set of
//! top-level keys whose values are parsed from a small YAML subset, with an
//! exact round-trip of every entry nobody edited.
//!
//! [`Fields`] keeps each top-level entry as the raw lines it was read from
//! (`key: value` plus any indented or `- ` continuation lines). Rendering
//! re-emits those lines untouched, so a file only changes where a key was set
//! or removed. A value is parsed only when asked for ([`Fields::get`]), which
//! keeps loading a large farm as cheap as before.
//!
//! The subset is what yaks, Hearth/OKF pages, sources and schemas actually
//! use: plain, single- and double-quoted scalars; flow lists `[a, b]` and flow
//! maps `{a: b}`, nested; block lists (of scalars or of maps) and block maps,
//! nested by indentation; literal and folded block scalars (`|`, `>`); and
//! `#` comments. Anchors, aliases, tags and multiple documents are not
//! supported: such an entry still round-trips, it just has no parsed value.

// Readers used by tests now and by pages and schemas next (yaks-9af0, yaks-8e75).
#![allow(dead_code)]

use serde_json::{Map, Value as Json, json};

/// A parsed frontmatter value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// No value (`key:` with nothing after it, `~`, `null`).
    Null,
    /// A scalar written without quotes: may read as a number, bool or date.
    Plain(String),
    /// A quoted or block scalar: always a string.
    Str(String),
    List(Vec<Value>),
    /// Keys in file order.
    Map(Vec<(String, Value)>),
}

impl Value {
    /// The scalar text of a plain or quoted scalar; `None` for anything else.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Plain(s) | Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// A list's items; a lone scalar reads as a one-item list, `Null` as empty.
    pub fn as_list(&self) -> Vec<&Value> {
        match self {
            Value::List(items) => items.iter().collect(),
            Value::Null => Vec::new(),
            other => vec![other],
        }
    }

    /// The value under `key` in a map.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// JSON for `--json`: a plain scalar that reads as a bool, integer or
    /// float becomes one; everything else is a string.
    pub fn to_json(&self) -> Json {
        match self {
            Value::Null => Json::Null,
            Value::Str(s) => json!(s),
            Value::Plain(s) => plain_json(s),
            Value::List(items) => Json::Array(items.iter().map(Value::to_json).collect()),
            Value::Map(entries) => {
                let mut m = Map::new();
                for (k, v) in entries {
                    m.insert(k.clone(), v.to_json());
                }
                Json::Object(m)
            }
        }
    }

    /// A value from a command-line `key=value`: `[a, b]` and `{a: b}` parse
    /// as flow collections, an empty string is `Null`, anything else is a
    /// plain scalar (quoted on output when YAML would misread it).
    pub fn from_cli(text: &str) -> Value {
        let t = text.trim();
        if t.is_empty() {
            return Value::Null;
        }
        if t.starts_with('[') || t.starts_with('{') {
            if let Some(v) = parse_flow(t) {
                return v;
            }
        }
        Value::Plain(t.to_string())
    }
}

fn plain_json(s: &str) -> Json {
    match s {
        "true" | "True" | "TRUE" => return json!(true),
        "false" | "False" | "FALSE" => return json!(false),
        _ => {}
    }
    if let Ok(i) = s.parse::<i64>() {
        return json!(i);
    }
    if s.bytes().any(|b| b.is_ascii_digit())
        && s.bytes()
            .all(|b| b.is_ascii_digit() || b".-+eE".contains(&b))
    {
        if let Ok(f) = s.parse::<f64>() {
            return json!(f);
        }
    }
    json!(s)
}

/// One top-level frontmatter entry: its key (`None` for a comment or blank
/// line kept between entries) and the exact lines it was written as.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    key: Option<String>,
    lines: Vec<String>,
}

/// An ordered set of frontmatter entries that round-trips byte for byte.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fields {
    entries: Vec<Entry>,
}

/// The key of a top-level `key: ...` line, if it is one: no indentation, not
/// a list item, not a comment, and a key of `[A-Za-z_][A-Za-z0-9_.-]*` (or a
/// quoted key) before the colon.
fn top_key(line: &str) -> Option<String> {
    if line.starts_with([' ', '\t', '-', '#']) {
        return None;
    }
    let (k, rest) = split_key(line)?;
    (rest.is_empty() || rest.starts_with([' ', '\t'])).then_some(k)
}

/// Split `key: rest` (or `key:` at the end of the line) into the key, unquoted,
/// and the text after the colon.
fn split_key(line: &str) -> Option<(String, &str)> {
    let t = line.trim_start();
    if let Some(q @ ('\'' | '"')) = t.chars().next() {
        let end = t[1..].find(q)? + 1;
        let rest = t[end + 1..].strip_prefix(':')?;
        return Some((t[1..end].to_string(), rest));
    }
    let colon = t.find(':')?;
    let k = &t[..colon];
    let rest = &t[colon + 1..];
    let ok = k
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
    if !ok || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    Some((k.to_string(), rest))
}

impl Fields {
    /// Parse the frontmatter text between the `---` lines. Never fails: any
    /// line that is not a new top-level key belongs to the entry before it.
    pub fn parse(text: &str) -> Fields {
        let mut entries: Vec<Entry> = Vec::new();
        for line in text.lines() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if let Some(k) = top_key(line) {
                entries.push(Entry {
                    key: Some(k),
                    lines: vec![line.to_string()],
                });
                continue;
            }
            let continues = !line.trim().is_empty() && !line.starts_with('#');
            match entries.last_mut() {
                Some(e) if continues && e.key.is_some() => e.lines.push(line.to_string()),
                _ => entries.push(Entry {
                    key: None,
                    lines: vec![line.to_string()],
                }),
            }
        }
        Fields { entries }
    }

    /// Build from already-grouped raw lines (each entry's first line is its
    /// `key: ...`), as [`crate::store::parse_task`] collects unmodelled keys.
    pub fn from_lines<S: AsRef<str>>(lines: &[S]) -> Fields {
        let text: Vec<&str> = lines.iter().map(|s| s.as_ref()).collect();
        Fields::parse(&text.join("\n"))
    }

    /// The frontmatter text, one line per `\n`-terminated line (no `---`).
    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in self.lines() {
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    /// Every raw line, in order.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .flat_map(|e| e.lines.iter().map(String::as_str))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|e| e.key.is_none())
    }

    /// The keys, in file order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().filter_map(|e| e.key.as_deref())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.keys().any(|k| k == key)
    }

    /// The raw lines of `key`'s entry.
    pub fn raw(&self, key: &str) -> Option<&[String]> {
        self.entry(key).map(|e| e.lines.as_slice())
    }

    fn entry(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.key.as_deref() == Some(key))
    }

    /// `key`'s parsed value. `None` when the key is absent or its value uses
    /// YAML this parser does not read (it still round-trips).
    pub fn get(&self, key: &str) -> Option<Value> {
        parse_entry(&self.entry(key)?.lines)
    }

    /// Every key with its value (`None` where it could not be parsed).
    pub fn iter(&self) -> impl Iterator<Item = (&str, Option<Value>)> {
        self.entries
            .iter()
            .filter_map(|e| Some((e.key.as_deref()?, parse_entry(&e.lines))))
    }

    /// Set `key` to `value`: its entry is rewritten in place, or appended at
    /// the end when new. Other entries keep their exact text.
    pub fn set(&mut self, key: &str, value: &Value) {
        let lines = render_entry(key, value);
        match self
            .entries
            .iter_mut()
            .find(|e| e.key.as_deref() == Some(key))
        {
            Some(e) => e.lines = lines,
            None => self.entries.push(Entry {
                key: Some(key.to_string()),
                lines,
            }),
        }
    }

    /// Set `key` to `value` written on one line (`key: [a, b]`) when it fits
    /// there, else as [`Fields::set`] writes it. Keeps a flow-style list in
    /// flow style when it is rewritten.
    pub fn set_flow(&mut self, key: &str, value: &Value) {
        if matches!(value, Value::List(_) | Value::Map(_)) && flow_fits(value) {
            self.set_raw(
                key,
                vec![format!("{}: {}", render_key(key), render_flow(value))],
            );
        } else {
            self.set(key, value);
        }
    }

    /// Replace `key`'s entry with these exact lines (the first must be its
    /// `key: ...` line). Used by the merge driver to keep a side's formatting.
    pub fn set_raw(&mut self, key: &str, lines: Vec<String>) {
        match self
            .entries
            .iter_mut()
            .find(|e| e.key.as_deref() == Some(key))
        {
            Some(e) => e.lines = lines,
            None => self.entries.push(Entry {
                key: Some(key.to_string()),
                lines,
            }),
        }
    }

    /// Remove `key`; true if it was there.
    pub fn remove(&mut self, key: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.key.as_deref() != Some(key));
        self.entries.len() != before
    }

    /// A JSON object of every key whose value parses.
    pub fn to_json(&self) -> Json {
        let mut m = Map::new();
        for (k, v) in self.iter() {
            if let Some(v) = v {
                m.insert(k.to_string(), v.to_json());
            }
        }
        Json::Object(m)
    }
}

// ---------------------------------------------------------------- parsing

/// One logical line: its indentation and text with any trailing comment
/// removed. Blank and comment-only lines are dropped before parsing.
#[derive(Debug, Clone)]
struct Line<'a> {
    indent: usize,
    text: &'a str,
    /// The raw line (for block scalars, which keep comments and spacing).
    raw: &'a str,
}

fn logical(lines: &[String]) -> Vec<Line<'_>> {
    lines
        .iter()
        .map(|l| {
            let indent = l.len() - l.trim_start_matches(' ').len();
            Line {
                indent,
                text: strip_comment(l[indent..].trim_end()),
                raw: l,
            }
        })
        .collect()
}

/// Drop a ` #` comment that is outside quotes and brackets' quoted strings.
fn strip_comment(s: &str) -> &str {
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    for (i, c) in s.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if (c == '\'' || c == '"') && (prev == ' ' || "[{,:".contains(prev) || i == 0) => {
                quote = Some(c)
            }
            None if c == '#' && (i == 0 || prev == ' ' || prev == '\t') => {
                return s[..i].trim_end();
            }
            None => {}
        }
        prev = c;
    }
    s
}

fn parse_entry(lines: &[String]) -> Option<Value> {
    let all = logical(lines);
    let first = all.first()?;
    let (_, rest) = split_key(first.text)?;
    let rest = rest.trim();
    let mut p = Parser { lines: all, pos: 1 };
    let v = p.value_after_key(rest, 0)?;
    p.skip_blank();
    (p.pos >= p.lines.len()).then_some(v)
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn skip_blank(&mut self) {
        while self.lines.get(self.pos).is_some_and(|l| l.text.is_empty()) {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<Line<'a>> {
        self.skip_blank();
        self.lines.get(self.pos).cloned()
    }

    /// The value of a key whose line is at `indent`, given the text after its
    /// colon. Nested blocks go deeper than `indent`; a block list may sit at
    /// the same indentation as its key (the style yaks writes).
    fn value_after_key(&mut self, rest: &str, indent: usize) -> Option<Value> {
        if rest.starts_with('|') || rest.starts_with('>') {
            return Some(self.block_scalar(rest, indent));
        }
        if !rest.is_empty() {
            return parse_inline(rest);
        }
        match self.peek() {
            Some(l) if l.text.starts_with("- ") || l.text == "-" => {
                if l.indent >= indent {
                    self.block_list(l.indent)
                } else {
                    Some(Value::Null)
                }
            }
            Some(l) if l.indent > indent => self.block_map(l.indent),
            _ => Some(Value::Null),
        }
    }

    fn block_list(&mut self, indent: usize) -> Option<Value> {
        let mut items = Vec::new();
        while let Some(l) = self.peek() {
            if l.indent != indent || !(l.text.starts_with("- ") || l.text == "-") {
                if l.indent > indent {
                    return None;
                }
                break;
            }
            self.pos += 1;
            let item = l.text[1..].trim_start();
            // The item's own content starts this many columns in.
            let inner = indent + (l.text.len() - item.len());
            if item.is_empty() {
                match self.peek() {
                    Some(n) if n.indent > indent => {
                        let v = if n.text.starts_with("- ") {
                            self.block_list(n.indent)?
                        } else {
                            self.block_map(n.indent)?
                        };
                        items.push(v);
                    }
                    _ => items.push(Value::Null),
                }
            } else if is_map_item(item) {
                // `- key: value` opens a map whose other keys line up with it.
                items.push(self.map_from(item, inner)?);
            } else {
                items.push(parse_inline(item)?);
            }
        }
        Some(Value::List(items))
    }

    fn block_map(&mut self, indent: usize) -> Option<Value> {
        let mut entries = Vec::new();
        while let Some(l) = self.peek() {
            if l.indent < indent {
                break;
            }
            if l.indent > indent {
                return None;
            }
            self.pos += 1;
            let (k, rest) = split_key(l.text)?;
            let v = self.value_after_key(rest.trim(), indent)?;
            entries.push((k, v));
        }
        Some(Value::Map(entries))
    }

    /// A map whose first `key: value` is `first` (already consumed, from a
    /// `- ` item) and whose other keys sit at `indent`.
    fn map_from(&mut self, first: &str, indent: usize) -> Option<Value> {
        let (k, rest) = split_key(first)?;
        let mut entries = vec![(k, self.value_after_key(rest.trim(), indent)?)];
        if let Some(Value::Map(more)) = self
            .peek()
            .filter(|l| l.indent == indent)
            .and_then(|_| self.block_map(indent))
        {
            entries.extend(more);
        }
        Some(Value::Map(entries))
    }

    fn block_scalar(&mut self, header: &str, indent: usize) -> Value {
        let folded = header.starts_with('>');
        let keep = header.contains('+');
        let strip = header.contains('-');
        let mut body: Vec<&str> = Vec::new();
        let mut base: Option<usize> = None;
        while let Some(l) = self.lines.get(self.pos) {
            let blank = l.raw.trim().is_empty();
            if !blank && l.indent <= indent {
                break;
            }
            if !blank && base.is_none() {
                base = Some(l.indent);
            }
            body.push(l.raw);
            self.pos += 1;
        }
        let base = base.unwrap_or(indent + 1);
        let lines: Vec<&str> = body
            .iter()
            .map(|r| if r.len() >= base { &r[base..] } else { "" })
            .collect();
        let mut text = if folded {
            let mut out = String::new();
            for (i, l) in lines.iter().enumerate() {
                if i > 0 {
                    out.push(if l.is_empty() || lines[i - 1].is_empty() {
                        '\n'
                    } else {
                        ' '
                    });
                }
                out.push_str(l);
            }
            out
        } else {
            lines.join("\n")
        };
        if !keep {
            text = text.trim_end_matches('\n').to_string();
            if !strip {
                text.push('\n');
            }
        }
        Value::Str(text)
    }
}

/// Whether a list item's text is `key: value` (a map), not a scalar.
fn is_map_item(item: &str) -> bool {
    !item.starts_with(['[', '{', '\'', '"']) && split_key(item).is_some()
}

/// A value written on one line after a key or `- `.
fn parse_inline(text: &str) -> Option<Value> {
    let t = text.trim();
    match t.chars().next()? {
        '[' | '{' => parse_flow(t),
        '\'' | '"' => {
            let (s, rest) = quoted(t)?;
            rest.trim().is_empty().then_some(Value::Str(s))
        }
        '&' | '*' | '!' => None,
        _ => Some(plain(t)),
    }
}

fn plain(t: &str) -> Value {
    match t {
        "~" | "null" | "Null" | "NULL" => Value::Null,
        _ => Value::Plain(t.to_string()),
    }
}

/// A quoted scalar at the start of `t`: its value and the text after it.
fn quoted(t: &str) -> Option<(String, &str)> {
    let q = t.chars().next()?;
    let mut out = String::new();
    let mut chars = t[1..].char_indices();
    while let Some((i, c)) = chars.next() {
        if q == '\'' && c == '\'' {
            if t[1 + i + 1..].starts_with('\'') {
                out.push('\'');
                chars.next();
                continue;
            }
            return Some((out, &t[1 + i + 1..]));
        }
        if q == '"' && c == '\\' {
            let (_, e) = chars.next()?;
            out.push(match e {
                'n' => '\n',
                't' => '\t',
                '0' => '\0',
                other => other,
            });
            continue;
        }
        if q == '"' && c == '"' {
            return Some((out, &t[1 + i + 1..]));
        }
        out.push(c);
    }
    None
}

/// A flow collection, `[...]` or `{...}`, taking the whole of `t`.
fn parse_flow(t: &str) -> Option<Value> {
    let (v, rest) = flow(t.trim())?;
    rest.trim().is_empty().then_some(v)
}

fn flow(t: &str) -> Option<(Value, &str)> {
    let open = t.chars().next()?;
    let close = match open {
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let mut rest = t[1..].trim_start();
    let mut items: Vec<Value> = Vec::new();
    let mut entries: Vec<(String, Value)> = Vec::new();
    loop {
        if let Some(r) = rest.strip_prefix(close) {
            return Some((
                if open == '[' {
                    Value::List(items)
                } else {
                    Value::Map(entries)
                },
                r,
            ));
        }
        if open == '{' {
            let (k, r) = flow_scalar(rest, true)?;
            let r = r.trim_start().strip_prefix(':')?.trim_start();
            let (v, r) = if r.starts_with([',', '}']) {
                (Value::Null, r)
            } else {
                flow_item(r)?
            };
            entries.push((k.as_str()?.to_string(), v));
            rest = r.trim_start();
        } else {
            let (v, r) = flow_item(rest)?;
            items.push(v);
            rest = r.trim_start();
        }
        if let Some(r) = rest.strip_prefix(',') {
            rest = r.trim_start();
        } else if !rest.starts_with(close) {
            return None;
        }
    }
}

fn flow_item(t: &str) -> Option<(Value, &str)> {
    if t.starts_with(['[', '{']) {
        flow(t)
    } else {
        flow_scalar(t, false)
    }
}

/// A scalar inside a flow collection: quoted, or plain up to `,` `]` `}` (and
/// up to `: ` when it is a map key).
fn flow_scalar(t: &str, key: bool) -> Option<(Value, &str)> {
    if t.starts_with(['\'', '"']) {
        let (s, r) = quoted(t)?;
        return Some((Value::Str(s), r));
    }
    let bytes = t.as_bytes();
    let mut end = t.len();
    for (i, &b) in bytes.iter().enumerate() {
        let colon_ends = b == b':'
            && (i + 1 == bytes.len() || matches!(bytes[i + 1], b' ' | b',' | b'}' | b']'));
        if matches!(b, b',' | b']' | b'}') || (key && colon_ends) {
            end = i;
            break;
        }
    }
    let s = t[..end].trim();
    if s.is_empty() {
        return None;
    }
    Some((plain(s), &t[end..]))
}

// ---------------------------------------------------------------- writing

/// The lines for `key: value`, in the style yaks writes: scalars plain unless
/// YAML would misread them, lists of scalars as block lists at the key's
/// indentation, short maps of scalars in flow style, anything else as nested
/// blocks.
fn render_entry(key: &str, value: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let k = render_key(key);
    match value {
        Value::Null => out.push(format!("{k}:")),
        Value::List(items) if items.is_empty() => out.push(format!("{k}: []")),
        Value::Map(entries) if entries.is_empty() => out.push(format!("{k}: {{}}")),
        Value::List(_) | Value::Map(_) if flow_fits(value) && matches!(value, Value::Map(_)) => {
            out.push(format!("{k}: {}", render_flow(value)))
        }
        Value::List(items) => {
            out.push(format!("{k}:"));
            // Scalars flush with the key (yaks' own style for `tags:` and
            // `depends_on:`); a list holding maps indented two, as OKF pages
            // and Hearth's `source.md` write `sources:`.
            let indent = if items.iter().any(|i| matches!(i, Value::Map(_))) {
                2
            } else {
                0
            };
            for item in items {
                render_item(item, indent, &mut out);
            }
        }
        Value::Map(entries) => {
            out.push(format!("{k}:"));
            for (ek, ev) in entries {
                render_nested(ek, ev, 2, &mut out);
            }
        }
        scalar => out.push(format!("{k}: {}", render_scalar(scalar))),
    }
    out
}

fn render_nested(key: &str, value: &Value, indent: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    let k = render_key(key);
    match value {
        Value::List(items) if !items.is_empty() && !flow_fits(value) => {
            out.push(format!("{pad}{k}:"));
            for item in items {
                render_item(item, indent, out);
            }
        }
        Value::Map(entries) if !entries.is_empty() && !flow_fits(value) => {
            out.push(format!("{pad}{k}:"));
            for (ek, ev) in entries {
                render_nested(ek, ev, indent + 2, out);
            }
        }
        Value::List(_) | Value::Map(_) => out.push(format!("{pad}{k}: {}", render_flow(value))),
        Value::Null => out.push(format!("{pad}{k}:")),
        scalar => out.push(format!("{pad}{k}: {}", render_scalar(scalar))),
    }
}

fn render_item(item: &Value, indent: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match item {
        Value::Map(entries) if !entries.is_empty() && !flow_fits(item) => {
            let mut first = true;
            for (k, v) in entries {
                let mut sub = Vec::new();
                render_nested(k, v, indent + 2, &mut sub);
                if first {
                    sub[0] = format!("{pad}- {}", sub[0].trim_start());
                    first = false;
                }
                out.extend(sub);
            }
        }
        Value::List(_) | Value::Map(_) => out.push(format!("{pad}- {}", render_flow(item))),
        Value::Null => out.push(format!("{pad}-")),
        scalar => out.push(format!("{pad}- {}", render_scalar(scalar))),
    }
}

/// Whether a collection is small enough to write on one line: scalars only,
/// at most one level of nesting, and under 80 columns.
fn flow_fits(v: &Value) -> bool {
    let scalars = match v {
        Value::List(items) => items
            .iter()
            .all(|i| !matches!(i, Value::List(_) | Value::Map(_))),
        Value::Map(entries) => entries
            .iter()
            .all(|(_, i)| !matches!(i, Value::List(_) | Value::Map(_))),
        _ => true,
    };
    scalars && render_flow(v).len() <= 72
}

fn render_flow(v: &Value) -> String {
    match v {
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(render_flow).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}: {}", render_key(k), render_flow(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Value::Null => "null".to_string(),
        Value::Plain(s) | Value::Str(s) => {
            let r = render_scalar(v);
            if r == *s && s.contains([',', '[', ']', '{', '}']) {
                format!("'{}'", s.replace('\'', "''"))
            } else {
                r
            }
        }
    }
}

fn render_key(k: &str) -> String {
    if split_key(&format!("{k}:")).is_some_and(|(kk, _)| kk == k) {
        k.to_string()
    } else {
        format!("'{}'", k.replace('\'', "''"))
    }
}

/// A scalar: a plain one as written when YAML would read it back the same
/// (numbers and bools stay unquoted), a string quoted only when needed.
fn render_scalar(v: &Value) -> String {
    match v {
        Value::Plain(s) if !s.contains('\n') && plain_safe(s) => s.clone(),
        Value::Plain(s) | Value::Str(s) if s.contains('\n') => {
            format!(
                "\"{}\"",
                s.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
            )
        }
        Value::Plain(s) | Value::Str(s) => crate::store::write::scalar(s),
        Value::Null => "null".to_string(),
        _ => String::new(),
    }
}

/// Whether a plain scalar reads back as the same text (numbers, bools and
/// dates are fine: they are meant as such).
fn plain_safe(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    !(first.is_whitespace()
        || s.ends_with(char::is_whitespace)
        || "!&*?|>%@`\"'#,[]{}:".contains(first)
        || s.starts_with("- ")
        || s == "-"
        || s.contains(": ")
        || s.ends_with(':')
        || s.contains(" #")
        || matches!(s, "~" | "null" | "Null" | "NULL"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn untouched_frontmatter_round_trips_byte_for_byte() {
        let text = "\
id: yaks-1a2b
title: 'It''s here: a title'
labels:
- ui
- docs
# a comment

generated: {by: agent:joel@joel, at: 2026-10-28T16:14:47Z}
sources:
  - id: file
    resource: \"/tmp/x.png\"   # trailing comment
    as_of: 2025-11-25
fields:
  platform: {type: text, description: Product name (the row key)}
  as_of: {type: date}
weird: &anchor value
";
        assert_eq!(Fields::parse(text).render(), text);
    }

    #[test]
    fn values_parse_from_the_subset_real_farms_use() {
        let f = Fields::parse(
            "\
title: 'It''s: here'
priority: 3
labels:
- ui
- 'a, b'
tags: [subtext, \"signals\", forms]
generated: {by: agent:joel@joel, at: 2026-10-28T16:14:47Z}
sources:
  - id: file
    resource: \"/tmp/x.png\"   # comment
    as_of: 2025-11-25
  - raw/sources/a/source.md
fields:
  platform: {type: text, description: Product name (the row key)}
empty:
note: |
  line one
  line two
",
        );
        assert_eq!(f.get("title"), Some(Value::Str(p("It's: here"))));
        assert_eq!(f.get("priority"), Some(Value::Plain(p("3"))));
        assert_eq!(
            f.get("labels"),
            Some(Value::List(vec![
                Value::Plain(p("ui")),
                Value::Str(p("a, b"))
            ]))
        );
        assert_eq!(
            f.get("tags").unwrap().to_json(),
            json!(["subtext", "signals", "forms"])
        );
        assert_eq!(
            f.get("generated").unwrap().to_json(),
            json!({"by": "agent:joel@joel", "at": "2026-10-28T16:14:47Z"})
        );
        assert_eq!(
            f.get("sources").unwrap().to_json(),
            json!([{"id": "file", "resource": "/tmp/x.png", "as_of": "2025-11-25"}, "raw/sources/a/source.md"])
        );
        assert_eq!(
            f.get("fields").unwrap().to_json(),
            json!({"platform": {"type": "text", "description": "Product name (the row key)"}})
        );
        assert_eq!(f.get("empty"), Some(Value::Null));
        assert_eq!(f.get("note"), Some(Value::Str(p("line one\nline two\n"))));
        assert_eq!(f.to_json()["priority"], json!(3));
    }

    #[test]
    fn unsupported_yaml_round_trips_without_a_value() {
        let f = Fields::parse("a: &x 1\nb: *x\n");
        assert_eq!(f.get("a"), None);
        assert_eq!(f.get("b"), None);
        assert_eq!(f.render(), "a: &x 1\nb: *x\n");
    }

    #[test]
    fn set_rewrites_only_that_entry_and_appends_new_keys() {
        let mut f = Fields::parse("a: 1\nlist:\n- x\n# keep me\nb: 'two'\n");
        f.set("a", &Value::Plain(p("5")));
        f.set(
            "commit",
            &Value::List(vec![Value::Plain(p("3f2a9c1")), Value::Plain(p("e99754d"))]),
        );
        f.set("due", &Value::Plain(p("2026-11-05")));
        assert_eq!(
            f.render(),
            "a: 5\nlist:\n- x\n# keep me\nb: 'two'\ncommit:\n- 3f2a9c1\n- e99754d\ndue: 2026-11-05\n"
        );
        assert!(f.remove("list"));
        assert!(!f.remove("list"));
        assert_eq!(
            f.render(),
            "a: 5\n# keep me\nb: 'two'\ncommit:\n- 3f2a9c1\n- e99754d\ndue: 2026-11-05\n"
        );
    }

    #[test]
    fn written_values_read_back_equal() {
        let cases = vec![
            Value::Plain(p("plain")),
            Value::Str(p("needs: quotes")),
            Value::Str(p("it's")),
            Value::Str(p("two\nlines")),
            Value::Str(p("true")),
            Value::List(vec![Value::Plain(p("a")), Value::Str(p("b, c"))]),
            Value::Map(vec![
                (p("by"), Value::Plain(p("human:joel"))),
                (p("at"), Value::Plain(p("2026-11-01T00:00:00Z"))),
            ]),
            Value::List(vec![
                Value::Map(vec![
                    (p("id"), Value::Plain(p("file"))),
                    (
                        p("resource"),
                        Value::Str(p(
                            "/a long path/that goes on and on and on/and further still/to make it wrap.png",
                        )),
                    ),
                ]),
                Value::Plain(p("x")),
            ]),
            Value::Map(vec![
                (p("nested"), Value::List(vec![Value::Plain(p("a"))])),
                (
                    p("deeper"),
                    Value::Map(vec![(
                        p("k"),
                        Value::List(vec![Value::Map(vec![(p("q"), Value::Plain(p("1")))])]),
                    )]),
                ),
            ]),
        ];
        for v in cases {
            let mut f = Fields::default();
            f.set("k", &v);
            let back = Fields::parse(&f.render()).get("k");
            assert_eq!(
                back.as_ref().map(Value::to_json),
                Some(v.to_json()),
                "rendered:\n{}",
                f.render()
            );
        }
    }

    #[test]
    fn cli_values() {
        assert_eq!(Value::from_cli(""), Value::Null);
        assert_eq!(Value::from_cli("2026-11-05"), Value::Plain(p("2026-11-05")));
        assert_eq!(
            Value::from_cli("[a, b]"),
            Value::List(vec![Value::Plain(p("a")), Value::Plain(p("b"))])
        );
        assert_eq!(Value::from_cli("[unclosed"), Value::Plain(p("[unclosed")));
    }

    #[test]
    fn a_hash_inside_a_value_is_not_a_comment() {
        let f = Fields::parse("url: https://x.test/a#frag\nq: 'a # b'\nc: x # gone\n");
        assert_eq!(f.get("url"), Some(Value::Plain(p("https://x.test/a#frag"))));
        assert_eq!(f.get("q"), Some(Value::Str(p("a # b"))));
        assert_eq!(f.get("c"), Some(Value::Plain(p("x"))));
    }
}
