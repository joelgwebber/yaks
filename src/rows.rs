//! Rows of a table dok (yaks-6cdd): the first markdown table in a dok's
//! body, read for `dok query --rows` and written by `dok row set|delete`.
//!
//! The first column is the row's key. A write changes only the lines it has
//! to: the body outside the table is returned byte for byte, rows it does
//! not touch keep their text, and a new column is appended to the header,
//! the separator and every row.

use anyhow::{Result, bail};

/// `<dok>#<key>`: the dok path and the row key (split at the first `#`, so a
/// key may hold more).
pub fn split_address(addr: &str) -> Result<(&str, &str)> {
    match addr.split_once('#') {
        Some((dok, key)) if !dok.is_empty() && !key.trim().is_empty() => Ok((dok, key.trim())),
        _ => bail!("{addr:?}: expected <dok>#<key>, e.g. doks/topics/reading.md#dune"),
    }
}

/// What a row write did.
#[derive(Debug, PartialEq, Eq)]
pub enum RowChange {
    Created,
    Updated,
    Unchanged,
}

/// The cells of one table line: one leading and one trailing `|` dropped,
/// split at every `|` that is not escaped as `\|`, cells trimmed.
pub fn split_cells(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = match t.strip_suffix('|') {
        Some(rest) if !rest.ends_with('\\') => rest,
        _ => t,
    };
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                chars.next();
                cur.push('|');
            }
            '|' => cells.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

fn render_cells(cells: &[String]) -> String {
    let cells: Vec<String> = cells.iter().map(|c| c.replace('|', "\\|")).collect();
    format!("| {} |", cells.join(" | "))
}

/// A line's text without its line ending, and the ending.
fn split_eol(line: &str) -> (&str, &str) {
    let body = line.trim_end_matches(['\n', '\r']);
    (body, &line[body.len()..])
}

/// The first table in a body, as its lines (endings kept).
struct Table {
    lines: Vec<String>,
    header: Vec<String>,
    /// Index of the header line; the separator is the next one.
    head: usize,
    /// Index of the first row, and one past the last.
    first: usize,
    end: usize,
}

impl Table {
    /// The first line starting with `|` is the header; a table needs a
    /// separator line (`| --- |`) right after it.
    fn locate(body: &str) -> Option<Table> {
        let lines: Vec<String> = body.split_inclusive('\n').map(String::from).collect();
        let text = |i: usize| lines.get(i).map(|l| l.trim());
        let head = lines.iter().position(|l| l.trim().starts_with('|'))?;
        let sep = text(head + 1)?;
        if !sep.starts_with('|') || !sep.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
            return None;
        }
        let first = head + 2;
        let end = (first..lines.len())
            .find(|&i| !lines[i].trim().starts_with('|'))
            .unwrap_or(lines.len());
        let header = split_cells(&lines[head]);
        Some(Table {
            lines,
            header,
            head,
            first,
            end,
        })
    }

    fn row(&self, i: usize) -> Vec<String> {
        split_cells(&self.lines[i])
    }

    fn body(&self) -> String {
        self.lines.concat()
    }
}

const NO_TABLE: &str = "has no markdown table (the first line starting with `|` must be a header, followed by a `| --- |` line); add one, then `yaks dok row set` can fill it";

/// Rows of a table dok: the first markdown table in the body, one list per
/// row of `(column, cell)` in header order, cells trimmed (Hearth's table
/// doks). Empty when there is no table.
pub fn table_rows(body: &str) -> Vec<Vec<(String, String)>> {
    let Some(t) = Table::locate(body) else {
        return Vec::new();
    };
    (t.first..t.end)
        .map(|i| t.header.iter().cloned().zip(t.row(i)).collect())
        .collect()
}

/// `line` with `extra` cells appended, keeping its text; a line that does not
/// have `old_n` cells (or end in `|`) is written out cell by cell.
fn widen(line: &str, old_n: usize, extra: &[String]) -> String {
    let (text, eol) = split_eol(line);
    let mut cells = split_cells(text);
    let t = text.trim_end();
    if cells.len() == old_n && t.ends_with('|') && !t.ends_with("\\|") {
        let mut out = t.to_string();
        for c in extra {
            out.push_str(&format!(" {} |", c.replace('|', "\\|")));
        }
        return out + eol;
    }
    cells.resize(old_n, String::new());
    cells.extend(extra.iter().cloned());
    render_cells(&cells) + eol
}

fn check_text(what: &str, s: &str) -> Result<()> {
    if s.contains(['\n', '\r']) {
        bail!("{what} {s:?} has a line break; a table cell is one line");
    }
    Ok(())
}

/// Create or update the row whose key (first column) is `key` with `cells`
/// (`column`, `value`). A column the table does not have is appended to it.
/// A new row goes at the end, or in place when the rows are already sorted by
/// key (ignoring case). Returns the new body and what changed.
pub fn set_row(body: &str, key: &str, cells: &[(String, String)]) -> Result<(String, RowChange)> {
    let Some(mut t) = Table::locate(body) else {
        bail!("{NO_TABLE}");
    };
    check_text("key", key)?;
    for (c, v) in cells {
        if c.is_empty() {
            bail!("an empty column name; use column=value");
        }
        check_text("column", c)?;
        check_text("value", v)?;
    }
    let key_col = t.header[0].clone();
    // The key column is the row's address: a --set of it may only repeat the key.
    for (c, v) in cells {
        if *c == key_col && v.trim() != key {
            bail!(
                "{c} is the key column of this table and cannot be changed by row set (delete the row and set a new one)"
            );
        }
    }
    let matching: Vec<usize> = (t.first..t.end)
        .filter(|&i| t.row(i).first().is_some_and(|k| k == key))
        .collect();
    if matching.len() > 1 {
        bail!(
            "{} rows have the key {key:?} (table lines {}); make the keys unique by hand first",
            matching.len(),
            matching
                .iter()
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    // Columns the table lacks, in the order given.
    let mut new_cols: Vec<String> = Vec::new();
    for (c, _) in cells {
        if !t.header.contains(c) && !new_cols.contains(c) {
            new_cols.push(c.clone());
        }
    }
    let old_n = t.header.len();
    if !new_cols.is_empty() {
        let dashes = vec!["---".to_string(); new_cols.len()];
        t.lines[t.head] = widen(&t.lines[t.head], old_n, &new_cols);
        t.lines[t.head + 1] = widen(&t.lines[t.head + 1], old_n, &dashes);
        let blanks = vec![String::new(); new_cols.len()];
        for i in t.first..t.end {
            t.lines[i] = widen(&t.lines[i], old_n, &blanks);
        }
        t.header.extend(new_cols);
    }
    let column = |c: &str| t.header.iter().position(|h| h == c).expect("column exists");
    let change = if let Some(&i) = matching.first() {
        let mut row = t.row(i);
        row.resize(t.header.len(), String::new());
        let before = row.clone();
        for (c, v) in cells {
            row[column(c)] = v.trim().to_string();
        }
        if row == before {
            RowChange::Unchanged
        } else {
            let (_, eol) = split_eol(&t.lines[i]);
            t.lines[i] = render_cells(&row) + eol;
            RowChange::Updated
        }
    } else {
        let mut row = vec![String::new(); t.header.len()];
        row[0] = key.to_string();
        for (c, v) in cells {
            row[column(c)] = v.trim().to_string();
        }
        let keys: Vec<String> = (t.first..t.end)
            .map(|i| t.row(i).first().cloned().unwrap_or_default().to_lowercase())
            .collect();
        let sorted = keys.windows(2).all(|w| w[0] <= w[1]);
        let at = if sorted {
            let k = key.to_lowercase();
            t.first + keys.iter().position(|x| *x > k).unwrap_or(keys.len())
        } else {
            t.end
        };
        let eol = match t.lines[t.head].ends_with("\r\n") {
            true => "\r\n",
            false => "\n",
        };
        if at > 0 && !t.lines[at - 1].ends_with('\n') {
            t.lines[at - 1].push_str(eol);
        }
        let mut line = render_cells(&row);
        // A last line with no newline stays that way.
        if at < t.lines.len() || body.ends_with('\n') {
            line.push_str(eol);
        }
        t.lines.insert(at, line);
        RowChange::Created
    };
    Ok((t.body(), change))
}

/// Remove the row whose key is `key`. Returns the new body.
pub fn delete_row(body: &str, key: &str) -> Result<String> {
    let Some(mut t) = Table::locate(body) else {
        bail!("{NO_TABLE}");
    };
    let matching: Vec<usize> = (t.first..t.end)
        .filter(|&i| t.row(i).first().is_some_and(|k| k == key))
        .collect();
    match matching.as_slice() {
        [] => bail!("no row with the key {key:?} (`yaks dok query --rows` lists the rows)"),
        [i] => {
            t.lines.remove(*i);
            Ok(t.body())
        }
        many => bail!(
            "{} rows have the key {key:?} (table lines {}); delete one by hand, or make the keys unique",
            many.len(),
            many.iter()
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "Intro.\n\n| name | author |\n| --- | --- |\n| dune | Herbert |\n| emma | Austen |\n\nAfter.\n";

    fn set(body: &str, key: &str, cells: &[(&str, &str)]) -> (String, RowChange) {
        let cells: Vec<(String, String)> = cells
            .iter()
            .map(|(c, v)| (c.to_string(), v.to_string()))
            .collect();
        set_row(body, key, &cells).unwrap()
    }

    #[test]
    fn address_splits_at_the_first_hash() {
        assert_eq!(
            split_address("doks/a.md#b#c").unwrap(),
            ("doks/a.md", "b#c")
        );
        assert!(split_address("doks/a.md").is_err());
        assert!(split_address("doks/a.md#").is_err());
        assert!(split_address("#k").is_err());
    }

    #[test]
    fn cells_split_on_unescaped_pipes() {
        assert_eq!(split_cells("| a | b\\|c | |"), ["a", "b|c", ""]);
        assert_eq!(split_cells("a | b"), ["a", "b"]);
        assert_eq!(split_cells("| a ||"), ["a", ""]);
    }

    #[test]
    fn a_new_row_goes_at_the_end_when_the_rows_are_not_sorted() {
        let (out, ch) = set(BODY, "ana", &[("author", "X")]);
        assert_eq!(ch, RowChange::Created);
        let un = BODY.replace(
            "dune | Herbert |\n| emma | Austen",
            "emma | Austen |\n| dune | Herbert",
        );
        let (out2, _) = set(&un, "ana", &[("author", "X")]);
        assert!(
            out2.contains("| dune | Herbert |\n| ana | X |\n\nAfter."),
            "{out2}"
        );
        // sorted: in place
        assert!(
            out.contains("| --- | --- |\n| ana | X |\n| dune | Herbert |"),
            "{out}"
        );
    }

    #[test]
    fn a_sorted_table_gets_the_row_in_place_and_the_end_when_it_sorts_last() {
        let (mid, _) = set(BODY, "Eli", &[("author", "Z")]);
        assert!(
            mid.contains("| dune | Herbert |\n| Eli | Z |\n| emma | Austen |\n"),
            "{mid}"
        );
        let (last, _) = set(BODY, "zed", &[]);
        assert!(
            last.contains("| emma | Austen |\n| zed |  |\n\nAfter."),
            "{last}"
        );
    }

    #[test]
    fn updating_a_row_changes_only_its_line() {
        let (out, ch) = set(BODY, "dune", &[("author", "F. Herbert")]);
        assert_eq!(ch, RowChange::Updated);
        assert_eq!(
            out,
            BODY.replace("| dune | Herbert |", "| dune | F. Herbert |")
        );
        let (same, ch) = set(BODY, "dune", &[("author", "Herbert")]);
        assert_eq!((same.as_str(), ch), (BODY, RowChange::Unchanged));
    }

    #[test]
    fn a_new_column_is_appended_to_the_header_separator_and_every_row() {
        let (out, ch) = set(BODY, "dune", &[("year", "1965")]);
        assert_eq!(ch, RowChange::Updated);
        assert_eq!(
            out,
            "Intro.\n\n| name | author | year |\n| --- | --- | --- |\n| dune | Herbert | 1965 |\n| emma | Austen |  |\n\nAfter.\n"
        );
    }

    #[test]
    fn a_pipe_in_a_cell_is_escaped_and_reads_back() {
        let (out, _) = set(BODY, "dune", &[("author", "a|b")]);
        assert!(out.contains("| dune | a\\|b |"), "{out}");
        assert_eq!(table_rows(&out)[0][1].1, "a|b");
    }

    #[test]
    fn delete_removes_one_row_and_leaves_the_rest_alone() {
        let out = delete_row(BODY, "dune").unwrap();
        assert_eq!(out, BODY.replace("| dune | Herbert |\n", ""));
        assert!(
            delete_row(BODY, "nope")
                .unwrap_err()
                .to_string()
                .contains("no row")
        );
    }

    #[test]
    fn refusals_say_what_to_do() {
        let e = set_row("No table here.\n", "a", &[])
            .unwrap_err()
            .to_string();
        assert!(e.contains("no markdown table"), "{e}");
        let dup = BODY.replace("emma | Austen", "dune | Other");
        for e in [
            set_row(&dup, "dune", &[]).unwrap_err().to_string(),
            delete_row(&dup, "dune").unwrap_err().to_string(),
        ] {
            assert!(e.contains("2 rows have the key"), "{e}");
        }
        let e = set_row(BODY, "dune", &[("name".into(), "other".into())]).unwrap_err();
        assert!(e.to_string().contains("key column"), "{e}");
        let e = set_row(BODY, "dune", &[("author".into(), "a\nb".into())]).unwrap_err();
        assert!(e.to_string().contains("line break"), "{e}");
    }

    #[test]
    fn a_table_ending_the_body_without_a_newline_stays_valid() {
        let body = "| k | v |\n| - | - |\n| a | 1 |";
        let (out, _) = set(body, "b", &[("v", "2")]);
        assert_eq!(out, "| k | v |\n| - | - |\n| a | 1 |\n| b | 2 |");
    }

    #[test]
    fn the_body_outside_the_table_is_untouched_whatever_it_holds() {
        let body = "# T\r\n\r\ntext | with pipes\r\n\r\n| k | v |\r\n| - | - |\r\n| a | 1 |\r\n\r\n```\r\n| code | x |\r\n```\r\n";
        let (out, _) = set(body, "b", &[("v", "2")]);
        assert_eq!(
            out,
            body.replace("| a | 1 |\r\n", "| a | 1 |\r\n| b | 2 |\r\n")
        );
        let back = delete_row(&out, "b").unwrap();
        assert_eq!(back, body);
    }
}
