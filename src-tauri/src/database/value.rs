//! Values the engine works with (filters, sorts, groups, calculations, formulas, rollups) and the
//! small date library they need. Dates are "naive" local date-times kept as minutes since
//! 1970-01-01 00:00 (no time zones, no network, no system calendar), written as `YYYY-MM-DD` or
//! `YYYY-MM-DDTHH:MM`.

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Date(DateVal),
    List(Vec<Val>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DateVal {
    /// Minutes since 1970-01-01T00:00.
    pub start: i64,
    pub end: Option<i64>,
    pub has_time: bool,
}

pub const MINUTES_PER_DAY: i64 = 1440;

// ---- civil dates (proleptic Gregorian; algorithms by Howard Hinnant) ----

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

/// Parts of a minute count: (year, month, day, hour, minute).
pub fn parts(minutes: i64) -> (i64, i64, i64, i64, i64) {
    let days = minutes.div_euclid(MINUTES_PER_DAY);
    let rem = minutes.rem_euclid(MINUTES_PER_DAY);
    let (y, m, d) = civil_from_days(days);
    (y, m, d, rem / 60, rem % 60)
}

pub fn from_parts(y: i64, m: i64, d: i64, hh: i64, mm: i64) -> i64 {
    days_from_civil(y, m, d) * MINUTES_PER_DAY + hh * 60 + mm
}

/// 0 = Monday ... 6 = Sunday.
pub fn weekday(minutes: i64) -> i64 {
    // 1970-01-01 was a Thursday (3).
    (minutes.div_euclid(MINUTES_PER_DAY) + 3).rem_euclid(7)
}

pub fn start_of_day(minutes: i64) -> i64 {
    minutes.div_euclid(MINUTES_PER_DAY) * MINUTES_PER_DAY
}

/// Monday of the week (Notion's default is Sunday in the US, but Monday is the ISO start).
pub fn start_of_week(minutes: i64) -> i64 {
    start_of_day(minutes) - weekday(minutes) * MINUTES_PER_DAY
}

pub fn start_of_month(minutes: i64) -> i64 {
    let (y, m, _, _, _) = parts(minutes);
    from_parts(y, m, 1, 0, 0)
}

pub fn start_of_year(minutes: i64) -> i64 {
    let (y, _, _, _, _) = parts(minutes);
    from_parts(y, 1, 1, 0, 0)
}

pub fn add_months(minutes: i64, months: i64) -> i64 {
    let (y, m, d, hh, mm) = parts(minutes);
    let total = y * 12 + (m - 1) + months;
    let ny = total.div_euclid(12);
    let nm = total.rem_euclid(12) + 1;
    let nd = d.min(days_in_month(ny, nm));
    from_parts(ny, nm, nd, hh, mm)
}

pub fn parse_date(s: &str) -> Option<DateVal> {
    let s = s.trim();
    let (dpart, tpart) = match s.split_once(['T', ' ']) {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };
    let mut it = dpart.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    if it.next().is_some() || !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) || !(1..=9999).contains(&y) {
        return None;
    }
    let (hh, mm, has_time) = match tpart {
        Some(t) => {
            let t = t.trim_end_matches('Z');
            let mut it = t.split(':');
            let h: i64 = it.next()?.parse().ok()?;
            let mi: i64 = it.next().unwrap_or("0").parse().ok()?;
            if !(0..24).contains(&h) || !(0..60).contains(&mi) {
                return None;
            }
            (h, mi, true)
        }
        None => (0, 0, false),
    };
    Some(DateVal { start: from_parts(y, m, d, hh, mm), end: None, has_time })
}

pub fn format_iso(minutes: i64, with_time: bool) -> String {
    let (y, m, d, hh, mm) = parts(minutes);
    if with_time {
        format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}")
    } else {
        format!("{y:04}-{m:02}-{d:02}")
    }
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

pub fn month_name(m: i64) -> &'static str {
    MONTHS[((m - 1).rem_euclid(12)) as usize]
}

pub fn format_date(d: &DateVal, style: &str) -> String {
    let one = |m: i64| -> String {
        let (y, mo, da, hh, mi) = parts(m);
        let date = match style {
            "short" => format!("{} {da}, {y}", &month_name(mo)[..3]),
            "dmy" => format!("{da:02}/{mo:02}/{y}"),
            "mdy" => format!("{mo:02}/{da:02}/{y}"),
            "ymd" => format!("{y}/{mo:02}/{da:02}"),
            _ => format!("{} {da}, {y}", month_name(mo)),
        };
        if d.has_time {
            format!("{date} {hh:02}:{mi:02}")
        } else {
            date
        }
    };
    match d.end {
        Some(e) => format!("{} → {}", one(d.start), one(e)),
        None => one(d.start),
    }
}

/// Formats a date with a Notion-style pattern (`YYYY`, `MMMM`, `MMM`, `MM`, `M`, `DD`, `D`, `dddd`, `ddd`, `HH`, `mm`, `hh`, `A`).
pub fn format_pattern(minutes: i64, pat: &str) -> String {
    let (y, mo, da, hh, mi) = parts(minutes);
    let wd = weekday(minutes) as usize;
    let mut out = String::new();
    let chars: Vec<char> = pat.chars().collect();
    let mut i = 0;
    let run = |c: char, i: usize| chars[i..].iter().take_while(|x| **x == c).count();
    while i < chars.len() {
        let c = chars[i];
        let n = run(c, i);
        let piece: Option<String> = match (c, n) {
            ('Y', 4..) => Some(format!("{y:04}")),
            ('Y', 2..=3) => Some(format!("{:02}", y % 100)),
            ('M', 4..) => Some(month_name(mo).to_string()),
            ('M', 3) => Some(month_name(mo)[..3].to_string()),
            ('M', 2) => Some(format!("{mo:02}")),
            ('M', 1) => Some(mo.to_string()),
            ('D', 2..) => Some(format!("{da:02}")),
            ('D', 1) => Some(da.to_string()),
            ('d', 4..) => Some(DAYS[wd].to_string()),
            ('d', 3) => Some(DAYS[wd][..3].to_string()),
            ('H', 2..) => Some(format!("{hh:02}")),
            ('H', 1) => Some(hh.to_string()),
            ('h', 2..) => Some(format!("{:02}", if hh % 12 == 0 { 12 } else { hh % 12 })),
            ('h', 1) => Some((if hh % 12 == 0 { 12 } else { hh % 12 }).to_string()),
            ('m', 2..) => Some(format!("{mi:02}")),
            ('m', 1) => Some(mi.to_string()),
            ('A', _) => Some(if hh < 12 { "AM".into() } else { "PM".into() }),
            ('a', _) => Some(if hh < 12 { "am".into() } else { "pm".into() }),
            _ => None,
        };
        match piece {
            Some(p) => {
                out.push_str(&p);
                i += n;
            }
            None => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

// ---- numbers ----

/// Rounds like a person expects: 0.1+0.2 shows 0.3. Keeps at most 12 significant digits.
pub fn tidy(n: f64) -> f64 {
    if !n.is_finite() {
        return n;
    }
    let r = format!("{:.12e}", n).parse::<f64>().unwrap_or(n);
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

pub fn num_str(n: f64) -> String {
    let n = tidy(n);
    if n.is_nan() {
        return "NaN".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "∞".into() } else { "-∞".into() };
    }
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{n}");
        s
    }
}

fn group_digits(int_part: &str) -> String {
    let neg = int_part.starts_with('-');
    let digits: Vec<char> = int_part.trim_start_matches('-').chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

pub fn format_number(n: f64, format: &str) -> String {
    let base = num_str(n);
    let with_commas = |s: &str| -> String {
        match s.split_once('.') {
            Some((i, f)) => format!("{}.{f}", group_digits(i)),
            None => group_digits(s),
        }
    };
    let money = |sym: &str| format!("{}{}", if n < 0.0 { "-" } else { "" }, format!("{sym}{}", with_commas(&num_str(n.abs()))));
    match format {
        "numberWithCommas" => with_commas(&base),
        "percent" => format!("{}%", num_str(n * 100.0)),
        "dollar" => money("$"),
        "euro" => money("€"),
        "pound" => money("£"),
        "yen" => money("¥"),
        "rupee" => money("₹"),
        "won" => money("₩"),
        "yuan" => money("CN¥"),
        "rial" => money("﷼"),
        _ => base,
    }
}

// ---- Val helpers ----

impl Val {
    pub fn is_empty(&self) -> bool {
        match self {
            Val::Null => true,
            Val::Str(s) => s.is_empty(),
            Val::List(l) => l.is_empty(),
            _ => false,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Val::Num(n) => Some(*n),
            Val::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Val::Str(s) => s.trim().parse::<f64>().ok().filter(|n| n.is_finite()),
            Val::Date(d) => Some(d.start as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Num(n) => *n != 0.0,
            Val::Str(s) => !s.is_empty(),
            Val::Date(_) => true,
            Val::List(l) => !l.is_empty(),
        }
    }

    /// Text as a person reads it (lists joined with ", ").
    pub fn text(&self) -> String {
        match self {
            Val::Null => String::new(),
            Val::Bool(b) => if *b { "true" } else { "false" }.into(),
            Val::Num(n) => num_str(*n),
            Val::Str(s) => s.clone(),
            Val::Date(d) => format_date(d, "full"),
            Val::List(l) => l.iter().map(|v| v.text()).collect::<Vec<_>>().join(", "),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Val::Null => "null",
            Val::Bool(_) => "bool",
            Val::Num(_) => "number",
            Val::Str(_) => "text",
            Val::Date(_) => "date",
            Val::List(_) => "list",
        }
    }
}

/// A total order used by sorts and groups: empty values last, numbers by value, text by
/// case-insensitive text, dates by start, booleans false < true. Different types compare by type.
pub fn compare(a: &Val, b: &Val) -> Ordering {
    let rank = |v: &Val| match v {
        Val::Null => 9,
        Val::Bool(_) => 1,
        Val::Num(_) => 2,
        Val::Date(_) => 3,
        Val::Str(_) => 4,
        Val::List(_) => 5,
    };
    match (a, b) {
        (Val::Null, Val::Null) => Ordering::Equal,
        (Val::Null, _) => Ordering::Greater,
        (_, Val::Null) => Ordering::Less,
        (Val::Bool(x), Val::Bool(y)) => x.cmp(y),
        (Val::Num(x), Val::Num(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (Val::Date(x), Val::Date(y)) => x.start.cmp(&y.start),
        (Val::Str(x), Val::Str(y)) => natural_cmp(x, y),
        (Val::List(x), Val::List(y)) => {
            for (p, q) in x.iter().zip(y.iter()) {
                let o = compare(p, q);
                if o != Ordering::Equal {
                    return o;
                }
            }
            x.len().cmp(&y.len())
        }
        _ => rank(a).cmp(&rank(b)),
    }
}

/// Case-insensitive comparison where digit runs compare as numbers ("item 2" < "item 10").
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0, 0);
    while i < ac.len() && j < bc.len() {
        if ac[i].is_ascii_digit() && bc[j].is_ascii_digit() {
            let si = i;
            while i < ac.len() && ac[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < bc.len() && bc[j].is_ascii_digit() {
                j += 1;
            }
            let x: String = ac[si..i].iter().collect::<String>().trim_start_matches('0').to_string();
            let y: String = bc[sj..j].iter().collect::<String>().trim_start_matches('0').to_string();
            let o = x.len().cmp(&y.len()).then_with(|| x.cmp(&y));
            if o != Ordering::Equal {
                return o;
            }
        } else {
            let x = ac[i].to_lowercase().next().unwrap_or(ac[i]);
            let y = bc[j].to_lowercase().next().unwrap_or(bc[j]);
            let o = x.cmp(&y);
            if o != Ordering::Equal {
                return o;
            }
            i += 1;
            j += 1;
        }
    }
    (ac.len() - i).cmp(&(bc.len() - j))
}

/// Equality for filters: numbers by value, text case-insensitively.
pub fn same(a: &Val, b: &Val) -> bool {
    match (a, b) {
        (Val::Str(x), Val::Str(y)) => x.to_lowercase() == y.to_lowercase(),
        (Val::Num(x), Val::Num(y)) => tidy(*x) == tidy(*y),
        (Val::Date(x), Val::Date(y)) => x.start == y.start,
        (x, y) => x == y,
    }
}

// ---- calculations (table footers, rollups, chart values) ----

/// The calculations a column footer or a rollup can apply.
pub const CALCS: &[&str] = &[
    "none", "countAll", "countValues", "countUnique", "countEmpty", "countNotEmpty", "percentEmpty", "percentNotEmpty", "sum", "avg",
    "median", "min", "max", "range", "earliest", "latest", "dateRange", "checked", "unchecked", "percentChecked", "percentUnchecked",
    "showOriginal", "showUnique",
];

pub fn calc(name: &str, vals: &[Val]) -> Val {
    let n = vals.len() as f64;
    let nums = || -> Vec<f64> {
        let mut out = Vec::new();
        for v in vals {
            match v {
                Val::List(l) => out.extend(l.iter().filter_map(|x| x.as_num())),
                Val::Num(x) => out.push(*x),
                Val::Str(s) => {
                    if let Some(x) = Val::Str(s.clone()).as_num() {
                        out.push(x)
                    }
                }
                Val::Bool(_) | Val::Date(_) | Val::Null => {}
            }
        }
        out
    };
    let pct = |count: f64| if n == 0.0 { Val::Num(0.0) } else { Val::Num(count / n) };
    match name {
        "countAll" | "count" => Val::Num(n),
        "countValues" | "countNotEmpty" => Val::Num(vals.iter().filter(|v| !v.is_empty()).count() as f64),
        "countEmpty" => Val::Num(vals.iter().filter(|v| v.is_empty()).count() as f64),
        "percentEmpty" => pct(vals.iter().filter(|v| v.is_empty()).count() as f64),
        "percentNotEmpty" => pct(vals.iter().filter(|v| !v.is_empty()).count() as f64),
        "countUnique" => {
            let mut seen: Vec<Val> = Vec::new();
            for v in vals {
                let items: Vec<&Val> = match v {
                    Val::List(l) => l.iter().collect(),
                    other => vec![other],
                };
                for it in items {
                    if !it.is_empty() && !seen.iter().any(|s| same(s, it)) {
                        seen.push(it.clone());
                    }
                }
            }
            Val::Num(seen.len() as f64)
        }
        "showUnique" => {
            let mut seen: Vec<Val> = Vec::new();
            for v in vals {
                let items: Vec<&Val> = match v {
                    Val::List(l) => l.iter().collect(),
                    other => vec![other],
                };
                for it in items {
                    if !it.is_empty() && !seen.iter().any(|s| same(s, it)) {
                        seen.push(it.clone());
                    }
                }
            }
            Val::List(seen)
        }
        "showOriginal" => {
            let mut out = Vec::new();
            for v in vals {
                match v {
                    Val::List(l) => out.extend(l.iter().cloned()),
                    Val::Null => {}
                    other => out.push(other.clone()),
                }
            }
            Val::List(out)
        }
        "sum" => Val::Num(tidy(nums().iter().sum())),
        "avg" => {
            let x = nums();
            if x.is_empty() {
                Val::Null
            } else {
                Val::Num(tidy(x.iter().sum::<f64>() / x.len() as f64))
            }
        }
        "median" => {
            let mut x = nums();
            if x.is_empty() {
                return Val::Null;
            }
            x.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let m = x.len() / 2;
            Val::Num(if x.len() % 2 == 1 { x[m] } else { (x[m - 1] + x[m]) / 2.0 })
        }
        "min" => nums().into_iter().fold(None, |a: Option<f64>, b| Some(a.map_or(b, |a| a.min(b)))).map_or(Val::Null, Val::Num),
        "max" => nums().into_iter().fold(None, |a: Option<f64>, b| Some(a.map_or(b, |a| a.max(b)))).map_or(Val::Null, Val::Num),
        "range" => {
            let x = nums();
            if x.is_empty() {
                Val::Null
            } else {
                let lo = x.iter().cloned().fold(f64::INFINITY, f64::min);
                let hi = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                Val::Num(tidy(hi - lo))
            }
        }
        "earliest" | "latest" | "dateRange" => {
            let mut dates: Vec<DateVal> = Vec::new();
            for v in vals {
                match v {
                    Val::Date(d) => dates.push(*d),
                    Val::List(l) => dates.extend(l.iter().filter_map(|x| if let Val::Date(d) = x { Some(*d) } else { None })),
                    _ => {}
                }
            }
            if dates.is_empty() {
                return Val::Null;
            }
            let lo = dates.iter().map(|d| d.start).min().unwrap();
            let hi = dates.iter().map(|d| d.end.unwrap_or(d.start)).max().unwrap();
            let t = dates.iter().any(|d| d.has_time);
            match name {
                "earliest" => Val::Date(DateVal { start: lo, end: None, has_time: t }),
                "latest" => Val::Date(DateVal { start: hi, end: None, has_time: t }),
                _ => Val::Num(((hi - lo) / MINUTES_PER_DAY) as f64),
            }
        }
        "checked" => Val::Num(vals.iter().filter(|v| matches!(v, Val::Bool(true))).count() as f64),
        "unchecked" => Val::Num(vals.iter().filter(|v| matches!(v, Val::Bool(false) | Val::Null)).count() as f64),
        "percentChecked" => pct(vals.iter().filter(|v| matches!(v, Val::Bool(true))).count() as f64),
        "percentUnchecked" => pct(vals.iter().filter(|v| matches!(v, Val::Bool(false) | Val::Null)).count() as f64),
        _ => Val::Null,
    }
}

/// How a calculation result is shown ("percent*" as a percentage, "dateRange" in days).
pub fn calc_display(name: &str, v: &Val) -> String {
    match (name, v) {
        ("percentEmpty" | "percentNotEmpty" | "percentChecked" | "percentUnchecked", Val::Num(x)) => format!("{}%", num_str((x * 1000.0).round() / 10.0)),
        ("dateRange", Val::Num(x)) => format!("{} days", num_str(*x)),
        (_, Val::Date(d)) => format_date(d, "full"),
        (_, other) => other.text(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_round_trip_and_weekdays_are_right() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        for d in [-800_000_i64, -1, 0, 1, 19_000, 20_000, 2_000_000] {
            let (y, m, dd) = civil_from_days(d);
            assert_eq!(days_from_civil(y, m, dd), d);
        }
        // 2026-11-03 is a Tuesday.
        let m = from_parts(2026, 11, 3, 12, 0);
        assert_eq!(weekday(m), 1);
        assert_eq!(parts(m), (2026, 11, 3, 12, 0));
        assert_eq!(weekday(from_parts(1970, 1, 1, 0, 0)), 3);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2100, 2), 28);
    }

    #[test]
    fn dates_parse_and_format() {
        let d = parse_date("2026-02-28T09:05").unwrap();
        assert!(d.has_time);
        assert_eq!(format_iso(d.start, true), "2026-02-28T09:05");
        assert_eq!(format_iso(parse_date("2026-02-28").unwrap().start, false), "2026-02-28");
        assert!(parse_date("2026-02-30").is_none());
        assert!(parse_date("2026-13-01").is_none());
        assert!(parse_date("2026-01-01T25:00").is_none());
        assert!(parse_date("nonsense").is_none());
        assert_eq!(format_date(&parse_date("2026-11-03").unwrap(), "full"), "November 3, 2026");
        assert_eq!(format_date(&parse_date("2026-11-03").unwrap(), "dmy"), "03/11/2026");
        assert_eq!(format_pattern(from_parts(2026, 11, 3, 15, 4), "dddd, MMMM D, YYYY h:mm A"), "Tuesday, November 3, 2026 3:04 PM");
    }

    #[test]
    fn months_add_with_clamping_and_weeks_start_on_monday() {
        let jan31 = from_parts(2026, 1, 31, 0, 0);
        assert_eq!(parts(add_months(jan31, 1)).1, 2);
        assert_eq!(parts(add_months(jan31, 1)).2, 28);
        assert_eq!(parts(add_months(jan31, -2)).0, 2025);
        let tue = from_parts(2026, 11, 3, 10, 0);
        assert_eq!(parts(start_of_week(tue)).2, 2);
    }

    #[test]
    fn numbers_format_like_notion() {
        assert_eq!(num_str(0.1 + 0.2), "0.3");
        assert_eq!(num_str(3.0), "3");
        assert_eq!(format_number(1234567.5, "numberWithCommas"), "1,234,567.5");
        assert_eq!(format_number(0.256, "percent"), "25.6%");
        assert_eq!(format_number(-1234.0, "dollar"), "-$1,234");
    }

    #[test]
    fn values_order_empty_last_and_numbers_naturally() {
        assert_eq!(compare(&Val::Null, &Val::Num(1.0)), Ordering::Greater);
        assert_eq!(compare(&Val::Str("item 2".into()), &Val::Str("Item 10".into())), Ordering::Less);
        assert_eq!(compare(&Val::Num(2.0), &Val::Num(10.0)), Ordering::Less);
        assert!(same(&Val::Str("A".into()), &Val::Str("a".into())));
    }

    #[test]
    fn calculations() {
        let v = vec![Val::Num(1.0), Val::Num(2.0), Val::Num(6.0), Val::Null];
        assert_eq!(calc("sum", &v), Val::Num(9.0));
        assert_eq!(calc("avg", &v), Val::Num(3.0));
        assert_eq!(calc("median", &v), Val::Num(2.0));
        assert_eq!(calc("min", &v), Val::Num(1.0));
        assert_eq!(calc("max", &v), Val::Num(6.0));
        assert_eq!(calc("range", &v), Val::Num(5.0));
        assert_eq!(calc("countAll", &v), Val::Num(4.0));
        assert_eq!(calc("countValues", &v), Val::Num(3.0));
        assert_eq!(calc("countEmpty", &v), Val::Num(1.0));
        assert_eq!(calc("percentNotEmpty", &v), Val::Num(0.75));
        assert_eq!(calc_display("percentNotEmpty", &Val::Num(0.75)), "75%");
        let b = vec![Val::Bool(true), Val::Bool(false), Val::Bool(true)];
        assert_eq!(calc("checked", &b), Val::Num(2.0));
        assert_eq!(calc("unchecked", &b), Val::Num(1.0));
        let t = vec![Val::Str("a".into()), Val::Str("A".into()), Val::Str("b".into())];
        assert_eq!(calc("countUnique", &t), Val::Num(2.0));
        let d = vec![Val::Date(parse_date("2026-01-01").unwrap()), Val::Date(parse_date("2026-01-11").unwrap())];
        assert_eq!(calc("dateRange", &d), Val::Num(10.0));
        assert_eq!(calc("earliest", &d), Val::Date(parse_date("2026-01-01").unwrap()));
        assert_eq!(calc("median", &[Val::Num(1.0), Val::Num(3.0)]), Val::Num(2.0));
    }
}
