//! Values the engine works with (filters, sorts, groups, calculations, formulas, rollups) and the
//! small date library they need. Dates are "naive" local date-times kept as minutes since
//! 1970-01-01 00:00 (no time zones, no network, no system calendar), written as `YYYY-MM-DD` or
//! `YYYY-MM-DDTHH:MM`.

use std::cmp::Ordering;

#[path = "number.rs"]
mod number;
pub use number::Number;
pub(super) use number::{Binary, Charge, LogBase};
pub(super) use number::with_budget;

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Bool(bool),
    Num(Number),
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

pub fn num_str(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "∞".into() } else { "-∞".into() };
    }
    Number::from_float(n).and_then(|n| n.significant_text()).expect("finite exact number")
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

pub fn currency_symbol(format: &str) -> Option<&'static str> {
    Some(match format {
        "dollar" => "$",
        "euro" => "€",
        "pound" => "£",
        "yen" => "¥",
        "rupee" => "₹",
        "won" => "₩",
        "yuan" => "CN¥",
        "rial" => "﷼",
        _ => return None,
    })
}

pub const CURRENCY_PLACES: u32 = 2;

fn with_commas(s: &str) -> String {
    if s.contains('e') {
        return s.to_owned();
    }
    match s.split_once('.') {
        Some((i, f)) => format!("{}.{f}", group_digits(i)),
        None => group_digits(s),
    }
}

/// The rounded decimal and its formatted currency text.
pub fn currency_amount(n: &Number, format: &str) -> Result<Option<(String, String)>, String> {
    let Some(symbol) = currency_symbol(format) else { return Ok(None) };
    let fixed = n.fixed(CURRENCY_PLACES)?;
    let (sign, magnitude) = fixed.strip_prefix('-').map_or(("", fixed.as_str()), |m| ("-", m));
    let text = format!("{sign}{symbol}{}", with_commas(magnitude));
    Ok(Some((fixed, text)))
}

pub fn format_number(n: &Number, format: &str) -> Result<String, String> {
    if let Some((_, text)) = currency_amount(n, format)? {
        return Ok(text);
    }
    Ok(match format {
        "numberWithCommas" => with_commas(&n.significant_text()?),
        "percent" => format!("{}%", n.mul(&Number::integer(100))?.significant_text()?),
        _ => n.significant_text()?,
    })
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

    pub fn integer(n: impl Into<number::BigInt>) -> Self {
        Self::Num(Number::integer(n))
    }

    pub fn from_float(n: f64) -> Result<Self, String> {
        Number::from_float(n).map(Self::Num)
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Num(n) => !n.is_zero(),
            Val::Str(s) => !s.is_empty(),
            Val::Date(_) => true,
            Val::List(l) => !l.is_empty(),
        }
    }

    pub(super) fn bool_checked(&self, charge: &mut Charge<'_>) -> Result<bool, String> {
        match self {
            Val::Num(n) if !n.is_exact() => Ok(!n.cmp_checked(&Number::integer(0), charge)?.is_eq()),
            other => Ok(other.as_bool()),
        }
    }

    /// Text as a person reads it (lists joined with ", ").
    pub fn text(&self) -> String {
        self.try_text().unwrap_or_else(|e| format!("\u{26a0} {e}"))
    }

    pub(crate) fn try_text(&self) -> Result<String, String> {
        with_budget(|charge| self.text_checked(charge))
    }

    pub(super) fn text_checked(&self, charge: &mut Charge<'_>) -> Result<String, String> {
        Ok(match self {
            Val::Null => String::new(),
            Val::Bool(b) => if *b { "true" } else { "false" }.into(),
            Val::Num(n) => n.significant_checked(charge)?,
            Val::Str(s) => s.clone(),
            Val::Date(d) => format_date(d, "full"),
            Val::List(l) => l.iter().map(|v| v.text_checked(charge)).collect::<Result<Vec<_>, _>>()?.join(", "),
        })
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
        (Val::Num(x), Val::Num(y)) => x.cmp(y),
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

pub(super) fn compare_checked(a: &Val, b: &Val, charge: &mut Charge<'_>) -> Result<Ordering, String> {
    Ok(match (a, b) {
        (Val::Num(a), Val::Num(b)) => a.cmp_checked(b, charge)?,
        (Val::List(a), Val::List(b)) => {
            for (left, right) in a.iter().zip(b) {
                let side = compare_checked(left, right, charge)?;
                if !side.is_eq() {
                    return Ok(side);
                }
            }
            a.len().cmp(&b.len())
        }
        _ => compare(a, b),
    })
}

pub(super) fn same_checked(a: &Val, b: &Val, charge: &mut Charge<'_>) -> Result<bool, String> {
    Ok(match (a, b) {
        (Val::Num(a), Val::Num(b)) => a.cmp_checked(b, charge)?.is_eq(),
        (Val::List(a), Val::List(b)) if a.len() == b.len() => {
            for (left, right) in a.iter().zip(b) {
                // List equality uses exact text and date values.
                let equal = match (left, right) {
                    (Val::Num(_), Val::Num(_)) | (Val::List(_), Val::List(_)) => same_checked(left, right, charge)?,
                    _ => left == right,
                };
                if !equal {
                    return Ok(false);
                }
            }
            true
        }
        _ => same(a, b),
    })
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

pub fn is_numeric_calc(name: &str) -> bool {
    matches!(name, "sum" | "avg" | "median" | "min" | "max" | "range")
}

pub fn calc_numbers(name: &str, nums: Vec<Number>) -> Result<Val, String> {
    if nums.iter().all(Number::is_exact) {
        calc_numbers_with(name, nums, |_| Ok(()))
    } else {
        with_budget(|charge| calc_numbers_with(name, nums, charge))
    }
}

pub(super) fn calc_numbers_with(
    name: &str,
    mut nums: Vec<Number>,
    mut charge: impl FnMut(usize) -> Result<(), String>,
) -> Result<Val, String> {
    let result = match name {
        "sum" | "avg" => {
            if name == "avg" && nums.is_empty() {
                return Ok(Val::Null);
            }
            let sum = nums.iter().try_fold(Number::integer(0), |sum, n| {
                charge(sum.arithmetic_work("+", n))?;
                sum.binary_checked(Binary::Add, n, &mut charge)
            })?;
            if name == "avg" {
                let count = Number::integer(nums.len());
                charge(sum.arithmetic_work("/", &count))?;
                sum.binary_checked(Binary::Div, &count, &mut charge)?
            } else { sum }
        }
        _ if nums.is_empty() => return Ok(Val::Null),
        "median" => {
            if nums.iter().all(Number::is_exact) {
                let largest_pair = nums.iter().map(|n| n.arithmetic_work("<", n)).max().unwrap_or(0);
                let passes = 2 * nums.len().ilog2() as usize + 1;
                charge(largest_pair.saturating_mul(nums.len()).saturating_mul(passes))?;
                nums.sort_unstable();
            } else {
                sort_checked(&mut nums, &mut |a, b| a.cmp_checked(b, &mut charge))?;
            }
            let m = nums.len() / 2;
            if nums.len() % 2 == 1 {
                nums.swap_remove(m)
            } else {
                charge(nums[m - 1].arithmetic_work("+", &nums[m]))?;
                let total = nums[m - 1].binary_checked(Binary::Add, &nums[m], &mut charge)?;
                let two = Number::integer(2);
                charge(total.arithmetic_work("/", &two))?;
                total.binary_checked(Binary::Div, &two, &mut charge)?
            }
        }
        "min" | "max" | "range" => {
            let extremum = |largest, charge: &mut Charge<'_>| -> Result<&Number, String> {
                nums.iter().skip(1).try_fold(&nums[0], |best, next| {
                    let side = next.cmp_checked(best, charge)?;
                    Ok(if (largest && side.is_gt()) || (!largest && side.is_lt()) { next } else { best })
                })
            };
            match name {
                "min" => extremum(false, &mut charge)?.clone(),
                "max" => extremum(true, &mut charge)?.clone(),
                _ => {
                    let hi = extremum(true, &mut charge)?;
                    let lo = extremum(false, &mut charge)?;
                    charge(hi.arithmetic_work("-", lo))?;
                    hi.binary_checked(Binary::Sub, lo, &mut charge)?
                }
            }
        }
        _ => return Ok(Val::Null),
    };
    Ok(Val::Num(result))
}

/// Fallible stable merge sort. The caller can discard the working list on error.
pub(super) fn sort_checked<T: Clone>(
    items: &mut [T], cmp: &mut impl FnMut(&T, &T) -> Result<Ordering, String>,
) -> Result<(), String> {
    fn merge<T: Clone>(
        items: &mut [T], scratch: &mut [T], cmp: &mut impl FnMut(&T, &T) -> Result<Ordering, String>,
    ) -> Result<(), String> {
        if items.len() < 2 {
            return Ok(());
        }
        let mid = items.len() / 2;
        let (left, right) = items.split_at_mut(mid);
        let (left_scratch, right_scratch) = scratch.split_at_mut(mid);
        merge(left, left_scratch, cmp)?;
        merge(right, right_scratch, cmp)?;
        let mut a = left.iter().peekable();
        let mut b = right.iter().peekable();
        for out in scratch.iter_mut() {
            let take_left = match (a.peek(), b.peek()) {
                (Some(left), Some(right)) => cmp(left, right)?.is_le(),
                (_, None) => true,
                (None, Some(_)) => false,
            };
            out.clone_from(if take_left { a.next() } else { b.next() }.expect("one remaining item"));
        }
        items.clone_from_slice(scratch);
        Ok(())
    }
    let mut scratch = items.to_vec();
    merge(items, &mut scratch, cmp)
}

fn collect_numbers(v: &Val, nested: bool, out: &mut Vec<Number>) -> Result<(), String> {
    match v {
        Val::List(items) => {
            for item in items { collect_numbers(item, true, out)?; }
        }
        Val::Num(n) => out.push(n.clone()),
        Val::Str(s) => out.extend(Number::parse(s)?),
        Val::Bool(b) if nested => out.push(Number::integer(u8::from(*b))),
        Val::Date(d) if nested => out.push(Number::integer(d.start)),
        _ => {}
    }
    Ok(())
}

pub(super) fn distinct(name: &str, vals: &[Val], charge: &mut Charge<'_>) -> Result<Val, String> {
    fn has_interval(value: &Val) -> bool {
        match value {
            Val::Num(n) => !n.is_exact(),
            Val::List(l) => l.iter().any(has_interval),
            _ => false,
        }
    }

    let mut seen = Vec::<(&Val, bool)>::new();
    for value in vals {
        let items = if let Val::List(items) = value { items } else { std::slice::from_ref(value) };
        'items: for item in items.iter().filter(|v| !v.is_empty()) {
            let ambiguous = has_interval(item);
            for &(previous, approximate) in &seen {
                let equal = if approximate || ambiguous {
                    same_checked(previous, item, charge)?
                } else {
                    same(previous, item)
                };
                if equal {
                    continue 'items;
                }
            }
            seen.push((item, ambiguous));
        }
    }
    Ok(if name == "countUnique" {
        Val::integer(seen.len())
    } else {
        Val::List(seen.into_iter().map(|(v, _)| v.clone()).collect())
    })
}

pub fn calc(name: &str, vals: &[Val]) -> Result<Val, String> {
    if is_numeric_calc(name) {
        let mut nums = Vec::new();
        for v in vals { collect_numbers(v, false, &mut nums)?; }
        return calc_numbers(name, nums);
    }
    let n = vals.len();
    let pct = |count: usize| -> Result<Val, String> {
        if n == 0 { return Ok(Val::integer(0)); }
        Number::integer(count).div(&Number::integer(n)).map(Val::Num)
    };
    Ok(match name {
        "countAll" | "count" => Val::integer(n),
        "countValues" | "countNotEmpty" => Val::integer(vals.iter().filter(|v| !v.is_empty()).count()),
        "countEmpty" => Val::integer(vals.iter().filter(|v| v.is_empty()).count()),
        "percentEmpty" => pct(vals.iter().filter(|v| v.is_empty()).count())?,
        "percentNotEmpty" => pct(vals.iter().filter(|v| !v.is_empty()).count())?,
        "countUnique" | "showUnique" => with_budget(|charge| distinct(name, vals, charge))?,
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
                return Ok(Val::Null);
            }
            let lo = dates.iter().map(|d| d.start).min().unwrap();
            let hi = dates.iter().map(|d| d.end.unwrap_or(d.start)).max().unwrap();
            let t = dates.iter().any(|d| d.has_time);
            match name {
                "earliest" => Val::Date(DateVal { start: lo, end: None, has_time: t }),
                "latest" => Val::Date(DateVal { start: hi, end: None, has_time: t }),
                _ => Val::integer((hi - lo) / MINUTES_PER_DAY),
            }
        }
        "checked" => Val::integer(vals.iter().filter(|v| matches!(v, Val::Bool(true))).count()),
        "unchecked" => Val::integer(vals.iter().filter(|v| matches!(v, Val::Bool(false) | Val::Null)).count()),
        "percentChecked" => pct(vals.iter().filter(|v| matches!(v, Val::Bool(true))).count())?,
        "percentUnchecked" => pct(vals.iter().filter(|v| matches!(v, Val::Bool(false) | Val::Null)).count())?,
        _ => Val::Null,
    })
}

pub fn calc_property_display(name: &str, value: &Val, prop: &super::types::Property) -> Result<String, String> {
    match value {
        Val::Num(n) if is_numeric_calc(name) => format_number(n, &prop.number_format),
        _ => calc_display_checked(name, value),
    }
}

/// How a calculation result is shown ("percent*" as a percentage, "dateRange" in days).
pub fn calc_display(name: &str, v: &Val) -> String {
    calc_display_checked(name, v).unwrap_or_else(|e| format!("\u{26a0} {e}"))
}

fn calc_display_checked(name: &str, v: &Val) -> Result<String, String> {
    Ok(match (name, v) {
        ("percentEmpty" | "percentNotEmpty" | "percentChecked" | "percentUnchecked", Val::Num(x)) => format!("{}%", num_str((x.try_as_f64()? * 1000.0).round() / 10.0)),
        ("dateRange", Val::Num(x)) => format!("{} days", x.significant_text()?),
        (_, Val::Date(d)) => format_date(d, "full"),
        (_, other) => other.try_text()?,
    })
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
        assert_eq!(format_number(&Number::decimal("1234567.5").unwrap(), "numberWithCommas").unwrap(), "1,234,567.5");
        assert_eq!(format_number(&Number::decimal("0.256").unwrap(), "percent").unwrap(), "25.6%");
        assert_eq!(format_number(&Number::decimal("-1234.0").unwrap(), "dollar").unwrap(), "-$1,234");
    }

    #[test]
    fn values_order_empty_last_and_numbers_naturally() {
        assert_eq!(compare(&Val::Null, &Val::integer(1)), Ordering::Greater);
        assert_eq!(compare(&Val::Str("item 2".into()), &Val::Str("Item 10".into())), Ordering::Less);
        assert_eq!(compare(&Val::integer(2), &Val::integer(10)), Ordering::Less);
        assert!(same(&Val::Str("A".into()), &Val::Str("a".into())));
    }

    #[test]
    fn calculations() {
        let v = vec![Val::integer(1), Val::integer(2), Val::integer(6), Val::Null];
        assert_eq!(calc("sum", &v).unwrap(), Val::integer(9));
        assert_eq!(calc("avg", &v).unwrap(), Val::integer(3));
        assert_eq!(calc("median", &v).unwrap(), Val::integer(2));
        assert_eq!(calc("min", &v).unwrap(), Val::integer(1));
        assert_eq!(calc("max", &v).unwrap(), Val::integer(6));
        assert_eq!(calc("range", &v).unwrap(), Val::integer(5));
        assert_eq!(calc("countAll", &v).unwrap(), Val::integer(4));
        assert_eq!(calc("countValues", &v).unwrap(), Val::integer(3));
        assert_eq!(calc("countEmpty", &v).unwrap(), Val::integer(1));
        assert_eq!(calc("percentNotEmpty", &v).unwrap(), Val::Num(Number::decimal("0.75").unwrap()));
        assert_eq!(calc_display("percentNotEmpty", &Val::Num(Number::decimal("0.75").unwrap())), "75%");
        let b = vec![Val::Bool(true), Val::Bool(false), Val::Bool(true)];
        assert_eq!(calc("checked", &b).unwrap(), Val::integer(2));
        assert_eq!(calc("unchecked", &b).unwrap(), Val::integer(1));
        let t = vec![Val::Str("a".into()), Val::Str("A".into()), Val::Str("b".into())];
        assert_eq!(calc("countUnique", &t).unwrap(), Val::integer(2));
        let d = vec![Val::Date(parse_date("2026-01-01").unwrap()), Val::Date(parse_date("2026-01-11").unwrap())];
        assert_eq!(calc("dateRange", &d).unwrap(), Val::integer(10));
        assert_eq!(calc("earliest", &d).unwrap(), Val::Date(parse_date("2026-01-01").unwrap()));
        assert_eq!(calc("median", &[Val::integer(1), Val::integer(3)]).unwrap(), Val::integer(2));
    }
}
