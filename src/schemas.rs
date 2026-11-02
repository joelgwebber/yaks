//! Schemas as data (yaks-8e75, design yaks-8384): `schemas/<type>.md` says
//! which fields a typed page has, in Hearth's format, so the agent and the
//! tools know what to expect. Checks only warn; a write never fails on them.
//!
//! ```markdown
//! ---
//! type: Page Type
//! title: Appliance
//! fields:
//!   name: {type: text, required: true}
//!   purchased: {type: date}
//!   warranty: {type: text, default: none}
//! ---
//! ```
//!
//! A field is `{type, required, default, description}` or the shorthand
//! `name: text`. Kinds: text, number, bool, date, datetime, money, link,
//! list; anything else reads as text, with a warning.
//!
//! `schemas/yak.md` extends the yak: its fields are extra yak fields (`due`,
//! `commit`), never the built-in ones, and its `tags:` map lists the farm's
//! known tags with descriptions (yaks-640a, yaks-f04d).

use crate::fields::{Fields, Value};
use std::path::Path;

/// The schemas directory at the farm root.
pub const SCHEMAS_DIR: &str = "schemas";

/// The type name of the schema that extends yaks.
pub const YAK_TYPE: &str = "yak";

const KINDS: &[&str] = &[
    "text", "number", "bool", "date", "datetime", "money", "link", "list",
];

/// Keys any page may carry without its schema listing them (OKF's and
/// Hearth's bookkeeping).
const META: &[&str] = &[
    "type",
    "title",
    "description",
    "tags",
    "status",
    "stale_after",
    "sources",
    "generated",
    "verified",
    "history",
    "provenance",
    "superseded_by",
    "id",
];

/// One field a schema declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub kind: String,
    pub required: bool,
    pub default: Option<Value>,
    pub description: Option<String>,
}

/// A parsed schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub type_name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    /// `schemas/yak.md` only: known tag -> description.
    pub tags: Vec<(String, String)>,
    /// How long a page of this type stays current (`P6M`, `P1Y`, `P2W`, or a
    /// date); a page's own `stale_after` wins. `yaks check` reports pages past it.
    pub stale_after: Option<String>,
    /// Problems with the schema file itself.
    pub warnings: Vec<String>,
}

/// Whether `name` is a valid type name: lowercase letters, digits and `-`,
/// starting with a letter.
pub fn valid_type(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Parse a schema file's text.
pub fn parse(type_name: &str, text: &str) -> Schema {
    let fields_of = |t: &str| match crate::store::split_frontmatter(t) {
        Some((f, _)) => Fields::parse(f),
        None => Fields::default(),
    };
    let f = fields_of(text);
    let mut s = Schema {
        type_name: type_name.to_string(),
        title: f.get("title").and_then(|v| v.as_str().map(String::from)),
        description: f
            .get("description")
            .and_then(|v| v.as_str().map(String::from)),
        fields: Vec::new(),
        tags: Vec::new(),
        stale_after: f
            .get("stale_after")
            .and_then(|v| v.as_str().map(String::from))
            .filter(|s| !s.is_empty()),
        warnings: Vec::new(),
    };
    match f.get("fields") {
        None | Some(Value::Null) => {}
        Some(Value::Map(entries)) => {
            for (name, spec) in entries {
                let mut field = Field {
                    name: name.clone(),
                    kind: "text".into(),
                    required: false,
                    default: None,
                    description: None,
                };
                match &spec {
                    Value::Map(_) => {
                        if let Some(k) = spec.get("type").and_then(Value::as_str) {
                            field.kind = k.to_string();
                        }
                        field.required =
                            spec.get("required").and_then(Value::as_str) == Some("true");
                        field.default = spec.get("default").filter(|v| **v != Value::Null).cloned();
                        field.description = spec
                            .get("description")
                            .and_then(|v| v.as_str().map(String::from));
                    }
                    Value::Plain(k) | Value::Str(k) => field.kind = k.clone(),
                    Value::Null => {}
                    Value::List(_) => s.warnings.push(format!(
                        "field {name}: expected {{type: ...}} or a type name"
                    )),
                }
                if !KINDS.contains(&field.kind.as_str()) {
                    s.warnings.push(format!(
                        "field {name}: type {:?} is not one yaks knows ({}); read as text",
                        field.kind,
                        KINDS.join(", ")
                    ));
                }
                s.fields.push(field);
            }
        }
        Some(_) => s.warnings.push(
            "fields: should be a map of field name to {type, required, default, description}"
                .into(),
        ),
    }
    if let Some(Value::Map(entries)) = f.get("tags") {
        s.tags = entries
            .into_iter()
            .map(|(k, v)| (k, v.as_str().map(String::from).unwrap_or_default()))
            .collect();
    }
    s
}

/// The schema for `type_name` in the farm at `root`, if there is one.
pub fn load(root: &Path, type_name: &str) -> Option<Schema> {
    if !valid_type(type_name) {
        return None;
    }
    let text =
        std::fs::read_to_string(root.join(SCHEMAS_DIR).join(format!("{type_name}.md"))).ok()?;
    Some(parse(type_name, &text))
}

/// Every schema in the farm, sorted by type.
pub fn load_all(root: &Path) -> Vec<Schema> {
    let mut out: Vec<Schema> = std::fs::read_dir(root.join(SCHEMAS_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_str()?.strip_suffix(".md")?.to_string();
            load(root, &n)
        })
        .collect();
    out.sort_by(|a, b| a.type_name.cmp(&b.type_name));
    out
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: std::ops::Range<usize>| {
        b.get(r.clone())
            .is_some_and(|x| x.iter().all(u8::is_ascii_digit))
    };
    (b.len() == 10 && digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10))
        || (b.len() == 7 && digits(0..4) && b[4] == b'-' && digits(5..7))
}

/// Why `v` does not fit `kind`, or `None` when it does.
fn kind_problem(kind: &str, v: &Value) -> Option<String> {
    let s = v.as_str();
    match kind {
        "number" => {
            (s.is_none_or(|s| s.parse::<f64>().is_err())).then(|| "expected a number".into())
        }
        "bool" => (!matches!(s, Some("true" | "false"))).then(|| "expected true or false".into()),
        "date" => (!s.is_some_and(is_date)).then(|| "expected a date, YYYY-MM-DD".into()),
        "datetime" => (!s.is_some_and(|s| {
            crate::store::parse_ts(s).is_some() || chrono::DateTime::parse_from_rfc3339(s).is_ok()
        }))
        .then(|| "expected a date and time, e.g. 2026-11-03T15:30:00Z".into()),
        "money" => (!s.is_some_and(|s| {
            s.split_once(' ').is_some_and(|(n, c)| {
                n.parse::<f64>().is_ok()
                    && c.len() == 3
                    && c.chars().all(|c| c.is_ascii_uppercase())
            })
        }))
        .then(|| "expected an amount and currency, e.g. 649.00 USD".into()),
        "list" => (!matches!(v, Value::List(_))).then(|| "expected a list".into()),
        _ => None,
    }
}

/// Warnings for a document's fields against `schema`. `extra_ok` names keys
/// that may appear without the schema listing them (the yak's built-ins;
/// for pages, OKF's bookkeeping keys).
pub fn check(schema: &Schema, fields: &Fields, extra_ok: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for f in &schema.fields {
        match fields.get(&f.name) {
            None | Some(Value::Null) => {
                if f.required {
                    out.push(format!(
                        "{} is usually set for type {} and is missing",
                        f.name, schema.type_name
                    ));
                }
            }
            Some(v) => {
                if let Some(p) = kind_problem(&f.kind, &v) {
                    out.push(format!("{}: {p}", f.name));
                }
            }
        }
    }
    let undeclared: Vec<&str> = fields
        .keys()
        .filter(|k| !extra_ok.contains(k) && !schema.fields.iter().any(|f| f.name == *k))
        .collect();
    if !undeclared.is_empty() {
        out.push(format!(
            "fields the {} schema does not list: {} (add them to schemas/{}.md if they are useful)",
            schema.type_name,
            undeclared.join(", "),
            schema.type_name
        ));
    }
    out
}

/// Warnings for a page against its type's schema (none for an untyped kind
/// or a type without a schema).
pub fn check_page(root: &Path, page: &crate::pages::Page) -> Vec<String> {
    let kind = page.kind();
    match load(root, &kind) {
        Some(s) => check(&s, &page.fields, META),
        None => Vec::new(),
    }
}

/// The defaults a new page of `type_name` gets for fields not given.
pub fn defaults(root: &Path, type_name: &str) -> Vec<(String, Value)> {
    load(root, type_name)
        .map(|s| {
            s.fields
                .into_iter()
                .filter_map(|f| Some((f.name, f.default?)))
                .collect()
        })
        .unwrap_or_default()
}

/// Known kinds of untyped page and every typed kind with a schema, with its
/// fields, for `yaks page types`.
pub fn describe(s: &Schema) -> String {
    let mut out = s.type_name.clone();
    if let Some(t) = s.title.as_ref().filter(|t| **t != s.type_name) {
        out.push_str(&format!(" ({t})"));
    }
    if let Some(d) = &s.description {
        out.push_str(&format!(": {d}"));
    }
    for f in &s.fields {
        out.push_str(&format!("\n    {}: {}", f.name, f.kind));
        if f.required {
            out.push_str(", usually set");
        }
        if let Some(d) = &f.default {
            out.push_str(&format!(", default {}", d.as_str().unwrap_or("(a value)")));
        }
        if let Some(d) = &f.description {
            out.push_str(&format!(" — {d}"));
        }
    }
    for (t, d) in &s.tags {
        out.push_str(&format!("\n    tag {t}: {d}"));
    }
    out
}

/// Rows of a table page: the first markdown table in the body, one map per
/// row keyed by the header, cells trimmed (Hearth's table pages).
pub fn table_rows(body: &str) -> Vec<Vec<(String, String)>> {
    let lines: Vec<&str> = body.lines().map(str::trim).collect();
    let Some(start) = lines.iter().position(|l| l.starts_with('|')) else {
        return Vec::new();
    };
    let cells = |l: &str| -> Vec<String> {
        l.trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect()
    };
    let header = cells(lines[start]);
    let sep = lines
        .get(start + 1)
        .is_some_and(|l| l.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')));
    if !sep {
        return Vec::new();
    }
    lines[start + 2..]
        .iter()
        .take_while(|l| l.starts_with('|'))
        .map(|l| header.iter().cloned().zip(cells(l)).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const APPLIANCE: &str = "---\ntype: Page Type\ntitle: Appliance\nfields:\n  name: {type: text, required: true}\n  purchased: {type: date}\n  price: money\n  warranty: {type: text, default: none}\n  odd: {type: colour}\n---\n\nNotes.\n";

    #[test]
    fn parses_hearths_format() {
        let s = parse("appliance", APPLIANCE);
        let names: Vec<(&str, &str, bool)> = s
            .fields
            .iter()
            .map(|f| (f.name.as_str(), f.kind.as_str(), f.required))
            .collect();
        assert_eq!(
            names,
            [
                ("name", "text", true),
                ("purchased", "date", false),
                ("price", "money", false),
                ("warranty", "text", false),
                ("odd", "colour", false)
            ]
        );
        assert_eq!(s.fields[3].default, Some(Value::Plain("none".into())));
        assert_eq!(s.warnings.len(), 1, "{:?}", s.warnings);
    }

    #[test]
    fn checks_warn_on_missing_wrong_and_undeclared() {
        let s = parse("appliance", APPLIANCE);
        let f = Fields::parse(
            "type: appliance\ntitle: Dishwasher\npurchased: last spring\nprice: 649.00 USD\ncolor: white\n",
        );
        assert_eq!(
            check(&s, &f, META),
            [
                "name is usually set for type appliance and is missing",
                "purchased: expected a date, YYYY-MM-DD",
                "fields the appliance schema does not list: color (add them to schemas/appliance.md if they are useful)",
            ]
        );
    }

    #[test]
    fn yak_schema_carries_known_tags() {
        let s = parse(
            "yak",
            "---\ntype: Page Type\nfields:\n  due: date\ntags:\n  ui: The TUI\n  cli: Commands and flags\n---\n",
        );
        assert_eq!(
            s.tags,
            [
                ("ui".to_string(), "The TUI".to_string()),
                ("cli".into(), "Commands and flags".into())
            ]
        );
        assert_eq!(s.fields[0].kind, "date");
    }

    #[test]
    fn table_rows_follow_the_header() {
        let body = "Intro.\n\n| platform | vendor |\n| --- | --- |\n| cursor | Anysphere |\n| glean | Glean |\n\nAfter.\n";
        let rows = table_rows(body);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1],
            [
                ("platform".to_string(), "glean".to_string()),
                ("vendor".into(), "Glean".into())
            ]
        );
    }
}
