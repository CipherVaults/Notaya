//! CSV for databases (A35): reading a file into a table of text (with the guesses the importer
//! needs), writing one safely, and reading the date formats that other tools (Notion included)
//! write. Nothing here touches the vault or the disk.

use super::value::{from_parts, parse_date};

pub const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_ROWS: usize = 100_000;
pub const MAX_COLUMNS: usize = 100;

pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Turns the bytes of a file into text: UTF-8 (with or without a byte-order mark) or UTF-16.
pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let le = bytes[0] == 0xFF;
        let units: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) }).collect();
        return String::from_utf16_lossy(&units);
    }
    let b = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8_lossy(b).into_owned()
}

fn detect_delimiter(text: &str) -> char {
    let (mut comma, mut semi, mut tab) = (0usize, 0usize, 0usize);
    let mut quoted = false;
    for c in text.chars() {
        match c {
            '"' => quoted = !quoted,
            '\n' if !quoted => break,
            ',' if !quoted => comma += 1,
            ';' if !quoted => semi += 1,
            '\t' if !quoted => tab += 1,
            _ => {}
        }
    }
    if tab > comma && tab >= semi {
        '\t'
    } else if semi > comma {
        ';'
    } else {
        ','
    }
}

/// Reads CSV text (RFC 4180 quotes, CRLF or LF, comma / semicolon / tab) into a header and rows.
/// Every row is padded to the same length. Empty lines are skipped.
#[allow(unused_assignments)]
pub fn parse(text: &str) -> Result<Table, String> {
    let delim = detect_delimiter(text);
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut field = String::new();
    let mut rec: Vec<String> = Vec::new();
    let mut quoted = false;
    let mut was_quoted = false;
    let mut chars = text.chars().peekable();
    let mut any = false;
    macro_rules! end_field {
        () => {{
            rec.push(std::mem::take(&mut field));
            was_quoted = false;
        }};
    }
    macro_rules! end_record {
        () => {{
            end_field!();
            if !(rec.len() == 1 && rec[0].is_empty() && !any) {
                records.push(std::mem::take(&mut rec));
            } else {
                rec.clear();
            }
            any = false;
            if records.len() > MAX_ROWS + 1 {
                return Err(format!("This file has more than {MAX_ROWS} rows."));
            }
        }};
    }
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !was_quoted => {
                quoted = true;
                was_quoted = true;
                any = true;
            }
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                end_record!();
            }
            '\n' => end_record!(),
            c if c == delim => {
                any = true;
                end_field!();
            }
            c => {
                any = true;
                field.push(c);
            }
        }
    }
    if quoted {
        return Err("A quoted value is never closed. The file may be damaged.".into());
    }
    if any || !field.is_empty() || !rec.is_empty() {
        end_record!();
    }
    // Drop rows that are only separators.
    records.retain(|r| r.iter().any(|f| !f.trim().is_empty()));
    if records.is_empty() {
        return Err("The file is empty.".into());
    }
    let cols = records.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols > MAX_COLUMNS {
        return Err(format!("This file has more than {MAX_COLUMNS} columns."));
    }
    for r in &mut records {
        r.resize(cols, String::new());
    }
    let mut header = records.remove(0);
    // Unique, non-empty names.
    let mut seen: Vec<String> = Vec::new();
    for (i, h) in header.iter_mut().enumerate() {
        let mut name: String = h.trim().chars().filter(|c| !c.is_control()).take(100).collect();
        if name.is_empty() {
            name = format!("Column {}", i + 1);
        }
        let base = name.clone();
        let mut n = 2;
        while seen.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
            name = format!("{base} {n}");
            n += 1;
        }
        seen.push(name.clone());
        *h = name;
    }
    Ok(Table { header, rows: records })
}

/// A text that a spreadsheet would run as a formula starts with one of these.
fn risky(s: &str) -> bool {
    matches!(s.chars().next(), Some('=' | '+' | '-' | '@' | '\t' | '\r'))
}

/// Makes a text safe to open in a spreadsheet: a leading `'` stops it being read as a formula.
/// `is_number` leaves a real number (such as -5) alone.
pub fn safe_cell(s: &str, is_number: bool) -> String {
    if !is_number && risky(s) {
        format!("'{s}")
    } else {
        s.to_string()
    }
}

/// Undoes `safe_cell` for a text read back in.
pub fn unsafe_cell(s: &str) -> String {
    match s.strip_prefix('\'') {
        Some(rest) if risky(rest) => rest.to_string(),
        _ => s.to_string(),
    }
}

fn quote(f: &str) -> String {
    if f.contains([',', '"', '\n', '\r']) || f.starts_with(' ') || f.ends_with(' ') {
        format!("\"{}\"", f.replace('"', "\"\""))
    } else {
        f.to_string()
    }
}

/// UTF-8 with a byte-order mark (so Excel reads Arabic and Chinese correctly) and CRLF line ends.
pub fn write(rows: &[Vec<String>]) -> Vec<u8> {
    let mut out = String::from("\u{FEFF}");
    for r in rows {
        out.push_str(&r.iter().map(|f| quote(f)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out.into_bytes()
}

// ------------------------------------------------------------------ dates

fn month_of(word: &str) -> Option<i64> {
    let w = word.trim_matches(|c: char| !c.is_alphabetic()).to_ascii_lowercase();
    if w.len() < 3 {
        return None;
    }
    const NAMES: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    NAMES.iter().position(|n| n.starts_with(&w)).map(|i| i as i64 + 1)
}

/// One date (or date and time) as minutes since 1970 and whether it has a time.
/// Reads `2026-11-03`, `2026-11-03T14:30`, `2026/11/03`, `November 3, 2026`, `3 November 2026`
/// and `November 3, 2026 2:30 PM`.
pub fn parse_one_date(s: &str) -> Option<(i64, bool)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let iso = s.replace('/', "-").replace('.', "-");
    if iso.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) && iso.split(['T', ' ']).next().is_some_and(|d| d.matches('-').count() == 2 && d.split('-').next().is_some_and(|y| y.len() == 4)) {
        if let Some(d) = parse_date(&iso) {
            return Some((d.start, d.has_time));
        }
    }
    let cleaned = s.replace(',', " ");
    let t: Vec<&str> = cleaned.split_whitespace().collect();
    if t.len() < 3 {
        return None;
    }
    let (m, d, y, rest) = if let Some(m) = month_of(t[0]) {
        (m, t[1].trim_end_matches(|c: char| c.is_alphabetic()).parse::<i64>().ok()?, t[2].parse::<i64>().ok()?, &t[3..])
    } else if let Some(m) = month_of(t[1]) {
        (m, t[0].trim_end_matches(|c: char| c.is_alphabetic()).parse::<i64>().ok()?, t[2].parse::<i64>().ok()?, &t[3..])
    } else {
        return None;
    };
    if !(1..=9999).contains(&y) || !(1..=31).contains(&d) || d > super::value::days_in_month(y, m) {
        return None;
    }
    if rest.is_empty() {
        return Some((from_parts(y, m, d, 0, 0), false));
    }
    let mut it = rest[0].split(':');
    let mut h: i64 = it.next()?.parse().ok()?;
    let mi: i64 = it.next().unwrap_or("0").parse().ok()?;
    match rest.get(1).map(|x| x.to_ascii_lowercase()).as_deref() {
        Some("pm") if h < 12 => h += 12,
        Some("am") if h == 12 => h = 0,
        Some("am") | Some("pm") | None => {}
        Some(_) => return None,
    }
    if !(0..24).contains(&h) || !(0..60).contains(&mi) {
        return None;
    }
    Some((from_parts(y, m, d, h, mi), true))
}

/// A date or a range (`start → end`, as Notion writes it) as minutes. `None` if it is not one.
pub fn parse_date_range(s: &str) -> Option<(i64, Option<i64>, bool)> {
    let (a, b) = match s.split_once('→').or_else(|| s.split_once(" to ")) {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let (start, st) = parse_one_date(a)?;
    match b {
        None => Some((start, None, st)),
        Some(b) => {
            let (end, et) = parse_one_date(b)?;
            if end < start {
                return None;
            }
            Some((start, Some(end), st || et))
        }
    }
}

// ------------------------------------------------------------------ guessing a column's kind

#[derive(Debug, PartialEq, Clone)]
pub enum Guess {
    Text,
    Number,
    Checkbox,
    Date,
    Url,
    Email,
    Select(Vec<String>),
    MultiSelect(Vec<String>),
    /// A Notion status column (names in the order first seen).
    Status(Vec<String>),
    /// A number shown with a unit, as Notion writes it: the property's number format ("dollar", "percent", ...).
    FormattedNumber(String),
}

pub fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "yes" | "true" | "checked" | "x" | "✓" => Some(true),
        "no" | "false" | "unchecked" => Some(false),
        _ => None,
    }
}

pub fn parse_number(s: &str) -> Option<f64> {
    let t = s.trim();
    let first = t.chars().next()?;
    if !(first.is_ascii_digit() || first == '-' || first == '+' || first == '.') {
        return None;
    }
    // "007" and phone-like values stay text.
    let digits = t.trim_start_matches(['-', '+']);
    if digits.len() > 1 && digits.starts_with('0') && !digits.starts_with("0.") {
        return None;
    }
    t.parse::<f64>().ok().filter(|n| n.is_finite())
}

/// A number as Notion writes it in a CSV: `$12,000.50`, `€5`, `60%`, `1,234`. Returns the value (a percent is
/// divided by 100, as the property stores it) and the number format that shows it the same way.
pub fn parse_formatted_number(s: &str) -> Option<(f64, &'static str)> {
    let t = s.trim();
    let (fmt, rest) = match t.chars().next()? {
        '$' => ("dollar", &t[1..]),
        '€' => ("euro", &t['€'.len_utf8()..]),
        '£' => ("pound", &t['£'.len_utf8()..]),
        '¥' => ("yen", &t['¥'.len_utf8()..]),
        _ => ("", t),
    };
    let (rest, percent) = match rest.strip_suffix('%') {
        Some(r) => (r, true),
        None => (rest, false),
    };
    if fmt != "" && percent {
        return None;
    }
    let negative = rest.starts_with('-');
    let body = rest.trim_start_matches('-').trim();
    if body.is_empty() || !body.chars().next()?.is_ascii_digit() {
        return None;
    }
    let had_commas = body.contains(',');
    if had_commas {
        // Thousands separators only: groups of three after the first.
        let mut groups = body.split('.').next()?.split(',');
        let first = groups.next()?;
        if first.is_empty() || first.len() > 3 || !groups.all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit())) {
            return None;
        }
    }
    let clean: String = body.chars().filter(|c| *c != ',').collect();
    let mut n: f64 = clean.parse().ok().filter(|n: &f64| n.is_finite())?;
    if negative {
        n = -n;
    }
    if percent {
        return Some((n / 100.0, "percent"));
    }
    if fmt.is_empty() && had_commas {
        return Some((n, "numberWithCommas"));
    }
    Some((n, if fmt.is_empty() { "number" } else { fmt }))
}

/// What a status column's value means for the status groups: "todo", "inProgress" or "complete".
pub fn status_group(name: &str) -> &'static str {
    let n = name.trim().to_ascii_lowercase();
    match n.as_str() {
        "done" | "complete" | "completed" | "finished" | "shipped" | "closed" | "resolved" | "published" => "complete",
        "in progress" | "doing" | "in review" | "review" | "reading" | "blocked" | "active" | "started" | "working" | "testing" | "on hold" => "inProgress",
        _ => "todo",
    }
}

/// Like `infer`, for the tables of a Notion export. Notion does not say what a column is, so this guesses a bit
/// harder: a short list of repeated values is a select, a column called "Status" is a status, and numbers written
/// with a currency sign, a percent sign or thousands separators are numbers with that format.
pub fn infer_notion(header: &str, values: &[&str], allow_choice: bool) -> Guess {
    let v: Vec<&str> = values.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if v.is_empty() {
        return Guess::Text;
    }
    let all: Vec<(f64, &'static str)> = v.iter().filter_map(|s| parse_formatted_number(s)).collect();
    if all.len() == v.len() && !v.iter().all(|s| parse_number(s).is_some()) {
        // One unit for the whole column ("number" mixed in with a unit still takes the unit).
        let unit = all.iter().map(|a| a.1).find(|f| *f != "number").unwrap_or("number");
        if all.iter().all(|a| a.1 == unit || a.1 == "number") {
            return Guess::FormattedNumber(unit.to_string());
        }
    }
    let g = infer(values, allow_choice);
    if !allow_choice || !matches!(g, Guess::Text) {
        if let Guess::Select(d) = g {
            return if header.trim().eq_ignore_ascii_case("status") { Guess::Status(d) } else { Guess::Select(d) };
        }
        return g;
    }
    if v.iter().any(|s| s.chars().count() > 80 || s.contains('\n')) {
        return Guess::Text;
    }
    let owned: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    if owned.iter().any(|s| s.contains(',')) {
        let tags: Vec<String> = owned.iter().flat_map(|s| split_tags(s)).collect();
        let d = distinct(&tags);
        if d.len() <= 30 && tags.len() > d.len() && d.iter().all(|t| t.chars().count() <= 40) {
            return Guess::MultiSelect(d);
        }
        return Guess::Text;
    }
    let d = distinct(&owned);
    let is_status = header.trim().eq_ignore_ascii_case("status");
    if d.len() <= 12 && (d.len() < owned.len() || (is_status && d.iter().all(|t| t.chars().count() <= 30))) {
        return if is_status { Guess::Status(d) } else { Guess::Select(d) };
    }
    Guess::Text
}

fn is_email(s: &str) -> bool {
    let t = s.trim();
    match t.split_once('@') {
        Some((a, b)) => !a.is_empty() && b.contains('.') && !b.ends_with('.') && !t.contains(char::is_whitespace) && !b.contains('@'),
        None => false,
    }
}

fn distinct(values: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in values {
        if !out.contains(v) {
            out.push(v.clone());
        }
    }
    out
}

/// Splits a multi-select cell (`a, b, c`).
pub fn split_tags(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in s.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()) {
        if !out.iter().any(|o| o == t) {
            out.push(t.to_string());
        }
    }
    out
}

/// Guesses the property kind of a column from its values. `allow_choice` is false for the title.
pub fn infer(values: &[&str], allow_choice: bool) -> Guess {
    let v: Vec<&str> = values.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if v.is_empty() {
        return Guess::Text;
    }
    if v.iter().all(|s| parse_number(s).is_some()) {
        return Guess::Number;
    }
    if v.iter().all(|s| parse_bool(s).is_some()) {
        return Guess::Checkbox;
    }
    if v.iter().all(|s| parse_date_range(s).is_some()) {
        return Guess::Date;
    }
    if v.iter().all(|s| (s.starts_with("http://") || s.starts_with("https://")) && !s.contains(char::is_whitespace)) {
        return Guess::Url;
    }
    if v.iter().all(|s| is_email(s)) {
        return Guess::Email;
    }
    if !allow_choice || v.iter().any(|s| s.chars().count() > 80 || s.contains('\n')) {
        return Guess::Text;
    }
    let owned: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    if owned.iter().any(|s| s.contains(',')) {
        let tags: Vec<String> = owned.iter().flat_map(|s| split_tags(s)).collect();
        let d = distinct(&tags);
        if d.len() <= 30 && tags.len() >= d.len() * 2 && d.iter().all(|t| t.chars().count() <= 40) {
            return Guess::MultiSelect(d);
        }
        return Guess::Text;
    }
    let d = distinct(&owned);
    if d.len() <= 20 && owned.len() >= d.len() * 2 {
        return Guess::Select(d);
    }
    Guess::Text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_quotes_newlines_and_bom() {
        let t = parse(&decode(b"\xEF\xBB\xBFName,Notes\r\n\"A, B\",\"line1\nline2 \"\"q\"\"\"\r\n\r\nC,\r\n")).unwrap();
        assert_eq!(t.header, vec!["Name", "Notes"]);
        assert_eq!(t.rows, vec![vec!["A, B", "line1\nline2 \"q\""], vec!["C", ""]]);
    }

    #[test]
    fn detects_semicolon_and_tab() {
        assert_eq!(parse("a;b\n1;2").unwrap().rows, vec![vec!["1", "2"]]);
        assert_eq!(parse("a\tb\n1\t2").unwrap().rows, vec![vec!["1", "2"]]);
        assert_eq!(parse("\"a;b\",c\n1,2").unwrap().header, vec!["a;b", "c"]);
    }

    #[test]
    fn utf16_and_arabic() {
        let mut b = vec![0xFF, 0xFE];
        for u in "اسم,x\nأحمد,1".encode_utf16() {
            b.extend_from_slice(&u.to_le_bytes());
        }
        let t = parse(&decode(&b)).unwrap();
        assert_eq!(t.header[0], "اسم");
        assert_eq!(t.rows[0][0], "أحمد");
    }

    #[test]
    fn bad_files_are_refused() {
        assert!(parse("").is_err());
        assert!(parse("a,b\n\"never closed,1").is_err());
        let wide = (0..101).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        assert!(parse(&wide).is_err());
    }

    #[test]
    fn names_are_unique_and_rows_padded() {
        let t = parse("A,a,\n1\n").unwrap();
        assert_eq!(t.header, vec!["A", "a 2", "Column 3"]);
        assert_eq!(t.rows[0].len(), 3);
    }

    #[test]
    fn formula_injection_is_stopped_and_round_trips() {
        assert_eq!(safe_cell("=HYPERLINK(\"x\")", false), "'=HYPERLINK(\"x\")");
        assert_eq!(safe_cell("-5", true), "-5");
        assert_eq!(safe_cell("+1 555", false), "'+1 555");
        assert_eq!(safe_cell("hello", false), "hello");
        assert_eq!(unsafe_cell("'=1+1"), "=1+1");
        assert_eq!(unsafe_cell("'quoted"), "'quoted");
    }

    #[test]
    fn writes_with_bom_crlf_and_quotes() {
        let out = String::from_utf8(write(&[vec!["a,b".into(), "c\"d".into()], vec!["x".into(), "".into()]])).unwrap();
        assert_eq!(out, "\u{FEFF}\"a,b\",\"c\"\"d\"\r\nx,\r\n");
        let back = parse(&decode(out.as_bytes())).unwrap();
        assert_eq!(back.header, vec!["a,b", "c\"d"]);
    }

    #[test]
    fn dates_in_several_shapes() {
        let a = parse_one_date("2026-11-03").unwrap();
        assert_eq!(a, (from_parts(2026, 11, 3, 0, 0), false));
        assert_eq!(parse_one_date("November 3, 2026").unwrap(), a);
        assert_eq!(parse_one_date("3 Nov 2026").unwrap(), a);
        assert_eq!(parse_one_date("2026/11/03").unwrap(), a);
        assert_eq!(parse_one_date("November 3, 2026 2:30 PM").unwrap(), (from_parts(2026, 11, 3, 14, 30), true));
        assert_eq!(parse_one_date("November 3, 2026 12:05 AM").unwrap(), (from_parts(2026, 11, 3, 0, 5), true));
        assert!(parse_one_date("February 30, 2026").is_none());
        assert!(parse_one_date("hello world 5").is_none());
        let (s, e, _) = parse_date_range("November 3, 2026 → November 5, 2026").unwrap();
        assert_eq!(e.unwrap() - s, 2 * 24 * 60);
        assert!(parse_date_range("November 5, 2026 → November 3, 2026").is_none());
    }

    #[test]
    fn guesses_kinds() {
        assert_eq!(infer(&["1", "2.5", "-3", ""], true), Guess::Number);
        assert_eq!(infer(&["007", "008"], true), Guess::Text, "leading zeros stay text");
        assert_eq!(infer(&["Yes", "No"], true), Guess::Checkbox);
        assert_eq!(infer(&["2026-01-01", "March 4, 2026"], true), Guess::Date);
        assert_eq!(infer(&["https://a.com", "http://b.org/x"], true), Guess::Url);
        assert_eq!(infer(&["a@b.com", "c@d.org"], true), Guess::Email);
        assert_eq!(infer(&["Open", "Done", "Open", "Done", "Open"], true), Guess::Select(vec!["Open".into(), "Done".into()]));
        assert_eq!(infer(&["Open", "Done", "Open", "Done", "Open"], false), Guess::Text, "a title is never a choice");
        assert_eq!(infer(&["a, b", "a", "b, a", "b"], true), Guess::MultiSelect(vec!["a".into(), "b".into()]));
        assert_eq!(infer(&["one", "two", "three"], true), Guess::Text);
        assert_eq!(infer(&[], true), Guess::Text);
    }
}
