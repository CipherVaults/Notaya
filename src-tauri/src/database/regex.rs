//! A tiny backtracking regular-expression matcher for the formula functions `test`, `match`,
//! `replace` and `replaceAll` (no outside crate). Supports literals, `.`, classes `[a-z]` `[^x]`,
//! `\d \w \s \D \W \S`, anchors `^ $`, groups `( )` and `(?: )`, alternation `|`, and the
//! repeats `* + ? {n,m}` (greedy, plus lazy with a trailing `?`). Matching is bounded by a step
//! budget so a nasty pattern cannot hang the app.

#[derive(Debug, Clone)]
enum Node {
    Char(char),
    Any,
    Class(Vec<ClassItem>, bool),
    Start,
    End,
    Group(Box<Node>, Option<usize>),
    Alt(Vec<Node>),
    Seq(Vec<Node>),
    Repeat(Box<Node>, usize, usize, bool),
}

#[derive(Debug, Clone)]
enum ClassItem {
    Range(char, char),
    Digit(bool),
    Word(bool),
    Space(bool),
}

pub struct Regex {
    root: Node,
    groups: usize,
    ignore_case: bool,
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    groups: usize,
    _src: &'a str,
}

const STEP_BUDGET: usize = 2_000_000;
/// How deep the matcher may nest. Each level costs stack, and a stack overflow cannot be caught: it ends
/// the whole app. So a short text is matched where we are (a Windows main thread has only 1 MB, hence
/// the small limit), and a longer text is matched on a helper thread with a big stack and a bigger limit.
/// When even that is not enough the pattern simply does not match.
const SMALL_DEPTH: usize = 500;
const BIG_DEPTH: usize = 30_000;
const SMALL_TEXT: usize = 100;
const BIG_STACK: usize = 64 * 1024 * 1024;

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn alt(&mut self, depth: usize) -> Result<Node, String> {
        if depth > 50 {
            return Err("pattern nested too deeply".into());
        }
        let mut alts = vec![self.seq(depth)?];
        while self.peek() == Some('|') {
            self.pos += 1;
            alts.push(self.seq(depth)?);
        }
        Ok(if alts.len() == 1 { alts.pop().unwrap() } else { Node::Alt(alts) })
    }
    fn seq(&mut self, depth: usize) -> Result<Node, String> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.atom(depth)?;
            items.push(self.repeat(atom)?);
        }
        Ok(Node::Seq(items))
    }
    fn repeat(&mut self, atom: Node) -> Result<Node, String> {
        let (min, max) = match self.peek() {
            Some('*') => (0, usize::MAX),
            Some('+') => (1, usize::MAX),
            Some('?') => (0, 1),
            Some('{') => {
                let save = self.pos;
                self.pos += 1;
                let mut a = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        a.push(c);
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let mut b = None;
                if self.peek() == Some(',') {
                    self.pos += 1;
                    let mut t = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            t.push(c);
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                    b = Some(t);
                }
                if self.peek() != Some('}') || a.is_empty() {
                    self.pos = save;
                    return Ok(atom); // a literal '{'
                }
                let lo: usize = a.parse().map_err(|_| "bad repeat")?;
                let hi = match b {
                    None => lo,
                    Some(t) if t.is_empty() => usize::MAX,
                    Some(t) => t.parse().map_err(|_| "bad repeat")?,
                };
                if lo > 1000 || (hi != usize::MAX && hi > 1000) || hi < lo {
                    return Err("bad repeat".into());
                }
                (lo, hi)
            }
            _ => return Ok(atom),
        };
        self.pos += 1;
        let lazy = if self.peek() == Some('?') {
            self.pos += 1;
            true
        } else {
            false
        };
        Ok(Node::Repeat(Box::new(atom), min, max, lazy))
    }
    fn escape_class(c: char) -> Option<ClassItem> {
        Some(match c {
            'd' => ClassItem::Digit(true),
            'D' => ClassItem::Digit(false),
            'w' => ClassItem::Word(true),
            'W' => ClassItem::Word(false),
            's' => ClassItem::Space(true),
            'S' => ClassItem::Space(false),
            _ => return None,
        })
    }
    fn escape_char(c: char) -> char {
        match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            other => other,
        }
    }
    fn atom(&mut self, depth: usize) -> Result<Node, String> {
        let c = self.peek().ok_or("unexpected end")?;
        self.pos += 1;
        Ok(match c {
            '.' => Node::Any,
            '^' => Node::Start,
            '$' => Node::End,
            '(' => {
                let mut capture = true;
                if self.chars.get(self.pos) == Some(&'?') && self.chars.get(self.pos + 1) == Some(&':') {
                    self.pos += 2;
                    capture = false;
                }
                let idx = if capture {
                    self.groups += 1;
                    Some(self.groups)
                } else {
                    None
                };
                let inner = self.alt(depth + 1)?;
                if self.peek() != Some(')') {
                    return Err("missing )".into());
                }
                self.pos += 1;
                Node::Group(Box::new(inner), idx)
            }
            '[' => {
                let mut neg = false;
                if self.peek() == Some('^') {
                    neg = true;
                    self.pos += 1;
                }
                let mut items = Vec::new();
                let mut first = true;
                loop {
                    let c = self.peek().ok_or("missing ]")?;
                    self.pos += 1;
                    if c == ']' && !first {
                        break;
                    }
                    first = false;
                    let lo = if c == '\\' {
                        let e = self.peek().ok_or("bad escape")?;
                        self.pos += 1;
                        if let Some(ci) = Self::escape_class(e) {
                            items.push(ci);
                            continue;
                        }
                        Self::escape_char(e)
                    } else {
                        c
                    };
                    if self.peek() == Some('-') && self.chars.get(self.pos + 1).map_or(false, |x| *x != ']') {
                        self.pos += 1;
                        let mut hi = self.peek().ok_or("bad range")?;
                        self.pos += 1;
                        if hi == '\\' {
                            hi = Self::escape_char(self.peek().ok_or("bad escape")?);
                            self.pos += 1;
                        }
                        items.push(ClassItem::Range(lo, hi));
                    } else {
                        items.push(ClassItem::Range(lo, lo));
                    }
                }
                Node::Class(items, neg)
            }
            '\\' => {
                let e = self.peek().ok_or("bad escape")?;
                self.pos += 1;
                if let Some(ci) = Self::escape_class(e) {
                    Node::Class(vec![ci], false)
                } else {
                    Node::Char(Self::escape_char(e))
                }
            }
            '*' | '+' | '?' => return Err("nothing to repeat".into()),
            other => Node::Char(other),
        })
    }
}

fn class_match(items: &[ClassItem], neg: bool, c: char, ic: bool) -> bool {
    let lc = c.to_lowercase().next().unwrap_or(c);
    let uc = c.to_uppercase().next().unwrap_or(c);
    let mut hit = false;
    for it in items {
        let h = match it {
            ClassItem::Range(a, b) => (*a <= c && c <= *b) || (ic && ((*a <= lc && lc <= *b) || (*a <= uc && uc <= *b))),
            ClassItem::Digit(p) => c.is_ascii_digit() == *p,
            ClassItem::Word(p) => (c.is_alphanumeric() || c == '_') == *p,
            ClassItem::Space(p) => c.is_whitespace() == *p,
        };
        if h {
            hit = true;
            break;
        }
    }
    hit != neg
}

struct Matcher<'a> {
    text: &'a [char],
    caps: Vec<Option<(usize, usize)>>,
    steps: usize,
    depth: usize,
    max_depth: usize,
    ic: bool,
}

impl<'a> Matcher<'a> {
    // Match `node` at `i`, then call `k` with the end; true if the continuation succeeds.
    fn m(&mut self, node: &Node, i: usize, k: &mut dyn FnMut(&mut Self, usize) -> bool) -> bool {
        self.steps += 1;
        if self.steps > STEP_BUDGET || self.depth >= self.max_depth {
            return false;
        }
        self.depth += 1;
        let r = self.m_inner(node, i, k);
        self.depth -= 1;
        r
    }

    fn m_inner(&mut self, node: &Node, i: usize, k: &mut dyn FnMut(&mut Self, usize) -> bool) -> bool {
        match node {
            Node::Char(c) => {
                if i < self.text.len() && (self.text[i] == *c || (self.ic && self.text[i].to_lowercase().eq(c.to_lowercase()))) {
                    k(self, i + 1)
                } else {
                    false
                }
            }
            Node::Any => {
                if i < self.text.len() && self.text[i] != '\n' {
                    k(self, i + 1)
                } else {
                    false
                }
            }
            Node::Class(items, neg) => {
                if i < self.text.len() && class_match(items, *neg, self.text[i], self.ic) {
                    k(self, i + 1)
                } else {
                    false
                }
            }
            Node::Start => i == 0 && k(self, i),
            Node::End => i == self.text.len() && k(self, i),
            Node::Group(inner, idx) => {
                let idx = *idx;
                self.m(inner, i, &mut |s: &mut Self, j| {
                    let old = idx.map(|x| s.caps[x]);
                    if let Some(x) = idx {
                        s.caps[x] = Some((i, j));
                    }
                    if k(s, j) {
                        true
                    } else {
                        if let (Some(x), Some(o)) = (idx, old) {
                            s.caps[x] = o;
                        }
                        false
                    }
                })
            }
            Node::Alt(alts) => {
                for a in alts {
                    if self.m(a, i, k) {
                        return true;
                    }
                }
                false
            }
            Node::Seq(items) => self.seq(items, i, k),
            Node::Repeat(inner, min, max, lazy) => self.rep(inner, *min, *max, *lazy, 0, i, k),
        }
    }

    fn seq(&mut self, items: &[Node], i: usize, k: &mut dyn FnMut(&mut Self, usize) -> bool) -> bool {
        match items.split_first() {
            None => k(self, i),
            Some((first, rest)) => self.m(first, i, &mut |s: &mut Self, j| s.seq(rest, j, k)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn rep(&mut self, inner: &Node, min: usize, max: usize, lazy: bool, count: usize, i: usize, k: &mut dyn FnMut(&mut Self, usize) -> bool) -> bool {
        if count < min {
            return self.m(inner, i, &mut |s: &mut Self, j| s.rep(inner, min, max, lazy, count + 1, j, k));
        }
        let try_more = |s: &mut Self, k: &mut dyn FnMut(&mut Self, usize) -> bool| -> bool {
            if count >= max {
                return false;
            }
            s.m(inner, i, &mut |s2: &mut Self, j| j != i && s2.rep(inner, min, max, lazy, count + 1, j, k))
        };
        if lazy {
            k(self, i) || try_more(self, k)
        } else {
            try_more(self, k) || k(self, i)
        }
    }
}

impl Regex {
    pub fn new(pattern: &str, ignore_case: bool) -> Result<Regex, String> {
        if pattern.chars().count() > 500 {
            return Err("pattern too long".into());
        }
        let mut p = Parser { chars: pattern.chars().collect(), pos: 0, groups: 0, _src: pattern };
        let root = p.alt(0)?;
        if p.pos < p.chars.len() {
            return Err("unmatched )".into());
        }
        Ok(Regex { root, groups: p.groups, ignore_case })
    }

    /// The first match at or after `from`: (start, end, captures as char ranges).
    pub fn find_at(&self, text: &[char], from: usize) -> Option<(usize, usize, Vec<Option<(usize, usize)>>)> {
        if text.len() <= SMALL_TEXT {
            return self.find_with(text, from, SMALL_DEPTH);
        }
        std::thread::scope(|sc| {
            match std::thread::Builder::new().stack_size(BIG_STACK).spawn_scoped(sc, || self.find_with(text, from, BIG_DEPTH)) {
                Ok(h) => h.join().unwrap_or(None),
                // No thread to be had: use the small limit here instead.
                Err(_) => self.find_with(text, from, SMALL_DEPTH),
            }
        })
    }

    fn find_with(&self, text: &[char], from: usize, max_depth: usize) -> Option<(usize, usize, Vec<Option<(usize, usize)>>)> {
        // One step budget for the whole search, not one per start position: a long text would
        // otherwise allow millions of steps for every position.
        let mut steps = 0;
        for start in from..=text.len() {
            let mut m = Matcher { text, caps: vec![None; self.groups + 1], steps, depth: 0, max_depth, ic: self.ignore_case };
            let mut end = None;
            let ok = m.m(&self.root, start, &mut |_, j| {
                end = Some(j);
                true
            });
            if ok {
                return Some((start, end.unwrap(), m.caps));
            }
            steps = m.steps;
            if steps > STEP_BUDGET {
                return None;
            }
        }
        None
    }

    pub fn is_match(&self, s: &str) -> bool {
        let t: Vec<char> = s.chars().collect();
        self.find_at(&t, 0).is_some()
    }

    /// Text of the whole match and of each group of the first match.
    pub fn first(&self, s: &str) -> Option<Vec<String>> {
        let t: Vec<char> = s.chars().collect();
        let (a, b, caps) = self.find_at(&t, 0)?;
        let mut out = vec![t[a..b].iter().collect::<String>()];
        for c in caps.iter().skip(1) {
            out.push(c.map(|(x, y)| t[x..y].iter().collect()).unwrap_or_default());
        }
        Some(out)
    }

    pub fn all(&self, s: &str) -> Vec<String> {
        let t: Vec<char> = s.chars().collect();
        let mut out = Vec::new();
        let mut from = 0;
        while from <= t.len() {
            match self.find_at(&t, from) {
                Some((a, b, _)) => {
                    out.push(t[a..b].iter().collect());
                    from = if b == a { b + 1 } else { b };
                }
                None => break,
            }
        }
        out
    }

    pub fn replace(&self, s: &str, with: &str, all: bool) -> String {
        let t: Vec<char> = s.chars().collect();
        let mut out = String::new();
        let mut from = 0;
        let mut last = 0;
        while from <= t.len() {
            match self.find_at(&t, from) {
                Some((a, b, caps)) => {
                    out.extend(&t[last..a]);
                    // $1 .. $9 and $& in the replacement
                    let w: Vec<char> = with.chars().collect();
                    let mut i = 0;
                    while i < w.len() {
                        if w[i] == '$' && i + 1 < w.len() {
                            if w[i + 1] == '&' {
                                out.extend(&t[a..b]);
                                i += 2;
                                continue;
                            }
                            if let Some(d) = w[i + 1].to_digit(10) {
                                if let Some(Some((x, y))) = caps.get(d as usize) {
                                    out.extend(&t[*x..*y]);
                                }
                                i += 2;
                                continue;
                            }
                        }
                        out.push(w[i]);
                        i += 1;
                    }
                    last = b;
                    from = if b == a { b + 1 } else { b };
                    if b == a && a < t.len() {
                        out.push(t[a]);
                        last = a + 1;
                    }
                    if !all {
                        break;
                    }
                }
                None => break,
            }
        }
        if last <= t.len() {
            out.extend(&t[last..]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn re(p: &str) -> Regex {
        Regex::new(p, false).unwrap()
    }

    /// A long text must make a pattern fail to match, never overflow the stack (that would end the
    /// whole app). Run on a thread with a small stack, smaller than a Windows main thread's 1 MB.
    #[test]
    fn long_text_does_not_overflow_the_stack() {
        let h = std::thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                let a: String = "ab".repeat(50_000);
                let _ = re("(?:a|b)*c").is_match(&a);
                let _ = re(".*b").is_match(&"x".repeat(100_000));
                let _ = re("(a|b)+$").is_match(&a);
                let _ = re("^(?:[a-z]+ ?)*!").is_match(&"word ".repeat(20_000));
                // Short texts still work.
                assert!(re("(?:a|b)*c").is_match("ababc"));
                assert!(re(".*b").is_match("xxxb"));
            })
            .unwrap();
        h.join().expect("the matcher must not overflow the stack");
    }

    #[test]
    fn basic_matching() {
        assert!(re("ab+c").is_match("xabbbc"));
        assert!(!re("^ab+c$").is_match("xabbbc"));
        assert!(re("^a.c$").is_match("abc"));
        assert!(re("[a-c]+d").is_match("zzbad"));
        assert!(re("[^0-9]").is_match("a1"));
        assert!(!re("^[^0-9]+$").is_match("a1"));
        assert!(re(r"^\d{3}-\d{2,}$").is_match("123-4567"));
        assert!(re("cat|dog").is_match("hotdog"));
        assert!(re("^(?:ab)*$").is_match("ababab"));
        assert!(re(r"\w+@\w+\.com").is_match("me@site.com"));
        assert!(Regex::new("abc", true).unwrap().is_match("xxABCxx"));
    }

    #[test]
    fn first_all_and_replace() {
        assert_eq!(re(r"(\d+)-(\d+)").first("tel 12-34").unwrap(), vec!["12-34", "12", "34"]);
        assert_eq!(re(r"\d+").all("a1b22c333"), vec!["1", "22", "333"]);
        assert_eq!(re("o").replace("foo boo", "0", false), "f0o boo");
        assert_eq!(re("o").replace("foo boo", "0", true), "f00 b00");
        assert_eq!(re(r"(\w+) (\w+)").replace("hello world", "$2 $1", false), "world hello");
        assert_eq!(re("x*").replace("abc", "-", true), "-a-b-c-");
        assert_eq!(re("a+?").first("aaa").unwrap(), vec!["a"]);
    }

    #[test]
    fn bad_patterns_are_errors_and_slow_ones_stop() {
        assert!(Regex::new("(abc", false).is_err());
        assert!(Regex::new("*a", false).is_err());
        assert!(Regex::new("a)", false).is_err());
        let evil = re("(a+)+$");
        assert!(!evil.is_match(&format!("{}b", "a".repeat(40))));
    }
}
