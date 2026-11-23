//! The formula language of a Formula property (Notion's style): `prop("Name")`, operators
//! (`+ - * / % ^`, comparisons, `and or not` or `&& || !`, `a ? b : c`), text, numbers, dates and
//! lists, function calls `f(x)` and method calls `x.f()`, and lambdas for the list functions
//! (`current`, `index`). A formula never touches files or the network. Evaluation is bounded
//! (a step budget and a nesting limit).

use std::cell::Cell;

use super::regex::Regex;
use super::value::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Num(Number),
    Str(String),
    Bool(bool),
    Ident(String),
    List(Vec<Expr>),
    Call(String, Vec<Expr>),
    Method(Box<Expr>, String, Vec<Expr>),
    Unary(char, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(Number),
    Str(String),
    Id(String),
    Op(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Question,
    Colon,
}

fn lex(src: &str) -> Result<Vec<Tok>, String> {
    if src.len() > 64 * 1024 {
        return Err("Formula is too large".into());
    }
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < c.len() {
        if out.len() >= 4096 {
            return Err("Formula has too many tokens".into());
        }
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch.is_ascii_digit() || (ch == '.' && c.get(i + 1).map_or(false, |x| x.is_ascii_digit())) {
            let s = i;
            while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
                i += 1;
            }
            if matches!(c.get(i), Some('e' | 'E')) {
                let first = i + if matches!(c.get(i + 1), Some('+' | '-')) { 2 } else { 1 };
                if c.get(first).is_some_and(char::is_ascii_digit) {
                    i = first + 1;
                    while i < c.len() && c[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let t: String = c[s..i].iter().collect();
            out.push(Tok::Num(Number::decimal(&t)?));
        } else if ch == '"' || ch == '\'' {
            let q = ch;
            i += 1;
            let mut s = String::new();
            loop {
                let x = *c.get(i).ok_or("Text is not closed")?;
                i += 1;
                if x == q {
                    break;
                }
                if x == '\\' {
                    let e = *c.get(i).ok_or("Text is not closed")?;
                    i += 1;
                    s.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                } else {
                    s.push(x);
                }
            }
            out.push(Tok::Str(s));
        } else if ch.is_alphabetic() || ch == '_' {
            let s = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Id(c[s..i].iter().collect()));
        } else {
            i += 1;
            let two: String = if i < c.len() { [ch, c[i]].iter().collect() } else { String::new() };
            match two.as_str() {
                "==" | "!=" | "<=" | ">=" | "&&" | "||" => {
                    i += 1;
                    out.push(Tok::Op(two));
                    continue;
                }
                _ => {}
            }
            out.push(match ch {
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '[' => Tok::LBracket,
                ']' => Tok::RBracket,
                ',' => Tok::Comma,
                '.' => Tok::Dot,
                '?' => Tok::Question,
                ':' => Tok::Colon,
                '+' | '-' | '*' | '/' | '%' | '^' | '<' | '>' | '!' | '=' => Tok::Op(ch.to_string()),
                other => return Err(format!("Unexpected character {other}")),
            });
        }
    }
    Ok(out)
}

struct P {
    t: Vec<Tok>,
    i: usize,
    depth: usize,
}

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.t.get(self.i).cloned();
        self.i += 1;
        t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<Expr, String> {
        if self.depth >= 60 {
            return Err("Formula is nested too deeply".into());
        }
        self.depth += 1;
        let r = self.ternary();
        self.depth -= 1;
        checked(r?)
    }
    fn ternary(&mut self) -> Result<Expr, String> {
        let c = self.binary(0)?;
        if self.eat(&Tok::Question) {
            let a = self.expr()?;
            if !self.eat(&Tok::Colon) {
                return Err("Expected : in a ? b : c".into());
            }
            let b = self.expr()?;
            return Ok(Expr::Ternary(Box::new(c), Box::new(a), Box::new(b)));
        }
        Ok(c)
    }
    fn prec(op: &str) -> Option<(u8, bool)> {
        // (precedence, right-assoc)
        Some(match op {
            "or" | "||" => (1, false),
            "and" | "&&" => (2, false),
            "==" | "!=" => (3, false),
            "<" | ">" | "<=" | ">=" => (4, false),
            "+" | "-" => (5, false),
            "*" | "/" | "%" => (6, false),
            "^" => (8, true),
            _ => return None,
        })
    }
    fn binary(&mut self, min: u8) -> Result<Expr, String> {
        // Every recursive cycle (including unary and right-associative ^) visits
        // expr or binary. Bound it before descending, including malformed input.
        if self.depth >= 60 {
            return Err("Formula is nested too deeply".into());
        }
        self.depth += 1;
        let result = self.binary_inner(min);
        self.depth -= 1;
        checked(result?)
    }
    fn binary_inner(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Op(o)) => o.clone(),
                Some(Tok::Id(o)) if o == "and" || o == "or" => o.clone(),
                _ => break,
            };
            let (p, right) = match Self::prec(&op) {
                Some(x) => x,
                None => break,
            };
            if p < min {
                break;
            }
            self.i += 1;
            let rhs = self.binary(if right { p } else { p + 1 })?;
            left = checked(Expr::Binary(op, Box::new(left), Box::new(rhs)))?;
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Tok::Op(o)) if o == "-" || o == "!" => {
                let c = if o == "-" { '-' } else { '!' };
                self.i += 1;
                // unary binds tighter than * / but looser than ^
                let e = self.binary(7)?;
                Ok(Expr::Unary(c, Box::new(e)))
            }
            Some(Tok::Op(o)) if o == "+" => {
                self.i += 1;
                self.binary(7)
            }
            Some(Tok::Id(o)) if o == "not" => {
                self.i += 1;
                let e = self.binary(7)?;
                Ok(Expr::Unary('!', Box::new(e)))
            }
            _ => self.postfix(),
        }
    }
    fn postfix(&mut self) -> Result<Expr, String> {
        let mut e = self.primary()?;
        while self.eat(&Tok::Dot) {
            let name = match self.next() {
                Some(Tok::Id(n)) => n,
                _ => return Err("Expected a function name after .".into()),
            };
            let args = if self.eat(&Tok::LParen) { self.args()? } else { Vec::new() };
            e = checked(Expr::Method(Box::new(e), name, args))?;
        }
        Ok(e)
    }
    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut v = Vec::new();
        if self.eat(&Tok::RParen) {
            return Ok(v);
        }
        loop {
            v.push(self.expr()?);
            if self.eat(&Tok::Comma) {
                continue;
            }
            if self.eat(&Tok::RParen) {
                return Ok(v);
            }
            return Err("Expected , or )".into());
        }
    }
    fn primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Expr::Num(n)),
            Some(Tok::Str(s)) => Ok(Expr::Str(s)),
            Some(Tok::LParen) => {
                let e = self.expr()?;
                if !self.eat(&Tok::RParen) {
                    return Err("Missing )".into());
                }
                Ok(e)
            }
            Some(Tok::LBracket) => {
                let mut v = Vec::new();
                if !self.eat(&Tok::RBracket) {
                    loop {
                        v.push(self.expr()?);
                        if self.eat(&Tok::Comma) {
                            continue;
                        }
                        if self.eat(&Tok::RBracket) {
                            break;
                        }
                        return Err("Expected , or ]".into());
                    }
                }
                Ok(Expr::List(v))
            }
            Some(Tok::Id(n)) => {
                if n == "true" {
                    return Ok(Expr::Bool(true));
                }
                if n == "false" {
                    return Ok(Expr::Bool(false));
                }
                if self.eat(&Tok::LParen) {
                    let a = self.args()?;
                    return Ok(Expr::Call(n, a));
                }
                Ok(Expr::Ident(n))
            }
            Some(t) => Err(format!("Unexpected {}", tok_text(&t))),
            None => Err("The formula ends too soon".into()),
        }
    }
}

fn tok_text(t: &Tok) -> String {
    match t {
        Tok::Num(n) => n.significant_text().expect("exact numeric token"),
        Tok::Str(s) => format!("\"{s}\""),
        Tok::Id(s) => s.clone(),
        Tok::Op(s) => s.clone(),
        Tok::LParen => "(".into(),
        Tok::RParen => ")".into(),
        Tok::LBracket => "[".into(),
        Tok::RBracket => "]".into(),
        Tok::Comma => ",".into(),
        Tok::Dot => ".".into(),
        Tok::Question => "?".into(),
        Tok::Colon => ":".into(),
    }
}

/// How deep an expression may nest (long chains such as `1 + 1 + 1 + ...` count too), so
/// evaluating or dropping one cannot overflow the stack.
const MAX_DEPTH: usize = 120;

// Children have already been checked: a rejected tree has at most MAX_DEPTH+1
// levels and can be dropped normally on *every* error path. Never leak an AST.
fn checked(e: Expr) -> Result<Expr, String> {
    if depth_of(&e, MAX_DEPTH) > MAX_DEPTH {
        Err("The formula is nested too deeply".into())
    } else {
        Ok(e)
    }
}

fn depth_of(e: &Expr, limit: usize) -> usize {
    if limit == 0 {
        return usize::MAX / 2;
    }
    1 + match e {
        Expr::Num(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Ident(_) => 0,
        Expr::List(v) | Expr::Call(_, v) => v.iter().map(|x| depth_of(x, limit - 1)).max().unwrap_or(0),
        Expr::Method(r, _, v) => std::iter::once(&**r).chain(v.iter()).map(|x| depth_of(x, limit - 1)).max().unwrap_or(0),
        Expr::Unary(_, x) => depth_of(x, limit - 1),
        Expr::Binary(_, a, b) => depth_of(a, limit - 1).max(depth_of(b, limit - 1)),
        Expr::Ternary(a, b, c) => depth_of(a, limit - 1).max(depth_of(b, limit - 1)).max(depth_of(c, limit - 1)),
    }
}

pub fn parse(src: &str) -> Result<Expr, String> {
    // Apply the byte budget even to the empty/whitespace fast path.
    if src.len() > 64 * 1024 {
        return Err("Formula is too large".into());
    }
    if src.trim().is_empty() {
        return Ok(Expr::Str(String::new()));
    }
    let t = lex(src)?;
    let mut p = P { t, i: 0, depth: 0 };
    let e = p.expr()?;
    if p.i < p.t.len() {
        return Err(format!("Unexpected {}", tok_text(&p.t[p.i])));
    }
    checked(e)
}

/// Where `prop("Name")` and `id()` get their answers.
pub trait Env {
    fn prop(&self, name: &str) -> Result<Val, String>;
    fn row_id(&self) -> String;
    /// "Now" as minutes since 1970 (local).
    fn now(&self) -> i64;
}

struct Scope<'a> {
    vars: Vec<(String, Val)>,
    env: &'a dyn Env,
    steps: usize,
    arithmetic_left: Cell<usize>,
}

const MAX_STEPS: usize = 200_000;
const MAX_LIST: usize = 100_000;

pub fn eval(expr: &Expr, env: &dyn Env) -> Result<Val, String> {
    let mut s = Scope { vars: Vec::new(), env, steps: 0, arithmetic_left: Cell::new(Number::MAX_ARITHMETIC_WORK) };
    s.ev(expr)
}

/// Parses and evaluates in one go.
pub fn run(src: &str, env: &dyn Env) -> Result<Val, String> {
    eval(&parse(src)?, env)
}

fn num(v: &Val, what: &str) -> Result<Number, String> {
    match v {
        Val::Num(n) => Ok(n.clone()),
        Val::Bool(b) => Ok(Number::integer(u8::from(*b))),
        Val::Str(s) => Number::parse(s)?.ok_or_else(|| format!("{what}: \"{s}\" is not a number")),
        Val::Null => Ok(Number::integer(0)),
        other => Err(format!("{what}: expected a number, got {}", other.type_name())),
    }
}

fn date_of(v: &Val, what: &str) -> Result<DateVal, String> {
    match v {
        Val::Date(d) => Ok(*d),
        Val::Str(s) => parse_date(s).ok_or_else(|| format!("{what}: \"{s}\" is not a date")),
        Val::Num(n) => Ok(DateVal { start: n.try_as_f64()? as i64, end: None, has_time: true }),
        other => Err(format!("{what}: expected a date, got {}", other.type_name())),
    }
}

fn list_of(v: &Val) -> Vec<Val> {
    match v {
        Val::List(l) => l.clone(),
        Val::Null => Vec::new(),
        other => vec![other.clone()],
    }
}

fn unit_minutes(unit: &str) -> Result<i64, String> {
    Ok(match unit {
        "minutes" | "minute" => 1,
        "hours" | "hour" => 60,
        "days" | "day" => MINUTES_PER_DAY,
        "weeks" | "week" => 7 * MINUTES_PER_DAY,
        other => return Err(format!("Unknown time unit {other}")),
    })
}

fn date_add(d: DateVal, n: f64, unit: &str) -> Result<DateVal, String> {
    let n = n.round() as i64;
    let shift = |m: i64| -> Result<i64, String> {
        Ok(match unit {
            "months" | "month" => add_months(m, n),
            "quarters" | "quarter" => add_months(m, 3 * n),
            "years" | "year" => add_months(m, 12 * n),
            u => m + n * unit_minutes(u)?,
        })
    };
    let has_time = d.has_time || matches!(unit, "minutes" | "minute" | "hours" | "hour");
    Ok(DateVal { start: shift(d.start)?, end: d.end.map(&shift).transpose()?, has_time })
}

fn date_between(a: DateVal, b: DateVal, unit: &str) -> Result<f64, String> {
    let (x, y) = (a.start, b.start);
    Ok(match unit {
        "months" | "month" => {
            let (y1, m1, d1, _, _) = parts(x);
            let (y2, m2, d2, _, _) = parts(y);
            let mut n = (y1 - y2) * 12 + (m1 - m2);
            if n > 0 && d1 < d2 {
                n -= 1
            } else if n < 0 && d1 > d2 {
                n += 1
            }
            n as f64
        }
        "quarters" | "quarter" => (date_between(a, b, "months")? / 3.0).trunc(),
        "years" | "year" => (date_between(a, b, "months")? / 12.0).trunc(),
        u => ((x - y) / unit_minutes(u)?) as f64,
    })
}

impl<'a> Scope<'a> {
    fn charge_arithmetic(&self, work: usize) -> Result<(), String> {
        let remaining = self.arithmetic_left.get().checked_sub(work)
            .ok_or("The formula exceeds its arithmetic work budget")?;
        self.arithmetic_left.set(remaining);
        Ok(())
    }

    fn binary(&mut self, op: &str, l: Val, r: Val) -> Result<Val, String> {
        binary(op, l, r, |work| self.charge_arithmetic(work))
    }

    fn lookup(&self, n: &str) -> Option<Val> {
        self.vars.iter().rev().find(|(k, _)| k == n).map(|(_, v)| v.clone())
    }

    fn ev(&mut self, e: &Expr) -> Result<Val, String> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err("The formula is too complicated".into());
        }
        Ok(match e {
            Expr::Num(n) => Val::Num(n.clone()),
            Expr::Str(s) => Val::Str(s.clone()),
            Expr::Bool(b) => Val::Bool(*b),
            Expr::Ident(n) => match self.lookup(n) {
                Some(v) => v,
                None => match n.as_str() {
                    "e" => Val::from_float(std::f64::consts::E)?,
                    "pi" => Val::from_float(std::f64::consts::PI)?,
                    other => return Err(format!("Unknown name {other}")),
                },
            },
            Expr::List(v) => {
                let mut out = Vec::new();
                for x in v {
                    out.push(self.ev(x)?);
                }
                Val::List(out)
            }
            Expr::Unary(c, x) => {
                let v = self.ev(x)?;
                if *c == '-' {
                    Val::Num(num(&v, "-")?.neg_checked(&mut |work| self.charge_arithmetic(work))?)
                } else {
                    Val::Bool(!v.bool_checked(&mut |work| self.charge_arithmetic(work))?)
                }
            }
            Expr::Ternary(c, a, b) => {
                if self.ev(c)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                    self.ev(a)?
                } else {
                    self.ev(b)?
                }
            }
            Expr::Binary(op, a, b) => {
                // and / or short-circuit.
                if op == "and" || op == "&&" {
                    let l = self.ev(a)?;
                    return Ok(if !l.bool_checked(&mut |work| self.charge_arithmetic(work))? { Val::Bool(false) } else { Val::Bool(self.ev(b)?.bool_checked(&mut |work| self.charge_arithmetic(work))?) });
                }
                if op == "or" || op == "||" {
                    let l = self.ev(a)?;
                    return Ok(if l.bool_checked(&mut |work| self.charge_arithmetic(work))? { Val::Bool(true) } else { Val::Bool(self.ev(b)?.bool_checked(&mut |work| self.charge_arithmetic(work))?) });
                }
                let l = self.ev(a)?;
                let r = self.ev(b)?;
                self.binary(op, l, r)?
            }
            Expr::Method(recv, name, args) => {
                let mut all = vec![(**recv).clone()];
                all.extend(args.iter().cloned());
                self.call(name, &all)?
            }
            Expr::Call(name, args) => self.call(name, args)?,
        })
    }

    /// Evaluates `body` with `current` and `index` bound (and the named variables, if given).
    fn lambda(&mut self, body: &Expr, current: Val, index: usize, names: &[String]) -> Result<Val, String> {
        let before = self.vars.len();
        self.vars.push(("current".into(), current.clone()));
        self.vars.push(("index".into(), Val::integer(index)));
        if let Some(n) = names.first() {
            self.vars.push((n.clone(), current));
        }
        if let Some(n) = names.get(1) {
            self.vars.push((n.clone(), Val::integer(index)));
        }
        let r = self.ev(body);
        self.vars.truncate(before);
        r
    }

    fn lambda_names(args: &[Expr]) -> (Vec<String>, usize) {
        // map(list, current -> ...) is not supported; Notion 2.0 uses `current` and `index`. Variable names are
        // taken from `let`-style first arguments only for the functions that document them.
        let _ = args;
        (Vec::new(), 1)
    }

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<Val, String> {
        // Functions that need lazy arguments.
        match name {
            "if" => {
                need(name, args, 2, 3)?;
                return if self.ev(&args[0])?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                    self.ev(&args[1])
                } else if args.len() == 3 {
                    self.ev(&args[2])
                } else {
                    Ok(Val::Null)
                };
            }
            "ifs" => {
                let mut i = 0;
                while i + 1 < args.len() {
                    if self.ev(&args[i])?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                        return self.ev(&args[i + 1]);
                    }
                    i += 2;
                }
                return if i < args.len() { self.ev(&args[i]) } else { Ok(Val::Null) };
            }
            "and" => {
                for a in args {
                    if !self.ev(a)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                        return Ok(Val::Bool(false));
                    }
                }
                return Ok(Val::Bool(true));
            }
            "or" => {
                for a in args {
                    if self.ev(a)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                        return Ok(Val::Bool(true));
                    }
                }
                return Ok(Val::Bool(false));
            }
            "let" => {
                need(name, args, 3, 3)?;
                let n = match &args[0] {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err("let: the first argument must be a name".into()),
                };
                let v = self.ev(&args[1])?;
                self.vars.push((n, v));
                let r = self.ev(&args[2]);
                self.vars.pop();
                return r;
            }
            "lets" => {
                if args.len() < 3 || args.len() % 2 == 0 {
                    return Err("lets: name, value pairs and then the expression".into());
                }
                let before = self.vars.len();
                let mut i = 0;
                while i + 1 < args.len() {
                    let n = match &args[i] {
                        Expr::Ident(n) => n.clone(),
                        _ => {
                            self.vars.truncate(before);
                            return Err("lets: names must be plain names".into());
                        }
                    };
                    let v = self.ev(&args[i + 1]);
                    match v {
                        Ok(v) => self.vars.push((n, v)),
                        Err(e) => {
                            self.vars.truncate(before);
                            return Err(e);
                        }
                    }
                    i += 2;
                }
                let r = self.ev(&args[args.len() - 1]);
                self.vars.truncate(before);
                return r;
            }
            "map" | "filter" | "some" | "every" | "find" | "findIndex" | "sort" if name != "sort" || args.len() == 2 => {
                need(name, args, 2, 2)?;
                let list = list_of(&self.ev(&args[0])?);
                if list.len() > MAX_LIST {
                    return Err("The list is too long".into());
                }
                let (names, _) = Self::lambda_names(args);
                match name {
                    "map" => {
                        let mut out = Vec::new();
                        for (i, x) in list.into_iter().enumerate() {
                            out.push(self.lambda(&args[1], x, i, &names)?);
                        }
                        return Ok(Val::List(out));
                    }
                    "filter" => {
                        let mut out = Vec::new();
                        for (i, x) in list.into_iter().enumerate() {
                            if self.lambda(&args[1], x.clone(), i, &names)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                                out.push(x);
                            }
                        }
                        return Ok(Val::List(out));
                    }
                    "some" => {
                        for (i, x) in list.into_iter().enumerate() {
                            if self.lambda(&args[1], x, i, &names)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                                return Ok(Val::Bool(true));
                            }
                        }
                        return Ok(Val::Bool(false));
                    }
                    "every" => {
                        for (i, x) in list.into_iter().enumerate() {
                            if !self.lambda(&args[1], x, i, &names)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                                return Ok(Val::Bool(false));
                            }
                        }
                        return Ok(Val::Bool(true));
                    }
                    "find" => {
                        for (i, x) in list.into_iter().enumerate() {
                            if self.lambda(&args[1], x.clone(), i, &names)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                                return Ok(x);
                            }
                        }
                        return Ok(Val::Null);
                    }
                    "findIndex" => {
                        for (i, x) in list.into_iter().enumerate() {
                            if self.lambda(&args[1], x, i, &names)?.bool_checked(&mut |work| self.charge_arithmetic(work))? {
                                return Ok(Val::integer(i));
                            }
                        }
                        return Ok(Val::integer(-1));
                    }
                    _ => {
                        // sort(list, key): sorts by the key computed for each element
                        let mut keyed = Vec::new();
                        for (i, x) in list.into_iter().enumerate() {
                            let k = self.lambda(&args[1], x.clone(), i, &names)?;
                            keyed.push((k, x));
                        }
                        sort_checked(&mut keyed, &mut |a, b| compare_checked(&a.0, &b.0, &mut |work| self.charge_arithmetic(work)))?;
                        return Ok(Val::List(keyed.into_iter().map(|(_, x)| x).collect()));
                    }
                }
            }
            _ => {}
        }
        // Eager functions.
        let mut v = Vec::new();
        for a in args {
            v.push(self.ev(a)?);
        }
        self.builtin(name, v)
    }

    fn builtin(&mut self, name: &str, a: Vec<Val>) -> Result<Val, String> {
        let n = a.len();
        let arg = |i: usize| -> Result<&Val, String> { a.get(i).ok_or_else(|| format!("{name}: missing argument {}", i + 1)) };
        let exact = |i: usize| -> Result<Number, String> { num(arg(i)?, name) };
        let numarg = |i: usize| -> Result<f64, String> { exact(i)?.float_checked(&mut |work| self.charge_arithmetic(work)) };
        let text = |v: &Val| v.text_checked(&mut |work| self.charge_arithmetic(work));
        let strarg = |i: usize| -> Result<String, String> { text(arg(i)?) };
        let unitarg = |i: usize| -> Result<String, String> { strarg(i) };
        let nums_all = || -> Result<Vec<Number>, String> {
            let mut out = Vec::new();
            for v in &a {
                match v {
                    Val::List(l) => {
                        for x in l {
                            out.push(num(x, name)?);
                        }
                    }
                    other => out.push(num(other, name)?),
                }
            }
            Ok(out)
        };
        match name {
            "unaryMinus" | "unaryPlus" | "abs" | "ceil" | "floor" | "round" | "sqrt"
            | "cbrt" | "exp" | "ln" | "log10" | "log2" | "sign" => {
                let value = exact(0)?;
                if !matches!(name, "sqrt" | "round") {
                    self.charge_arithmetic(value.unary_work(name))?;
                }
                let mut charge = |work| self.charge_arithmetic(work);
                return Ok(Val::Num(match name {
                    "unaryMinus" => value.neg_checked(&mut charge)?,
                    "unaryPlus" => value,
                    "abs" => value.abs(&mut charge)?,
                    "ceil" => value.ceil(&mut charge)?,
                    "floor" => value.floor(&mut charge)?,
                    "round" => {
                        let places = if n > 1 { numarg(1)? as i32 } else { 0 };
                        value.round(places, &mut charge)?
                    }
                    "sqrt" => value.sqrt(&mut charge)?,
                    "cbrt" => value.cbrt(&mut charge)?,
                    "exp" => value.exp(&mut charge)?,
                    "ln" => value.logarithm(LogBase::Natural, &mut charge)?,
                    "log10" => value.logarithm(LogBase::Ten, &mut charge)?,
                    "log2" => value.logarithm(LogBase::Two, &mut charge)?,
                    "sign" => value.sign(&mut charge)?,
                    _ => unreachable!(),
                }));
            }
            _ => {}
        }
        if name != "format" && let Some(Val::Num(value)) = a.first() {
            self.charge_arithmetic(value.unary_work(name))?;
        }
        Ok(match name {
            // ---- logic and comparison
            "not" => Val::Bool(!arg(0)?.bool_checked(&mut |work| self.charge_arithmetic(work))?),
            "empty" => Val::Bool(arg(0)?.is_empty()),
            "equal" | "unequal" | "larger" | "largerEq" | "smaller" | "smallerEq" => {
                let op = match name {
                    "equal" => "==",
                    "unequal" => "!=",
                    "larger" => ">",
                    "largerEq" => ">=",
                    "smaller" => "<",
                    _ => "<=",
                };
                Val::Bool(predicate(op, arg(0)?, arg(1)?, &mut |work| self.charge_arithmetic(work))?)
            }
            // ---- the page
            "prop" => {
                let nm = strarg(0)?;
                self.env.prop(&nm)?
            }
            "id" => Val::Str(self.env.row_id()),
            // ---- numbers
            "add" => self.binary("+", arg(0)?.clone(), arg(1)?.clone())?,
            "subtract" => self.binary("-", arg(0)?.clone(), arg(1)?.clone())?,
            "multiply" => self.binary("*", arg(0)?.clone(), arg(1)?.clone())?,
            "divide" => self.binary("/", arg(0)?.clone(), arg(1)?.clone())?,
            "mod" => self.binary("%", arg(0)?.clone(), arg(1)?.clone())?,
            "pow" => self.binary("^", arg(0)?.clone(), arg(1)?.clone())?,
            "min" | "max" | "sum" | "median" | "mean" | "average" => {
                let name = if matches!(name, "mean" | "average") { "avg" } else { name };
                calc_numbers_with(name, nums_all()?, |work| self.charge_arithmetic(work))?
            }
            "toNumber" => match arg(0)? {
                Val::Str(s) => {
                    let t: String = s.chars().filter(|c| !c.is_whitespace() && *c != ',').collect();
                    let t = t.trim_start_matches(['$', '€', '£', '¥']);
                    let t = t.strip_suffix('%').unwrap_or(t);
                    Val::Num(Number::decimal(t).map_err(|e| format!("toNumber: {e}"))?)
                }
                Val::Date(d) => Val::integer(i128::from(d.start) * 60_000),
                Val::Bool(b) => Val::integer(u8::from(*b)),
                Val::Num(x) => Val::Num(x.clone()),
                Val::Null => Val::integer(0),
                Val::List(_) => return Err("toNumber: expected a single value".into()),
            },
            // ---- text
            "length" => match arg(0)? {
                Val::List(l) => Val::integer(l.len()),
                v => Val::integer(text(v)?.chars().count()),
            },
            "concat" => {
                let mut s = String::new();
                for v in &a {
                    match v {
                        Val::List(l) => {
                            for x in l {
                                s.push_str(&text(x)?)
                            }
                        }
                        other => s.push_str(&text(other)?),
                    }
                }
                Val::Str(s)
            }
            "join" => {
                let sep = strarg(1)?;
                Val::Str(list_of(arg(0)?).iter().map(text).collect::<Result<Vec<_>, _>>()?.join(&sep))
            }
            "split" => {
                let s = strarg(0)?;
                let sep = strarg(1)?;
                if sep.is_empty() {
                    Val::List(s.chars().map(|c| Val::Str(c.to_string())).collect())
                } else {
                    Val::List(s.split(sep.as_str()).map(|p| Val::Str(p.to_string())).collect())
                }
            }
            "lower" => Val::Str(strarg(0)?.to_lowercase()),
            "upper" => Val::Str(strarg(0)?.to_uppercase()),
            "trim" => Val::Str(strarg(0)?.trim().to_string()),
            "repeat" => {
                let k = numarg(1)?.max(0.0) as usize;
                if strarg(0)?.len().saturating_mul(k) > 1_000_000 {
                    return Err("repeat: the result is too long".into());
                }
                Val::Str(strarg(0)?.repeat(k))
            }
            "padStart" | "padEnd" => {
                let s = strarg(0)?;
                let len = numarg(1)?.max(0.0).min(100_000.0) as usize;
                let pad = if n > 2 { strarg(2)? } else { " ".into() };
                let have = s.chars().count();
                if have >= len || pad.is_empty() {
                    Val::Str(s)
                } else {
                    let fill: String = pad.chars().cycle().take(len - have).collect();
                    Val::Str(if name == "padStart" { format!("{fill}{s}") } else { format!("{s}{fill}") })
                }
            }
            "substring" | "slice" => {
                let start = numarg(1)? as i64;
                let end = if n > 2 { Some(numarg(2)? as i64) } else { None };
                let cut = |len: usize| -> (usize, usize) {
                    let l = len as i64;
                    let fix = |i: i64| (if i < 0 { l + i } else { i }).clamp(0, l) as usize;
                    let s = fix(start);
                    let e = end.map_or(len, fix);
                    (s, e.max(s))
                };
                match arg(0)? {
                    Val::List(l) => {
                        let (s, e) = cut(l.len());
                        Val::List(l[s..e].to_vec())
                    }
                    v => {
                        let c: Vec<char> = text(v)?.chars().collect();
                        let (s, e) = cut(c.len());
                        Val::Str(c[s..e].iter().collect())
                    }
                }
            }
            "contains" => match arg(0)? {
                Val::List(l) => Val::Bool(includes(l, arg(1)?, &mut |work| self.charge_arithmetic(work))?),
                v => Val::Bool(text(v)?.to_lowercase().contains(&strarg(1)?.to_lowercase())),
            },
            "includes" => Val::Bool(includes(&list_of(arg(0)?), arg(1)?, &mut |work| self.charge_arithmetic(work))?),
            "test" => {
                let re = Regex::new(&strarg(1)?, false).map_err(|e| format!("test: {e}"))?;
                Val::Bool(re.is_match(&strarg(0)?))
            }
            "match" => {
                let re = Regex::new(&strarg(1)?, false).map_err(|e| format!("match: {e}"))?;
                Val::List(re.all(&strarg(0)?).into_iter().map(Val::Str).collect())
            }
            "replace" | "replaceAll" => {
                let re = Regex::new(&strarg(1)?, false).map_err(|e| format!("{name}: {e}"))?;
                Val::Str(re.replace(&strarg(0)?, &strarg(2)?, name == "replaceAll"))
            }
            "format" => Val::Str(strarg(0)?),
            "link" | "style" | "unstyle" => Val::Str(strarg(0)?),
            // ---- dates
            "now" => Val::Date(DateVal { start: self.env.now(), end: None, has_time: true }),
            "today" => Val::Date(DateVal { start: start_of_day(self.env.now()), end: None, has_time: false }),
            // An empty date gives an empty result (as in Notion), not an error.
            "year" | "month" | "date" | "day" | "hour" | "minute" | "start" | "dateStart" | "end" | "dateEnd" | "dateAdd" | "dateSubtract" | "formatDate" | "timestamp"
                if matches!(arg(0)?, Val::Null) =>
            {
                Val::Null
            }
            "dateBetween" if matches!(arg(0)?, Val::Null) || matches!(arg(1)?, Val::Null) => Val::Null,
            "year" => Val::integer(parts(date_of(arg(0)?, name)?.start).0),
            "month" => Val::integer(parts(date_of(arg(0)?, name)?.start).1),
            "date" => Val::integer(parts(date_of(arg(0)?, name)?.start).2),
            "day" => Val::integer(weekday(date_of(arg(0)?, name)?.start) + 1),
            "hour" => Val::integer(parts(date_of(arg(0)?, name)?.start).3),
            "minute" => Val::integer(parts(date_of(arg(0)?, name)?.start).4),
            "start" | "dateStart" => {
                let d = date_of(arg(0)?, name)?;
                Val::Date(DateVal { start: d.start, end: None, has_time: d.has_time })
            }
            "end" | "dateEnd" => {
                let d = date_of(arg(0)?, name)?;
                Val::Date(DateVal { start: d.end.unwrap_or(d.start), end: None, has_time: d.has_time })
            }
            "dateRange" => {
                let s = date_of(arg(0)?, name)?;
                let e = date_of(arg(1)?, name)?;
                Val::Date(DateVal { start: s.start, end: Some(e.start), has_time: s.has_time || e.has_time })
            }
            "dateAdd" => Val::Date(date_add(date_of(arg(0)?, name)?, numarg(1)?, &unitarg(2)?)?),
            "dateSubtract" => Val::Date(date_add(date_of(arg(0)?, name)?, -numarg(1)?, &unitarg(2)?)?),
            "dateBetween" => Val::from_float(date_between(date_of(arg(0)?, name)?, date_of(arg(1)?, name)?, &unitarg(2)?)?)?,
            "formatDate" => Val::Str(format_pattern(date_of(arg(0)?, name)?.start, &strarg(1)?)),
            "parseDate" | "toDate" => match arg(0)? {
                Val::Date(d) => Val::Date(*d),
                v => {
                    let value = text(v)?;
                    Val::Date(parse_date(&value).ok_or_else(|| format!("{name}: \"{value}\" is not a date"))?)
                }
            },
            "timestamp" => Val::integer(i128::from(date_of(arg(0)?, name)?.start) * 60_000),
            "fromTimestamp" => Val::Date(DateVal { start: (numarg(0)? / 60_000.0).floor() as i64, end: None, has_time: true }),
            // ---- lists
            "at" => {
                let l = list_of(arg(0)?);
                let i = numarg(1)? as i64;
                let i = if i < 0 { l.len() as i64 + i } else { i };
                if i < 0 || i as usize >= l.len() {
                    Val::Null
                } else {
                    l[i as usize].clone()
                }
            }
            "first" => list_of(arg(0)?).first().cloned().unwrap_or(Val::Null),
            "last" => list_of(arg(0)?).last().cloned().unwrap_or(Val::Null),
            "reverse" => {
                let mut l = list_of(arg(0)?);
                l.reverse();
                Val::List(l)
            }
            "sort" => {
                let mut l = list_of(arg(0)?);
                sort_checked(&mut l, &mut |a, b| compare_checked(a, b, &mut |work| self.charge_arithmetic(work)))?;
                Val::List(l)
            }
            "unique" => distinct("showUnique", std::slice::from_ref(arg(0)?), &mut |work| self.charge_arithmetic(work))?,
            "flat" => {
                let mut out = Vec::new();
                for x in list_of(arg(0)?) {
                    match x {
                        Val::List(inner) => out.extend(inner),
                        other => out.push(other),
                    }
                }
                Val::List(out)
            }
            "count" => Val::integer(list_of(arg(0)?).len()),
            other => return Err(format!("Unknown function {other}")),
        })
    }
}

fn need(name: &str, args: &[Expr], lo: usize, hi: usize) -> Result<(), String> {
    if args.len() < lo || args.len() > hi {
        return Err(format!("{name} needs {lo}{} argument(s)", if hi != lo { format!(" to {hi}") } else { String::new() }));
    }
    Ok(())
}

/// An empty value next to a number, text or checkbox counts as 0, "" or unchecked (as in Notion:
/// an empty number is not greater than 5). Next to another empty value or a date it stays empty.
fn fill_empty(l: &Val, r: &Val) -> (Val, Val) {
    let zero = |other: &Val| match other {
        Val::Num(_) => Some(Val::integer(0)),
        Val::Str(_) => Some(Val::Str(String::new())),
        Val::Bool(_) => Some(Val::Bool(false)),
        Val::List(_) => Some(Val::List(vec![])),
        _ => None,
    };
    match (l, r) {
        (Val::Null, o) => (zero(o).unwrap_or(Val::Null), o.clone()),
        (o, Val::Null) => (o.clone(), zero(o).unwrap_or(Val::Null)),
        _ => (l.clone(), r.clone()),
    }
}

fn predicate(op: &str, l: &Val, r: &Val, charge: &mut Charge<'_>) -> Result<bool, String> {
    Ok(match op {
        "==" => same_checked(l, r, charge)?,
        "!=" => !same_checked(l, r, charge)?,
        "<" => compare_checked(l, r, charge)?.is_lt(),
        ">" => compare_checked(l, r, charge)?.is_gt(),
        "<=" => compare_checked(l, r, charge)?.is_le(),
        _ => compare_checked(l, r, charge)?.is_ge(),
    })
}

fn includes(list: &[Val], value: &Val, charge: &mut Charge<'_>) -> Result<bool, String> {
    for item in list {
        if same_checked(item, value, charge)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn binary(op: &str, l: Val, r: Val, mut charge: impl FnMut(usize) -> Result<(), String>) -> Result<Val, String> {
    if matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=") {
        let (l, r) = fill_empty(&l, &r);
        return Ok(Val::Bool(predicate(op, &l, &r, &mut charge)?));
    }
    let special = match op {
        "+" => match (&l, &r) {
            (Val::Date(d), Val::Num(n)) | (Val::Num(n), Val::Date(d)) => Some(Val::Date(date_add(*d, n.float_checked(&mut charge)?, "days")?)),
            (Val::Str(_), _) | (_, Val::Str(_)) => Some(Val::Str(format!("{}{}", l.text_checked(&mut charge)?, r.text_checked(&mut charge)?))),
            (Val::List(x), Val::List(y)) => Some(Val::List(x.iter().chain(y.iter()).cloned().collect())),
            _ => None,
        },
        "-" => match (&l, &r) {
            (Val::Date(a), Val::Date(b)) => Some(Val::Num(Number::integer(i128::from(a.start) - i128::from(b.start)).div(&Number::integer(MINUTES_PER_DAY))?)),
            (Val::Date(d), Val::Num(n)) => Some(Val::Date(date_add(*d, -n.float_checked(&mut charge)?, "days")?)),
            _ => None,
        },
        _ => None,
    };
    if let Some(v) = special {
        return Ok(v);
    }
    let a = num(&l, op)?;
    let b = num(&r, op)?;
    if op != "%" {
        charge(a.arithmetic_work(op, &b))?;
    }
    Ok(Val::Num(match op {
        "+" => a.binary_checked(Binary::Add, &b, &mut charge)?,
        "-" => a.binary_checked(Binary::Sub, &b, &mut charge)?,
        "*" => a.binary_checked(Binary::Mul, &b, &mut charge)?,
        "/" => a.binary_checked(Binary::Div, &b, &mut charge)?,
        "%" => a.rem(&b, &mut charge)?,
        "^" => a.pow(&b, &mut charge)?,
        other => return Err(format!("Unknown operator {other}")),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct E(HashMap<String, Val>);

    #[test]
    fn exact_square_root_keeps_decimal_equality() {
        let env = E(HashMap::new());
        assert_eq!(run("sqrt(0.01) == 0.1", &env), Ok(Val::Bool(true)));
    }

    #[test]
    fn fractional_power_and_square_root_boundaries_reach_formula_output() {
        let env = E(HashMap::new());
        for (expression, expected) in [
            ("(5.410860736180795545311869365348484425069e-423)^(-0.73)", "1.79769313486e308"),
            ("(1.00000000001999999999999e-800)^0.25", "1e-200"),
            ("(9.999999999799996e-801)^(-0.25)", "1.00000000001e200"),
            ("sqrt(1e-700)", "1e-350"),
        ] {
            assert_eq!(run(expression, &env).unwrap().text(), expected, "{expression}");
        }
        assert_eq!(
            run("(4.326339818212956088414698819550503504786e-441)^(-0.7)", &env),
            Err("Numeric overflow".into())
        );
    }

    #[test]
    fn roots_resolve_exact_overflow_and_significant_digit_midpoints() {
        let env = E(HashMap::new());
        let max_neighbor = "9574977460952189610090905978696373101655155263624144103513409188430681395619728961152731367650476052030729";
        assert_eq!(
            run(&format!("({max_neighbor}6e-1340)^(-0.25)"), &env),
            Err("Numeric overflow".into())
        );
        assert_eq!(
            run(&format!("({max_neighbor}7e-1340)^(-0.25)"), &env).unwrap().text(),
            "1.79769313486e308"
        );

        for (expression, expected) in [
            ("(23230572289234306275384052137556256960158691406250000000000000000000000000000000000000000000000000001e-900)^0.25", "1.23456789013e-200"),
            ("((1.234567890125e-200)^4 - 1e-900)^0.25", "1.23456789012e-200"),
            ("((1.234567890125e-200)^5)^0.2", "1.23456789013e-200"),
            ("sqrt(15241578753276940725156250000000000000000000000000000000000000000000000000000000000000000000000000001e-800)", "1.23456789013e-350"),
            ("sqrt((1.234567890125e-350)^2 - 1e-800)", "1.23456789012e-350"),
        ] {
            assert_eq!(run(expression, &env).unwrap().text(), expected, "{expression}");
        }
        assert_eq!(
            run("((1.234567890125e-200)^5)^0.2 == 1.234567890125e-200", &env),
            Ok(Val::Bool(true))
        );
    }

    #[test]
    fn approximate_formula_predicates_and_integer_operations_are_certified() {
        let env = E(HashMap::new());
        for expression in [
            "sqrt(1+1e-100) > 1",
            "sqrt(1-1e-100) < 1",
            "sqrt(1+1e-100) != 1",
            "sqrt(2) == sqrt(2)",
            "length(unique([sqrt(2),sqrt(2)])) == 1",
            "largerEq(sqrt(1+1e-100),1)",
            "min(sqrt(1+1e-100),1) == 1",
            "max(-sqrt(1+1e-100),-1) == -1",
            "floor(sqrt(1-1e-100)) == 0",
            "ceil(sqrt(1+1e-100)) == 2",
            "round(sqrt(0.005^2-1e-90),2) == 0",
            "round(-sqrt(0.005^2+1e-90),2) == -0.01",
            "sign(sqrt(1+1e-100)-1) == 1",
            "mod(sqrt(1+1e-100),1) > 0",
        ] {
            assert_eq!(run(expression, &env), Ok(Val::Bool(true)), "{expression}");
        }
        for expression in ["sqrt(2)-sqrt(2) == 0", "1/(sqrt(2)-sqrt(2))", "sqrt(2)%sqrt(2)", "format(sqrt(2)-sqrt(2))"] {
            assert_eq!(run(expression, &env).unwrap_err(), "The formula exceeds its arithmetic work budget", "{expression}");
        }
        assert_eq!(
            run("sqrt(2)-sqrt(2-1e-200)", &env).unwrap().try_text().unwrap(),
            "3.53553390593e-201"
        );
    }

    #[test]
    fn repeated_fractional_powers_with_exact_roots_hit_the_arithmetic_budget() {
        let env = E(HashMap::new());
        assert_eq!(
            run("1.01568603515625^1000.5 == (129/128)^2001", &env),
            Ok(Val::Bool(true))
        );
        let items = ["0"; 32].join(",");
        let expression = format!("lets(items,[{items}],b,1.01568603515625,length(map(items,b^1000.5)))");
        assert_eq!(
            run(&expression, &env).unwrap_err(),
            "The formula exceeds its arithmetic work budget"
        );
    }

    #[test]
    fn wide_square_root_charges_numeric_strings_and_numbers_equally() {
        let env = E(HashMap::new());
        let root = Val::Num(Number::decimal("1e-2200").unwrap());
        for input in ["1e-4400", "\"1e-4400\""] {
            assert_eq!(run(&format!("sqrt({input})"), &env), Ok(root.clone()));
            let repeated = format!("map([0,0],sqrt({input}))");
            assert_eq!(
                run(&repeated, &env).unwrap_err(),
                "The formula exceeds its arithmetic work budget"
            );
        }
    }

    #[test]
    fn exact_formatting_charges_both_implicit_and_explicit_projections() {
        let zeros = ["0"; 64].join(",");
        let env = E(HashMap::new());
        for body in [
            format!(r#"length(join(map([{zeros}], a), ""))"#),
            format!("length(map([{zeros}], format(a)))"),
        ] {
            let expression = format!("let(a, 3.14e-1000, {body})");
            assert_eq!(run(&expression, &env).unwrap_err(), "The formula exceeds its arithmetic work budget");
        }
        assert_eq!(
            run("format(3.14e-1000)", &env),
            Ok(Val::Str("3.14e-1000".into()))
        );
        // 16 projections are within budget only when each projection is charged once.
        let zeros = ["0"; 16].join(",");
        let expression = format!("let(a, 3.14e-1000, length(map([{zeros}], format(a))))");
        assert_eq!(run(&expression, &env), Ok(Val::integer(16)));
    }

    #[test]
    fn transcendental_formula_projections_preserve_compound_precision() {
        let env = E(HashMap::new());
        for (expression, expected) in [
            ("ln(sqrt(1+1e-100))", "5e-101"),
            ("ln(sqrt(1-1e-100))", "-5e-101"),
            ("exp(sqrt(1e-200+1e-250))-1", "1e-100"),
            ("exp(-sqrt(1e-200+1e-250))-1", "-1e-100"),
            ("ln(1)", "0"),
            ("exp(0)", "1"),
            ("exp(1)", "2.71828182846"),
            ("exp(-745)", "2.82235073047e-324"),
            ("exp(-746)", "1.03828480952e-324"),
            ("exp(-750)*1e308", "1.90168496348e-18"),
            ("exp(-750)/1e-325", "0.190168496348"),
            ("ln(exp(-750))", "-750"),
            ("ln(exp(-sqrt(562500+1e-100)))+750", "-6.66666666667e-104"),
            ("log2(1/1024)", "-10"),
            ("log10(1e-1000)", "-1000"),
            ("log2(sqrt(1+1e-100))", "7.21347520444e-101"),
            ("log10(sqrt(1+1e-100))", "2.17147240952e-101"),
            ("cbrt(-8/27)+2/3", "0"),
            ("cbrt(-2)", "-1.25992104989"),
            ("cbrt(sqrt(1+1e-100))-1", "1.66666666667e-101"),
        ] {
            let actual = run(expression, &env).unwrap().try_text().unwrap();
            assert_eq!(actual, expected, "{expression}");
        }
        let Val::Num(amount) = run("exp(-746)*1e308*1e15", &env).unwrap() else {
            panic!("expected a number");
        };
        assert_eq!(format_number(&amount, "dollar").unwrap(), "$0.1");
        let Val::Num(tiny) = run("exp(-746)", &env).unwrap() else {
            panic!("expected a number");
        };
        assert_eq!(tiny.try_as_f64().unwrap().to_bits(), 0);
        for (expression, error) in [
            ("ln(sqrt(1-1e-100)-1)", "Not a real number"),
            ("log2(0)", "Not a real number"),
            ("log10(-1)", "Not a real number"),
            ("exp(710)", "Numeric overflow"),
            ("-exp(710)", "Numeric overflow"),
            ("exp(-1e308)", "Numeric precision is out of range"),
            ("exp(-sqrt(1e100+1))", "Numeric precision is out of range"),
        ] {
            assert_eq!(run(expression, &env).unwrap_err(), error, "{expression}");
        }
    }

    #[test]
    fn repeated_wide_products_hit_the_arithmetic_budget() {
        let env = E(HashMap::new());
        let once = "lets(a,(2/7)^2400,b,(3/5)^2400,a*b)";
        assert!(matches!(run(once, &env), Ok(Val::Num(n)) if !n.is_zero()));
        let list = vec!["0"; 32].join(",");
        let repeated = format!("lets(a,(2/7)^2400,b,(3/5)^2400,map([{list}],a*b))");
        assert_eq!(run(&repeated, &env).unwrap_err(), "The formula exceeds its arithmetic work budget");
    }

    impl Env for E {
        fn prop(&self, name: &str) -> Result<Val, String> {
            self.0.get(name).cloned().ok_or_else(|| format!("No property named {name}"))
        }
        fn row_id(&self) -> String {
            "row1".into()
        }
        fn now(&self) -> i64 {
            from_parts(2026, 11, 3, 15, 30)
        }
    }

    fn env() -> E {
        let mut m = HashMap::new();
        m.insert("Price".into(), Val::Num(Number::decimal("12.5").unwrap()));
        m.insert("Qty".into(), Val::integer(4));
        m.insert("Name".into(), Val::Str("Widget".into()));
        m.insert("Tags".into(), Val::List(vec![Val::Str("a".into()), Val::Str("b".into()), Val::Str("c".into())]));
        m.insert("Due".into(), Val::Date(parse_date("2026-11-10").unwrap()));
        m.insert("Done".into(), Val::Bool(true));
        E(m)
    }

    fn ok(src: &str) -> Val {
        run(src, &env()).unwrap_or_else(|e| panic!("{src}: {e}"))
    }

    #[test]
    fn arithmetic_and_precedence() {
        assert_eq!(ok("1 + 2 * 3"), Val::integer(7));
        assert_eq!(ok("(1 + 2) * 3"), Val::integer(9));
        assert_eq!(ok("2 ^ 3 ^ 2"), Val::integer(512));
        assert_eq!(ok("-2 ^ 2"), Val::integer(-4));
        assert_eq!(ok("10 % 4"), Val::integer(2));
        assert_eq!(ok("0.1 + 0.2"), Val::Num(Number::decimal("0.3").unwrap()));
        assert_eq!(ok("prop(\"Price\") * prop(\"Qty\")"), Val::integer(50));
        assert!(run("1 / 0", &env()).is_err());
    }

    #[test]
    fn logic_comparison_and_conditionals() {
        assert_eq!(ok("1 < 2 and 3 >= 3"), Val::Bool(true));
        assert_eq!(ok("not true or false"), Val::Bool(false));
        assert_eq!(ok("1 == 1 ? \"yes\" : \"no\""), Val::Str("yes".into()));
        assert_eq!(ok("if(prop(\"Done\"), \"done\", \"open\")"), Val::Str("done".into()));
        assert_eq!(ok("ifs(false, 1, false, 2, 3)"), Val::integer(3));
        assert_eq!(ok("empty(\"\")"), Val::Bool(true));
        assert_eq!(ok("\"A\" == \"a\""), Val::Bool(true));
    }

    #[test]
    fn text_functions_in_both_call_styles() {
        assert_eq!(ok("upper(prop(\"Name\"))"), Val::Str("WIDGET".into()));
        assert_eq!(ok("prop(\"Name\").upper().length()"), Val::integer(6));
        assert_eq!(ok("\"abc\" + 1"), Val::Str("abc1".into()));
        assert_eq!(ok("substring(\"hello\", 1, 3)"), Val::Str("el".into()));
        assert_eq!(ok("slice(\"hello\", -3)"), Val::Str("llo".into()));
        assert_eq!(ok("replaceAll(\"a-b-c\", \"-\", \"+\")"), Val::Str("a+b+c".into()));
        assert_eq!(ok("test(\"abc123\", \"\\\\d+\")"), Val::Bool(true));
        assert_eq!(ok("join(split(\"a,b\", \",\"), \"/\")"), Val::Str("a/b".into()));
        assert_eq!(ok("padStart(\"7\", 3, \"0\")"), Val::Str("007".into()));
        assert_eq!(ok("toNumber(\"$1,200.5\")"), Val::Num(Number::decimal("1200.5").unwrap()));
        assert_eq!(ok("contains(\"Hello\", \"ELL\")"), Val::Bool(true));
    }

    #[test]
    fn number_functions() {
        assert_eq!(ok("round(2.5)"), Val::integer(3));
        assert_eq!(ok("round(3.14159, 2)"), Val::Num(Number::decimal("3.14").unwrap()));
        assert_eq!(ok("max(1, 5, 3)"), Val::integer(5));
        assert_eq!(ok("sum(prop(\"Qty\"), 1, 2)"), Val::integer(7));
        assert_eq!(ok("sqrt(16) + abs(-2) + ceil(0.2) + floor(1.9)"), Val::integer(8));
        assert_eq!(ok("pi > 3.14 and e > 2.7"), Val::Bool(true));
    }

    #[test]
    fn date_functions() {
        assert_eq!(ok("year(prop(\"Due\"))"), Val::integer(2026));
        assert_eq!(ok("dateBetween(prop(\"Due\"), today(), \"days\")"), Val::integer(7));
        assert_eq!(ok("formatDate(dateAdd(prop(\"Due\"), 1, \"months\"), \"YYYY-MM-DD\")"), Val::Str("2026-12-10".into()));
        assert_eq!(ok("formatDate(now(), \"HH:mm\")"), Val::Str("15:30".into()));
        assert_eq!(ok("day(parseDate(\"2026-11-03\"))"), Val::integer(2));
        assert_eq!(ok("formatDate(prop(\"Due\") - 3, \"D MMM\")"), Val::Str("7 Nov".into()));
        assert_eq!(ok("prop(\"Due\") > today()"), Val::Bool(true));
    }

    #[test]
    fn lists_and_lambdas() {
        assert_eq!(ok("length(prop(\"Tags\"))"), Val::integer(3));
        assert_eq!(ok("join(map(prop(\"Tags\"), upper(current)), \"\")"), Val::Str("ABC".into()));
        assert_eq!(ok("filter([1,2,3,4], current % 2 == 0)"), Val::List(vec![Val::integer(2), Val::integer(4)]));
        assert_eq!(ok("prop(\"Tags\").map(current + index).join(\",\")"), Val::Str("a0,b1,c2".into()));
        assert_eq!(ok("some([1,2], current > 1)"), Val::Bool(true));
        assert_eq!(ok("every([1,2], current > 1)"), Val::Bool(false));
        assert_eq!(ok("find([5,6,7], current > 5)"), Val::integer(6));
        assert_eq!(ok("first(prop(\"Tags\")) + last(prop(\"Tags\"))"), Val::Str("ac".into()));
        assert_eq!(ok("at([1,2,3], -1)"), Val::integer(3));
        assert_eq!(ok("sort([3,1,2])"), Val::List(vec![Val::integer(1), Val::integer(2), Val::integer(3)]));
        assert_eq!(ok("sum(map([1,2,3], current * 2))"), Val::integer(12));
        assert_eq!(ok("length(unique([\"a\",\"A\",\"b\"]))"), Val::integer(2));
    }

    #[test]
    fn let_binds_names() {
        assert_eq!(ok("let(x, 5, x * x)"), Val::integer(25));
        assert_eq!(ok("lets(a, 2, b, a + 1, a * b)"), Val::integer(6));
    }

    #[test]
    fn errors_are_messages_not_crashes() {
        for bad in ["1 +", "(1", "foo(", "\"abc", "prop(\"Nope\")", "nofn(1)", "1 @ 2", "[1,", "if()"] {
            assert!(run(bad, &env()).is_err(), "{bad}");
        }
        // an endless-looking formula stays bounded
        let long = format!("{}1", "1 + ".repeat(100));
        assert_eq!(run(&long, &env()).unwrap(), Val::integer(101));
        let too_long = format!("{}1", "1 + ".repeat(5000));
        assert!(run(&too_long, &env()).is_err());
        let deep = format!("{}1{}", "(".repeat(200), ")".repeat(200));
        assert!(run(&deep, &env()).is_err());
    }
}
