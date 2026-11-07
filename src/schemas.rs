//! Schemas as data (yaks-8e75, design yaks-8384): `schemas/<type>.md` says
//! which fields a typed dok has, in Hearth's format, so the agent and the
//! tools know what to expect. Checks only warn; a write never fails on them.
//!
//! ```markdown
//! ---
//! type: Dok Type
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
use anyhow::{Context, Result, bail};
use std::path::Path;

/// The schemas directory at the farm root.
pub const SCHEMAS_DIR: &str = "schemas";

/// The type name of the schema that extends yaks.
pub const YAK_TYPE: &str = "yak";

const KINDS: &[&str] = &[
    "text", "number", "bool", "date", "datetime", "money", "link", "list",
];

/// Keys any dok may carry without its schema listing them (OKF's and
/// Hearth's bookkeeping, and the `created:` stamp of `yaks dok new`).
const META: &[&str] = &[
    "type",
    "title",
    "created",
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
    /// How long a dok of this type stays current (`P6M`, `P1Y`, `P2W`, or a
    /// date); a dok's own `stale_after` wins. `yaks check` reports doks past it.
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
/// for doks, OKF's bookkeeping keys).
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

/// Warnings for a dok against its type's schema (none for an untyped kind
/// or a type without a schema).
pub fn check_dok(root: &Path, dok: &crate::doks::Dok) -> Vec<String> {
    let kind = dok.kind();
    match load(root, &kind) {
        Some(s) => check(&s, &dok.fields, META),
        None => Vec::new(),
    }
}

/// The defaults a new dok of `type_name` gets for fields not given.
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

/// Known kinds of untyped dok and every typed kind with a schema, with its
/// fields, for `yaks dok types`.
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

/// Rows of a table dok: the first markdown table in the body, one map per
/// row keyed by the header, cells trimmed (Hearth's table doks). The rows
/// are written by [`crate::rows`], which this reads through.
pub fn table_rows(body: &str) -> Vec<Vec<(String, String)>> {
    crate::rows::table_rows(body)
}

/// One `--field name=type[!]` of `yaks dok define`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: String,
    pub kind: String,
    pub required: bool,
}

impl FieldSpec {
    /// Parse `name=type` or `name=type!` (`!` marks the field required).
    pub fn parse(text: &str) -> Result<FieldSpec> {
        let Some((name, kind)) = text.split_once('=') else {
            bail!("--field {text}: write it as name=type, or name=type! for a required field");
        };
        let (name, kind) = (name.trim(), kind.trim());
        let (kind, required) = match kind.strip_suffix('!') {
            Some(k) => (k.trim_end(), true),
            None => (kind, false),
        };
        let name_ok = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !name_ok {
            bail!(
                "--field {text}: field name {name:?} should be letters, digits, `_` and `-`, starting with a letter"
            );
        }
        if !KINDS.contains(&kind) {
            bail!(
                "--field {text}: type {kind:?} is not one yaks knows ({})",
                KINDS.join(", ")
            );
        }
        Ok(FieldSpec {
            name: name.to_string(),
            kind: kind.to_string(),
            required,
        })
    }

    fn describe(&self) -> String {
        format!(
            "{}{}",
            self.kind,
            if self.required { " (required)" } else { "" }
        )
    }

    /// How a schema file writes this field: the bare type name, or a map
    /// when required.
    fn value(&self) -> Value {
        if self.required {
            Value::Map(vec![
                ("type".into(), Value::Plain(self.kind.clone())),
                ("required".into(), Value::Plain("true".into())),
            ])
        } else {
            Value::Plain(self.kind.clone())
        }
    }
}

/// What `yaks dok define` was asked to write.
#[derive(Debug, Clone, Default)]
pub struct Define {
    pub kind: String,
    pub fields: Vec<FieldSpec>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub stale_after: Option<String>,
    /// Let a field's type or required flag change.
    pub replace: bool,
}

/// What `define` did to `schemas/<kind>.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Defined {
    pub path: String,
    pub created: bool,
    pub added: Vec<String>,
    pub changed: Vec<String>,
    /// Top-level keys (`title`, `description`, `stale_after`) written.
    pub keys: Vec<String>,
}

impl Defined {
    pub fn is_noop(&self) -> bool {
        !self.created && self.added.is_empty() && self.changed.is_empty() && self.keys.is_empty()
    }
}

/// The type and required flag a field's value in a schema file states.
fn spec_of(v: &Value) -> (String, bool) {
    match v {
        Value::Map(_) => (
            v.get("type")
                .and_then(Value::as_str)
                .unwrap_or("text")
                .to_string(),
            v.get("required").and_then(Value::as_str) == Some("true"),
        ),
        Value::Plain(k) | Value::Str(k) => (k.clone(), false),
        _ => ("text".into(), false),
    }
}

/// `v` with its type and required flag set to `spec`'s, keeping any default
/// and description.
fn respec(v: &Value, spec: &FieldSpec) -> Value {
    let Value::Map(entries) = v else {
        return spec.value();
    };
    let mut out: Vec<(String, Value)> = Vec::new();
    let mut had_type = false;
    for (k, ev) in entries {
        match k.as_str() {
            "type" => {
                had_type = true;
                out.push((k.clone(), Value::Plain(spec.kind.clone())));
            }
            "required" if spec.required => {
                out.push((k.clone(), Value::Plain("true".into())));
            }
            "required" => {}
            _ => out.push((k.clone(), ev.clone())),
        }
    }
    if !had_type {
        out.insert(0, ("type".into(), Value::Plain(spec.kind.clone())));
    }
    if spec.required && !out.iter().any(|(k, _)| k == "required") {
        let at = out
            .iter()
            .position(|(k, _)| k == "type")
            .map_or(0, |i| i + 1);
        out.insert(at, ("required".into(), Value::Plain("true".into())));
    }
    Value::Map(out)
}

/// Write or merge `schemas/<kind>.md` (the caller holds the farm lock).
/// A new file gets `type: Dok Type`, a title and the fields. An existing one
/// keeps every field, every other key and its prose: new fields are added,
/// a field whose type or required flag differs is refused unless
/// `def.replace` (its default and description stay), and `title`,
/// `description` and `stale_after` are set only when given.
pub fn define(root: &Path, def: &Define) -> Result<Defined> {
    if !valid_type(&def.kind) {
        bail!(
            "{:?} is not a kind name: use lowercase letters, digits and `-`, starting with a letter",
            def.kind
        );
    }
    if crate::doks::UNTYPED_KINDS.contains(&def.kind.as_str()) {
        bail!(
            "{} is one of the untyped kinds ({}); define a kind of your own",
            def.kind,
            crate::doks::UNTYPED_KINDS.join(", ")
        );
    }
    if let Some(a) = &def.stale_after {
        let date = chrono::NaiveDate::parse_from_str(a.trim(), "%Y-%m-%d").is_ok();
        if !date && crate::recur::parse_duration(a.trim()).is_none() {
            bail!(
                "--stale-after {a}: use a duration such as P6M, P1Y or P2W, or a date YYYY-MM-DD"
            );
        }
    }
    if def.fields.is_empty()
        && def.title.is_none()
        && def.description.is_none()
        && def.stale_after.is_none()
    {
        bail!(
            "nothing to define: give --field name=type (repeatable), --title, --description or --stale-after"
        );
    }
    for (i, f) in def.fields.iter().enumerate() {
        if def.fields[..i].iter().any(|g| g.name == f.name) {
            bail!("--field {} is given twice", f.name);
        }
    }

    let rel = format!("{SCHEMAS_DIR}/{}.md", def.kind);
    let path = root.join(&rel);
    let existing = match std::fs::read_to_string(&path) {
        Ok(t) => Some(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("reading {rel}")),
    };
    let mut done = Defined {
        path: rel.clone(),
        created: existing.is_none(),
        ..Defined::default()
    };
    let mut page = match &existing {
        Some(t) => {
            let p = crate::doks::Dok::parse(&rel, t);
            if !p.has_frontmatter {
                bail!("{rel} has no frontmatter; fix it by hand first");
            }
            p
        }
        None => {
            let mut f = Fields::default();
            f.set("type", &Value::Plain("Dok Type".into()));
            let title = def.title.clone().unwrap_or_else(|| {
                let mut t = def.kind.replace('-', " ");
                if let Some(c) = t.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                t
            });
            f.set("title", &Value::Str(title));
            crate::doks::Dok {
                path: rel.clone(),
                fields: f,
                body: String::new(),
                has_frontmatter: true,
            }
        }
    };
    for (key, given) in [
        ("title", &def.title),
        ("description", &def.description),
        ("stale_after", &def.stale_after),
    ] {
        let Some(v) = given.as_deref().map(str::trim) else {
            continue;
        };
        if page
            .fields
            .get(key)
            .and_then(|c| c.as_str().map(String::from))
            .as_deref()
            == Some(v)
        {
            continue;
        }
        let value = if key == "stale_after" {
            Value::Plain(v.to_string())
        } else {
            Value::Str(v.to_string())
        };
        page.fields.set(key, &value);
        if existing.is_some() {
            done.keys.push(key.to_string());
        }
    }

    // The fields as the file has them now.
    let mut current: Vec<(String, Value)> = if !page.fields.contains("fields") {
        Vec::new()
    } else {
        match page.fields.get("fields") {
            Some(Value::Null) => Vec::new(),
            Some(Value::Map(m)) => m,
            _ => bail!("{rel}: `fields:` is not a map of field name to type; fix it by hand first"),
        }
    };
    let mut conflicts = Vec::new();
    let mut fresh: Vec<(String, Value)> = Vec::new();
    for spec in &def.fields {
        match current.iter_mut().find(|(n, _)| *n == spec.name) {
            None => {
                fresh.push((spec.name.clone(), spec.value()));
                done.added.push(spec.name.clone());
            }
            Some((_, v)) => {
                let (kind, required) = spec_of(v);
                if kind == spec.kind && required == spec.required {
                    continue;
                }
                if def.replace {
                    *v = respec(v, spec);
                    done.changed.push(spec.name.clone());
                } else {
                    let was = FieldSpec {
                        name: spec.name.clone(),
                        kind,
                        required,
                    };
                    conflicts.push(format!(
                        "{} is {} here, not {}",
                        spec.name,
                        was.describe(),
                        spec.describe()
                    ));
                }
            }
        }
    }
    if !conflicts.is_empty() {
        bail!(
            "{rel} already has: {}; nothing written. Pass --replace to change a field's type",
            conflicts.join("; ")
        );
    }
    if !done.changed.is_empty() || !page.fields.contains("fields") {
        current.extend(fresh);
        if !current.is_empty() {
            page.fields.set("fields", &Value::Map(current));
        }
    } else if !fresh.is_empty() {
        // Only additions: keep the existing lines as they are, append the new.
        let raw: Vec<String> = page
            .fields
            .raw("fields")
            .map(<[_]>::to_vec)
            .unwrap_or_default();
        let block = raw.first().is_some_and(|l| l.trim_end() == "fields:")
            && raw[1..]
                .iter()
                .all(|l| l.starts_with(' ') || l.trim().is_empty());
        if block {
            let mut lines = raw;
            for (name, v) in &fresh {
                let mut one = Fields::default();
                one.set(name, v);
                lines.extend(one.lines().map(|l| format!("  {l}")));
            }
            page.fields.set_raw("fields", lines);
        } else {
            current.extend(fresh);
            page.fields.set("fields", &Value::Map(current));
        }
    }

    let new = page.render();
    if existing.as_deref() != Some(new.as_str()) {
        std::fs::create_dir_all(root.join(SCHEMAS_DIR))
            .with_context(|| format!("creating {SCHEMAS_DIR}/"))?;
        crate::store::write::atomic_write(&path, &new)?;
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    const APPLIANCE: &str = "---\ntype: Dok Type\ntitle: Appliance\nfields:\n  name: {type: text, required: true}\n  purchased: {type: date}\n  price: money\n  warranty: {type: text, default: none}\n  odd: {type: colour}\n---\n\nNotes.\n";

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
            "---\ntype: Dok Type\nfields:\n  due: date\ntags:\n  ui: The TUI\n  cli: Commands and flags\n---\n",
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
