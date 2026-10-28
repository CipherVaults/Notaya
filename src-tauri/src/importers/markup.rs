//! A small, forgiving tag tokenizer for HTML and for the XML inside Word files.
//!
//! It never fails: anything it does not understand becomes text. It is linear in the input
//! and keeps no state except the position, so hostile input can only make it produce junk.
//! Comments, doctypes and processing instructions are dropped. In HTML mode the content of
//! `script`, `style` and similar elements is skipped without being looked at.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Start { name: String, attrs: Vec<(String, String)>, self_closing: bool },
    End { name: String },
    Text(String),
}

pub struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
    html: bool,
    /// After a raw-text start tag (HTML): skip to its end tag. `true` = keep the text.
    raw: Option<(String, bool)>,
}

/// Elements whose content is never markup in HTML. The bool says whether the text is kept.
fn raw_kind(name: &str) -> Option<bool> {
    match name {
        "script" | "style" | "xmp" | "iframe" | "noembed" | "noframes" | "noscript" | "template" | "plaintext" => Some(false),
        "title" | "textarea" => Some(true),
        _ => None,
    }
}

impl<'a> Tokenizer<'a> {
    pub fn html(src: &'a str) -> Self {
        Tokenizer { src, pos: 0, html: true, raw: None }
    }

    pub fn xml(src: &'a str) -> Self {
        Tokenizer { src, pos: 0, html: false, raw: None }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    /// Skips to just after `pat` (or to the end).
    fn skip_past(&mut self, pat: &str) {
        match self.rest().find(pat) {
            Some(i) => self.pos += i + pat.len(),
            None => self.pos = self.src.len(),
        }
    }

    fn raw_text(&mut self, name: &str, keep: bool) -> Option<Token> {
        let rest = self.rest();
        let lower = rest.to_ascii_lowercase();
        let close = format!("</{name}");
        let end = lower.find(&close).unwrap_or(rest.len());
        let body = &rest[..end];
        self.pos += end;
        if keep && !body.is_empty() {
            Some(Token::Text(decode_entities(body)))
        } else {
            None
        }
    }

    fn tag(&mut self) -> Option<Token> {
        let rest = self.rest();
        let bytes = rest.as_bytes();
        // rest starts with '<'
        if rest.starts_with("<!--") {
            self.pos += 4;
            self.skip_past("-->");
            return None;
        }
        if rest.starts_with("<![CDATA[") {
            let start = self.pos + 9;
            let end = self.src[start..].find("]]>").map_or(self.src.len(), |i| start + i);
            let text = self.src[start..end].to_string();
            self.pos = (end + 3).min(self.src.len());
            return Some(Token::Text(text));
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            self.skip_past(">");
            return None;
        }
        let closing = bytes.get(1) == Some(&b'/');
        let name_start = if closing { 2 } else { 1 };
        if !bytes.get(name_start).is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_') {
            // Not a tag: a lone '<' is text.
            self.pos += 1;
            return Some(Token::Text("<".into()));
        }
        let mut i = name_start;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' && bytes[i] != b'/' {
            i += 1;
        }
        let mut name = rest[name_start..i].to_string();
        if self.html {
            name.make_ascii_lowercase();
        }
        // Attributes.
        let mut attrs = Vec::new();
        let mut self_closing = false;
        loop {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'>' => {
                    i += 1;
                    break;
                }
                b'/' => {
                    self_closing = true;
                    i += 1;
                    continue;
                }
                _ => {}
            }
            self_closing = false;
            let a_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && !matches!(bytes[i], b'=' | b'>' | b'/') {
                i += 1;
            }
            if i == a_start {
                // A stray character such as a quote: skip it.
                i += 1;
                continue;
            }
            let mut a_name = rest[a_start..i].to_string();
            if self.html {
                a_name.make_ascii_lowercase();
            }
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let mut value = String::new();
            if i < bytes.len() && bytes[i] == b'=' {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    let q = bytes[i];
                    let v_start = i + 1;
                    let v_end = rest[v_start..].bytes().position(|b| b == q).map_or(rest.len(), |p| v_start + p);
                    value = decode_entities(&rest[v_start..v_end]);
                    i = (v_end + 1).min(rest.len());
                } else {
                    let v_start = i;
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                        i += 1;
                    }
                    value = decode_entities(&rest[v_start..i]);
                }
            }
            if attrs.len() < 64 {
                attrs.push((a_name, value));
            }
        }
        self.pos += i.min(rest.len());
        if closing {
            return Some(Token::End { name });
        }
        if self.html && !self_closing {
            if let Some(keep) = raw_kind(&name) {
                self.raw = Some((name.clone(), keep));
            }
        }
        Some(Token::Start { name, attrs, self_closing })
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        loop {
            if self.pos >= self.src.len() {
                return None;
            }
            if let Some((name, keep)) = self.raw.take() {
                if let Some(t) = self.raw_text(&name, keep) {
                    return Some(t);
                }
                continue;
            }
            let rest = self.rest();
            if rest.starts_with('<') {
                if let Some(t) = self.tag() {
                    return Some(t);
                }
                continue;
            }
            let end = rest.find('<').unwrap_or(rest.len());
            self.pos += end;
            return Some(Token::Text(decode_entities(&rest[..end])));
        }
    }
}

/// Looks up an attribute by name. In XML, `w:val` also matches a lookup for `val` when
/// no exact name exists.
pub fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .or_else(|| attrs.iter().find(|(k, _)| local(k) == name))
        .map(|(_, v)| v.as_str())
}

/// The part of a qualified name after the prefix (`w:p` gives `p`).
pub fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// Replaces character references (`&amp;`, `&#233;`, `&#x1F600;` and common names).
/// Unknown names are left as they are.
pub fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        match decode_one(rest) {
            Some((c, used)) => {
                out.push_str(&c);
                rest = &rest[used..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Decodes one reference at the start of `s` (which starts with '&'): the text and the bytes used.
pub fn decode_one(s: &str) -> Option<(String, usize)> {
    // Byte search, so a cut never falls inside a character.
    let semi = s.as_bytes().iter().take(40).position(|&b| b == b';')?;
    let body = &s[1..semi];
    let text = if let Some(num) = body.strip_prefix('#') {
        let code = if let Some(hex) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            num.parse::<u32>().ok()?
        };
        let c = char::from_u32(code).filter(|c| *c != '\0').unwrap_or('\u{FFFD}');
        c.to_string()
    } else {
        named(body)?.to_string()
    };
    Some((text, semi + 1))
}

fn named(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{a0}",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "hellip" => "…",
        "mdash" => "—",
        "ndash" => "–",
        "lsquo" => "‘",
        "rsquo" => "’",
        "sbquo" => "‚",
        "ldquo" => "“",
        "rdquo" => "”",
        "bdquo" => "„",
        "bull" => "•",
        "middot" => "·",
        "laquo" => "«",
        "raquo" => "»",
        "deg" => "°",
        "times" => "×",
        "divide" => "÷",
        "plusmn" => "±",
        "euro" => "€",
        "pound" => "£",
        "yen" => "¥",
        "cent" => "¢",
        "sect" => "§",
        "para" => "¶",
        "shy" => "\u{ad}",
        "zwj" => "\u{200d}",
        "zwnj" => "\u{200c}",
        "ensp" => "\u{2002}",
        "emsp" => "\u{2003}",
        "thinsp" => "\u{2009}",
        "larr" => "←",
        "rarr" => "→",
        "uarr" => "↑",
        "darr" => "↓",
        "harr" => "↔",
        "check" => "✓",
        "hearts" => "♥",
        "star" => "☆",
        "iexcl" => "¡",
        "iquest" => "¿",
        "frac12" => "½",
        "frac14" => "¼",
        "frac34" => "¾",
        "sup2" => "²",
        "sup3" => "³",
        "micro" => "µ",
        "aacute" => "á",
        "eacute" => "é",
        "iacute" => "í",
        "oacute" => "ó",
        "uacute" => "ú",
        "agrave" => "à",
        "egrave" => "è",
        "auml" => "ä",
        "ouml" => "ö",
        "uuml" => "ü",
        "Auml" => "Ä",
        "Ouml" => "Ö",
        "Uuml" => "Ü",
        "szlig" => "ß",
        "ntilde" => "ñ",
        "ccedil" => "ç",
        "Eacute" => "É",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_and_skips_scripts() {
        let toks: Vec<Token> = Tokenizer::html("<P class=x>a &amp; b<br/><script>alert('<p>')</script><!-- c --><img src='i.png' onerror=\"x()\">a < b").collect();
        assert_eq!(toks[0], Token::Start { name: "p".into(), attrs: vec![("class".into(), "x".into())], self_closing: false });
        assert_eq!(toks[1], Token::Text("a & b".into()));
        assert!(matches!(&toks[2], Token::Start { name, self_closing: true, .. } if name == "br"));
        assert!(matches!(&toks[3], Token::Start { name, .. } if name == "script"));
        assert_eq!(toks[4], Token::End { name: "script".into() }, "the script body is skipped");
        assert!(matches!(&toks[5], Token::Start { name, .. } if name == "img"));
        let rest: String = toks[6..].iter().map(|t| if let Token::Text(s) = t { s.clone() } else { String::new() }).collect();
        assert_eq!(rest, "a < b");
    }

    #[test]
    fn hostile_input_never_panics() {
        for s in ["<", "<a", "<a b='", "&#xFFFFFFFF;", "&#0;", "<![CDATA[", "</", "<a =x>", "&#55296;", "<<<<>>>>", "<a\u{e9}>"] {
            let _ = Tokenizer::html(s).count();
            let _ = Tokenizer::xml(s).count();
        }
        assert_eq!(decode_entities("&#55296;&#0;&bogus;"), "\u{FFFD}\u{FFFD}&bogus;");
    }

    #[test]
    fn xml_keeps_case_and_prefixes() {
        let toks: Vec<Token> = Tokenizer::xml(r#"<?xml version="1.0"?><w:p><w:r w:rsidR="1"><w:t xml:space="preserve"> Hi </w:t></w:r></w:p>"#).collect();
        assert!(matches!(&toks[0], Token::Start { name, .. } if name == "w:p"));
        assert!(toks.contains(&Token::Text(" Hi ".into())));
        let attrs = vec![("w:val".to_string(), "Heading1".to_string())];
        assert_eq!(attr(&attrs, "val"), Some("Heading1"));
    }
}
