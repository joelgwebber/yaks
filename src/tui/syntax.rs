//! Tiny per-language tokenizers for the code inside fenced code blocks
//! (yaks-3020), shared by the detail pane's renderer (`mdrender`) and the
//! editor's highlighter (`markdown`).
//!
//! Pure Rust, no dependencies (no syntect/onig: the release matrix stays
//! C-free). This is deliberately *not* a parser: one generic lexer driven by a
//! per-language [`Spec`] (keywords, comment and string syntax) that labels each
//! char as keyword / string / number / comment / type / punctuation / plain.
//! Wrong-but-harmless beats exact: an odd construct only costs a colour.
//!
//! One line at a time, with a [`State`] carried to the next line for the only
//! multi-line constructs that matter (block comments, multi-line strings), so
//! the editor can feed it logical lines in order exactly as it feeds markdown.
//! Runs are in *char* coordinates within the line, like every other span here.
//! The styles come from the palette's token section.

use ratatui::style::Style;

use super::palette;

/// A language we can colour. `None` from [`Lang::from_info`] means "unknown":
/// the caller keeps the flat code style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Sh,
    Json,
    Toml,
    Yaml,
    Python,
    Js,
    Java,
}

impl Lang {
    /// The language named by a fence's info string (`rust`, `rust,no_run`,
    /// `{.sh}`, `Bash title="x"`): the first word, case-insensitive, with the
    /// common aliases.
    pub fn from_info(info: &str) -> Option<Lang> {
        let word = info
            .trim()
            .trim_start_matches(['{', '.'])
            .split(|c: char| c.is_whitespace() || matches!(c, ',' | '{' | '}'))
            .next()?
            .to_ascii_lowercase();
        Some(match word.as_str() {
            "rust" | "rs" => Lang::Rust,
            "sh" | "bash" | "shell" | "zsh" | "console" | "shell-session" => Lang::Sh,
            "json" | "jsonc" | "json5" => Lang::Json,
            "toml" => Lang::Toml,
            "yaml" | "yml" => Lang::Yaml,
            "python" | "py" | "python3" => Lang::Python,
            "js" | "javascript" | "jsx" | "ts" | "typescript" | "tsx" | "mjs" => Lang::Js,
            "java" => Lang::Java,
            _ => return None,
        })
    }

    fn spec(self) -> &'static Spec {
        match self {
            Lang::Rust => &RUST,
            Lang::Sh => &SH,
            Lang::Json => &JSON,
            Lang::Toml => &TOML,
            Lang::Yaml => &YAML,
            Lang::Python => &PYTHON,
            Lang::Js => &JS,
            Lang::Java => &JAVA,
        }
    }
}

/// What a char is. `Plain` takes the caller's base style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Plain,
    Keyword,
    Str,
    Number,
    Comment,
    Type,
    Punct,
}

impl Kind {
    fn style(self, base: Style) -> Style {
        match self {
            Kind::Plain => base,
            Kind::Keyword => palette::syntax_keyword(),
            Kind::Str => palette::syntax_string(),
            Kind::Number => palette::syntax_number(),
            Kind::Comment => palette::syntax_comment(),
            Kind::Type => palette::syntax_type(),
            Kind::Punct => palette::syntax_punct(),
        }
    }
}

/// Where a line ends, for the next one: inside a block comment (with its
/// nesting depth) or a multi-line string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    Normal,
    Block(u8),
    Str {
        quote: char,
        triple: bool,
    },
}

/// Lexer state carried from one line to the next. `State::default()` starts a
/// block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State(Mode);

/// A styled run of chars within one line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub start: usize,
    pub len: usize,
    pub style: Style,
}

/// How a language's `key` positions are recognised (they read as `Type`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keys {
    None,
    /// A string followed by `:`.
    Json,
    /// `[table]` headers and `key =` at the start of a line.
    Toml,
    /// `key:` (optionally after `- `) at the start of a line.
    Yaml,
}

struct Spec {
    keywords: &'static [&'static str],
    /// Built-in type names (besides the capitalised-identifier rule).
    types: &'static [&'static str],
    capitalised_types: bool,
    line_comments: &'static [&'static str],
    /// `#` only starts a comment at a word boundary (shell, yaml).
    hash_needs_boundary: bool,
    block_comment: Option<(&'static str, &'static str)>,
    nested_blocks: bool,
    quotes: &'static [char],
    /// Quotes whose strings carry over to the next line.
    multiline: &'static [char],
    /// `"""` / `'''` strings.
    triple: bool,
    /// `'a` is a lifetime unless it closes like a char literal.
    lifetimes: bool,
    /// `$name` and `${...}`.
    dollar_vars: bool,
    keys: Keys,
}

const BASE: Spec = Spec {
    keywords: &[],
    types: &[],
    capitalised_types: false,
    line_comments: &[],
    hash_needs_boundary: false,
    block_comment: None,
    nested_blocks: false,
    quotes: &['"', '\''],
    multiline: &[],
    triple: false,
    lifetimes: false,
    dollar_vars: false,
    keys: Keys::None,
};

const RUST: Spec = Spec {
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true",
        "type", "unsafe", "use", "where", "while",
    ],
    types: &[
        "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
        "u16", "u32", "u64", "u128", "usize",
    ],
    capitalised_types: true,
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    nested_blocks: true,
    multiline: &['"'],
    lifetimes: true,
    ..BASE
};

const SH: Spec = Spec {
    keywords: &[
        "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
        "in", "function", "return", "exit", "local", "export", "source", "cd", "echo", "set",
        "unset", "true", "false",
    ],
    line_comments: &["#"],
    hash_needs_boundary: true,
    multiline: &['"', '\''],
    dollar_vars: true,
    ..BASE
};

const JSON: Spec = Spec {
    keywords: &["true", "false", "null"],
    quotes: &['"'],
    // jsonc / json5 comments are harmless to accept.
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    keys: Keys::Json,
    ..BASE
};

const TOML: Spec = Spec {
    keywords: &["true", "false"],
    line_comments: &["#"],
    multiline: &[],
    triple: true,
    keys: Keys::Toml,
    ..BASE
};

const YAML: Spec = Spec {
    keywords: &["true", "false", "null", "yes", "no", "on", "off"],
    line_comments: &["#"],
    hash_needs_boundary: true,
    keys: Keys::Yaml,
    ..BASE
};

const PYTHON: Spec = Spec {
    keywords: &[
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
        "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
        "with", "yield", "True", "False",
    ],
    types: &[
        "int", "str", "float", "bool", "list", "dict", "set", "tuple", "bytes",
    ],
    capitalised_types: true,
    line_comments: &["#"],
    triple: true,
    ..BASE
};

const JS: Spec = Spec {
    keywords: &[
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "from",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "interface",
        "let",
        "new",
        "null",
        "of",
        "return",
        "static",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "type",
        "typeof",
        "undefined",
        "var",
        "void",
        "while",
        "yield",
    ],
    types: &["string", "number", "boolean", "any", "unknown", "never"],
    capitalised_types: true,
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\'', '`'],
    multiline: &['`'],
    ..BASE
};

const JAVA: Spec = Spec {
    keywords: &[
        "abstract",
        "break",
        "case",
        "catch",
        "class",
        "continue",
        "default",
        "do",
        "else",
        "enum",
        "extends",
        "false",
        "final",
        "finally",
        "for",
        "if",
        "implements",
        "import",
        "instanceof",
        "interface",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "static",
        "super",
        "switch",
        "this",
        "throw",
        "throws",
        "true",
        "try",
        "var",
        "void",
        "while",
    ],
    types: &[
        "boolean", "byte", "char", "double", "float", "int", "long", "short", "String",
    ],
    capitalised_types: true,
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    triple: true,
    ..BASE
};

fn starts_with(chars: &[char], at: usize, pat: &str) -> bool {
    let mut i = at;
    for p in pat.chars() {
        if chars.get(i) != Some(&p) {
            return false;
        }
        i += 1;
    }
    true
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// End of the identifier starting at `at`.
fn word_end(chars: &[char], at: usize) -> usize {
    let mut i = at;
    while chars.get(i).is_some_and(|c| is_ident(*c)) {
        i += 1;
    }
    i
}

/// Scan a string body from `from` for its closing quote; returns the index
/// after it (or the line's end) and whether it closed.
fn scan_string(chars: &[char], from: usize, quote: char, triple: bool) -> (usize, bool) {
    let mut i = from;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == quote {
            if !triple {
                return (i + 1, true);
            }
            if chars.get(i + 1) == Some(&quote) && chars.get(i + 2) == Some(&quote) {
                return (i + 3, true);
            }
        }
        i += 1;
    }
    (chars.len(), false)
}

/// Scan a block comment body from `from` at nesting `depth`; returns the index
/// after the text consumed and the depth still open (0 = closed).
fn scan_block(chars: &[char], from: usize, mut depth: u8, spec: &Spec) -> (usize, u8) {
    let (open, close) = spec.block_comment.expect("block comment spec");
    let mut i = from;
    while i < chars.len() && depth > 0 {
        if starts_with(chars, i, close) {
            depth -= 1;
            i += close.chars().count();
        } else if spec.nested_blocks && starts_with(chars, i, open) {
            depth = depth.saturating_add(1);
            i += open.chars().count();
        } else {
            i += 1;
        }
    }
    (i, depth)
}

fn fill(kinds: &mut [Kind], from: usize, to: usize, kind: Kind) {
    let n = kinds.len();
    for k in &mut kinds[from.min(n)..to.min(n)] {
        *k = kind;
    }
}

/// Keys recognised at the very start of a line (toml, yaml): marks them and
/// returns where the ordinary lexing should resume.
fn line_start_keys(chars: &[char], kinds: &mut [Kind], keys: Keys) -> usize {
    let mut i = chars
        .iter()
        .take_while(|c| **c == ' ' || **c == '\t')
        .count();
    match keys {
        Keys::Toml if chars.get(i) == Some(&'[') => {
            // `[table]` / `[[array]]`, up to the last `]` before any comment.
            let stop = chars.iter().position(|c| *c == '#').unwrap_or(chars.len());
            if let Some(end) = chars[..stop].iter().rposition(|c| *c == ']') {
                fill(kinds, i, end + 1, Kind::Type);
                return end + 1;
            }
            i
        }
        Keys::Toml | Keys::Yaml => {
            if keys == Keys::Yaml && starts_with(chars, i, "- ") {
                kinds[i] = Kind::Punct;
                i += 2;
            }
            let start = i;
            while chars
                .get(i)
                .is_some_and(|c| is_ident(*c) || matches!(c, '.' | '-'))
            {
                i += 1;
            }
            let mut after = i;
            while chars.get(after) == Some(&' ') {
                after += 1;
            }
            let is_key = i > start
                && match keys {
                    Keys::Toml => chars.get(after) == Some(&'='),
                    _ => {
                        chars.get(i) == Some(&':')
                            && chars.get(i + 1).is_none_or(|c| c.is_whitespace())
                    }
                };
            if is_key {
                fill(kinds, start, i, Kind::Type);
                i
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Colour one line of `lang` code: runs covering the whole line, with plain
/// text in `base` and tokens in the palette's syntax styles. Advances `state`
/// to the next line.
pub fn highlight(lang: Lang, state: &mut State, chars: &[char], base: Style) -> Vec<Run> {
    let spec = lang.spec();
    let n = chars.len();
    let mut kinds = vec![Kind::Plain; n];
    let mut i;

    // Resume whatever the previous line left open.
    let carried = std::mem::take(&mut state.0);
    match carried {
        Mode::Block(depth) => {
            let (end, left) = scan_block(chars, 0, depth, spec);
            fill(&mut kinds, 0, end, Kind::Comment);
            if left > 0 {
                state.0 = Mode::Block(left);
            }
            i = end;
        }
        Mode::Str { quote, triple } => {
            let (end, closed) = scan_string(chars, 0, quote, triple);
            fill(&mut kinds, 0, end, Kind::Str);
            if !closed {
                state.0 = carried;
            }
            i = end;
        }
        Mode::Normal => i = line_start_keys(chars, &mut kinds, spec.keys),
    }

    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // Line comment.
        if spec.line_comments.iter().any(|lc| {
            starts_with(chars, i, lc)
                && (!spec.hash_needs_boundary
                    || *lc != "#"
                    || i == 0
                    || chars[i - 1].is_whitespace())
        }) {
            fill(&mut kinds, i, n, Kind::Comment);
            break;
        }
        // Block comment.
        if let Some((open, _)) = spec.block_comment
            && starts_with(chars, i, open)
        {
            let (end, left) = scan_block(chars, i + open.chars().count(), 1, spec);
            fill(&mut kinds, i, end, Kind::Comment);
            if left > 0 {
                state.0 = Mode::Block(left);
            }
            i = end;
            continue;
        }
        // Rust lifetime: `'a` that does not close like `'a'` or `'\n'`.
        if spec.lifetimes
            && c == '\''
            && chars.get(i + 1).is_some_and(|c| is_ident_start(*c))
            && chars.get(i + 2) != Some(&'\'')
        {
            let end = word_end(chars, i + 1);
            fill(&mut kinds, i, end, Kind::Type);
            i = end;
            continue;
        }
        // String.
        if spec.quotes.contains(&c) {
            let triple =
                spec.triple && chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c);
            let open = if triple { 3 } else { 1 };
            let (end, closed) = scan_string(chars, i + open, c, triple);
            let mut kind = Kind::Str;
            if closed && spec.keys == Keys::Json {
                let mut after = end;
                while chars.get(after) == Some(&' ') {
                    after += 1;
                }
                if chars.get(after) == Some(&':') {
                    kind = Kind::Type;
                }
            }
            fill(&mut kinds, i, end, kind);
            if !closed && (triple || spec.multiline.contains(&c)) {
                state.0 = Mode::Str { quote: c, triple };
            }
            i = end;
            continue;
        }
        // Shell variables.
        if spec.dollar_vars && c == '$' {
            let end = match chars.get(i + 1) {
                Some('{') => chars[i..]
                    .iter()
                    .position(|c| *c == '}')
                    .map_or(n, |p| i + p + 1),
                Some(c) if is_ident_start(*c) => word_end(chars, i + 1),
                _ => i + 1,
            };
            fill(
                &mut kinds,
                i,
                end,
                if end > i + 1 { Kind::Type } else { Kind::Punct },
            );
            i = end;
            continue;
        }
        // Number: digits, then letters/underscores (`0xff`, `1_000u64`) and a
        // fractional part.
        if c.is_ascii_digit() {
            let mut end = i;
            while end < n {
                if is_ident(chars[end]) {
                    end += 1;
                } else if chars[end] == '.'
                    && chars.get(end + 1).is_some_and(|c| c.is_ascii_digit())
                {
                    end += 1;
                } else {
                    break;
                }
            }
            fill(&mut kinds, i, end, Kind::Number);
            i = end;
            continue;
        }
        // Identifier / keyword.
        if is_ident_start(c) {
            let end = word_end(chars, i);
            let word: String = chars[i..end].iter().collect();
            let kind = if spec.keywords.contains(&word.as_str()) {
                Kind::Keyword
            } else if spec.types.contains(&word.as_str())
                || (spec.capitalised_types && c.is_uppercase())
            {
                Kind::Type
            } else {
                Kind::Plain
            };
            fill(&mut kinds, i, end, kind);
            i = end;
            continue;
        }
        if c.is_ascii_punctuation() {
            kinds[i] = Kind::Punct;
        }
        i += 1;
    }

    // Collapse per-char kinds into runs.
    let mut runs: Vec<Run> = Vec::new();
    let mut start = 0;
    while start < n {
        let kind = kinds[start];
        let mut end = start + 1;
        while end < n && kinds[end] == kind {
            end += 1;
        }
        runs.push(Run {
            start,
            len: end - start,
            style: kind.style(base),
        });
        start = end;
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(style: Style) -> char {
        let base = Style::new();
        [
            ('K', Kind::Keyword),
            ('S', Kind::Str),
            ('N', Kind::Number),
            ('C', Kind::Comment),
            ('T', Kind::Type),
            ('P', Kind::Punct),
        ]
        .into_iter()
        .find(|(_, k)| k.style(base) == style)
        .map_or('.', |(t, _)| t)
    }

    /// The non-plain runs of each line as `(text, tag)`, state carried.
    fn lex(lang: Lang, src: &str) -> Vec<Vec<(String, char)>> {
        let base = palette::code_block();
        let mut st = State::default();
        src.split('\n')
            .map(|line| {
                let chars: Vec<char> = line.chars().collect();
                let runs = highlight(lang, &mut st, &chars, base);
                // Coverage: runs tile the line exactly.
                assert_eq!(runs.iter().map(|r| r.len).sum::<usize>(), chars.len());
                runs.iter()
                    .filter(|r| r.style != base)
                    .map(|r| {
                        (
                            chars[r.start..r.start + r.len].iter().collect(),
                            tag(r.style),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    fn line(lang: Lang, src: &str) -> Vec<(String, char)> {
        lex(lang, src).remove(0)
    }

    fn t(s: &str, k: char) -> (String, char) {
        (s.to_string(), k)
    }

    #[test]
    fn info_strings_and_aliases() {
        for (info, want) in [
            ("rust", Some(Lang::Rust)),
            ("rs", Some(Lang::Rust)),
            ("Rust,no_run", Some(Lang::Rust)),
            ("{.bash}", Some(Lang::Sh)),
            ("console", Some(Lang::Sh)),
            ("shell", Some(Lang::Sh)),
            ("yml", Some(Lang::Yaml)),
            ("ts title=\"a\"", Some(Lang::Js)),
            ("  json ", Some(Lang::Json)),
            ("", None),
            ("mermaid", None),
            ("brainfuck", None),
        ] {
            assert_eq!(Lang::from_info(info), want, "{info:?}");
        }
    }

    #[test]
    fn rust_tokens() {
        assert_eq!(
            line(Lang::Rust, "let x: u32 = 42; // hi"),
            vec![
                t("let", 'K'),
                t(":", 'P'),
                t("u32", 'T'),
                t("=", 'P'),
                t("42", 'N'),
                t(";", 'P'),
                t("// hi", 'C'),
            ]
        );
        assert_eq!(
            line(Lang::Rust, "fn f<'a>(s: &'a str) -> Vec<String> {"),
            vec![
                t("fn", 'K'),
                t("<", 'P'),
                t("'a", 'T'),
                t(">(", 'P'),
                t(":", 'P'),
                t("&", 'P'),
                t("'a", 'T'),
                t("str", 'T'),
                t(")", 'P'),
                t("->", 'P'),
                t("Vec", 'T'),
                t("<", 'P'),
                t("String", 'T'),
                t(">", 'P'),
                t("{", 'P'),
            ]
        );
    }

    #[test]
    fn rust_char_literals_are_strings_not_lifetimes() {
        assert_eq!(
            line(Lang::Rust, "let c = 'x'; let n = '\\n';"),
            vec![
                t("let", 'K'),
                t("=", 'P'),
                t("'x'", 'S'),
                t(";", 'P'),
                t("let", 'K'),
                t("=", 'P'),
                t("'\\n'", 'S'),
                t(";", 'P'),
            ]
        );
    }

    #[test]
    fn strings_keep_escapes_and_hide_keywords() {
        assert_eq!(
            line(Lang::Rust, r#"let s = "if \" // not"; x"#),
            vec![
                t("let", 'K'),
                t("=", 'P'),
                t(r#""if \" // not""#, 'S'),
                t(";", 'P'),
            ]
        );
    }

    #[test]
    fn block_comments_nest_and_carry_across_lines() {
        let out = lex(Lang::Rust, "a /* x /* y */\nstill */ b\nc");
        assert_eq!(out[0], vec![t("/* x /* y */", 'C')]);
        assert_eq!(out[1], vec![t("still */", 'C')]);
        assert_eq!(out[2], vec![]);
    }

    #[test]
    fn multiline_strings_carry_over() {
        let out = lex(Lang::Rust, "let s = \"one\nlet two\";\nlet y;");
        assert_eq!(out[1], vec![t("let two\"", 'S'), t(";", 'P')]);
        assert_eq!(out[2][0], t("let", 'K'));
        // A JSON string never spans lines: an unterminated one stops at EOL.
        let out = lex(Lang::Json, "\"open\n1");
        assert_eq!(out[1], vec![t("1", 'N')]);
    }

    #[test]
    fn sh_comments_variables_and_strings() {
        assert_eq!(
            line(Lang::Sh, "echo \"hi $USER\" ${HOME}/x # done"),
            vec![
                t("echo", 'K'),
                t("\"hi $USER\"", 'S'),
                t("${HOME}", 'T'),
                t("/", 'P'),
                t("# done", 'C'),
            ]
        );
        // `#` inside a word is not a comment.
        assert_eq!(
            line(Lang::Sh, "git log abc#1"),
            vec![t("#", 'P'), t("1", 'N')]
        );
        assert_eq!(
            line(Lang::Sh, "$ cargo test --release"),
            vec![t("$", 'P'), t("--", 'P')]
        );
    }

    #[test]
    fn json_keys_values_and_literals() {
        assert_eq!(
            line(Lang::Json, r#"  "name": "yaks", "n": -1.5e3, "ok": true,"#),
            vec![
                t("\"name\"", 'T'),
                t(":", 'P'),
                t("\"yaks\"", 'S'),
                t(",", 'P'),
                t("\"n\"", 'T'),
                t(":", 'P'),
                t("-", 'P'),
                t("1.5e3", 'N'),
                t(",", 'P'),
                t("\"ok\"", 'T'),
                t(":", 'P'),
                t("true", 'K'),
                t(",", 'P'),
            ]
        );
    }

    #[test]
    fn toml_tables_keys_and_values() {
        let out = lex(
            Lang::Toml,
            "[package]\nname = \"yaks\" # c\nversion = 1\nok = true\n[[bin]]",
        );
        assert_eq!(out[0], vec![t("[package]", 'T')]);
        assert_eq!(
            out[1],
            vec![
                t("name", 'T'),
                t("=", 'P'),
                t("\"yaks\"", 'S'),
                t("# c", 'C')
            ]
        );
        assert_eq!(out[2], vec![t("version", 'T'), t("=", 'P'), t("1", 'N')]);
        assert_eq!(out[3], vec![t("ok", 'T'), t("=", 'P'), t("true", 'K')]);
        assert_eq!(out[4], vec![t("[[bin]]", 'T')]);
    }

    #[test]
    fn yaml_keys() {
        let out = lex(Lang::Yaml, "name: yaks\n- id: 3\n  # note\nurl: http://x");
        assert_eq!(out[0], vec![t("name", 'T'), t(":", 'P')]);
        assert_eq!(
            out[1],
            vec![t("-", 'P'), t("id", 'T'), t(":", 'P'), t("3", 'N')]
        );
        assert_eq!(out[2], vec![t("# note", 'C')]);
        // `http://x` has no `: ` key separator after the first.
        assert_eq!(out[3][0], t("url", 'T'));
    }

    #[test]
    fn python_triple_strings_and_comments() {
        let out = lex(
            Lang::Python,
            "def f(x):  # hi\n    \"\"\"doc\n    more\"\"\"\n    return None",
        );
        assert_eq!(out[0][0], t("def", 'K'));
        assert_eq!(*out[0].last().unwrap(), t("# hi", 'C'));
        assert_eq!(out[1], vec![t("\"\"\"doc", 'S')]);
        assert_eq!(out[2], vec![t("    more\"\"\"", 'S')]);
        assert_eq!(out[3], vec![t("return", 'K'), t("None", 'K')]);
    }

    #[test]
    fn js_template_literals_and_java_annotations() {
        let out = lex(Lang::Js, "const a = `x\ny` + 1; // c");
        assert_eq!(out[0][0], t("const", 'K'));
        assert_eq!(out[0][2], t("`x", 'S'));
        assert_eq!(out[1][0], t("y`", 'S'));
        assert_eq!(*out[1].last().unwrap(), t("// c", 'C'));
        assert_eq!(
            line(Lang::Java, "@Override public int size() { return 0; }")
                .into_iter()
                .take(4)
                .collect::<Vec<_>>(),
            vec![
                t("@", 'P'),
                t("Override", 'T'),
                t("public", 'K'),
                t("int", 'T')
            ]
        );
    }

    #[test]
    fn unicode_columns_are_chars() {
        // The run for the string starts at char 8, not byte 8.
        let base = Style::new();
        let chars: Vec<char> = "é = \"ü\"; ".chars().collect();
        let runs = highlight(Lang::Rust, &mut State::default(), &chars, base);
        let s = runs.iter().find(|r| tag(r.style) == 'S').unwrap();
        assert_eq!((s.start, s.len), (4, 3));
    }
}
