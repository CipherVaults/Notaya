//! A compact version of the Unicode Bidirectional Algorithm (UAX #9) for the PDF export:
//! paragraph direction from the first strong character (P2, P3), the weak rules W1-W7, paired
//! brackets (N0), neutrals (N1, N2), implicit levels (I1, I2), trailing white space (L1) and the
//! visual reordering of a line (L2). Mirroring of brackets is `mirror` (L4).
//!
//! Left out: the explicit embeddings, overrides and isolates (LRE, RLE, LRO, RLO, PDF, LRI, RLI,
//! FSI, PDI). Those characters are invisible and ignored; LRM and RLM count as strong characters.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bc {
    L,
    R,
    AL,
    EN,
    ES,
    ET,
    AN,
    CS,
    NSM,
    BN,
    B,
    S,
    WS,
    ON,
}

pub fn class(c: char) -> Bc {
    use Bc::*;
    let u = c as u32;
    match u {
        0x09 | 0x0B | 0x1F => S,
        0x0A | 0x0D | 0x1C..=0x1E | 0x85 | 0x2029 => B,
        0x0C | 0x20 | 0x2000..=0x200A | 0x2028 | 0x205F | 0x3000 | 0x1680 => WS,
        0x00..=0x08 | 0x0E..=0x1B | 0x7F..=0x84 | 0x86..=0x9F | 0xAD | 0x200B..=0x200D | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F | 0xFEFF => BN,
        0x30..=0x39 => EN,
        0x2B | 0x2D => ES,
        0x23..=0x25 | 0xA2..=0xA5 | 0xB0 | 0xB1 | 0x2030..=0x2034 | 0x20A0..=0x20CF | 0x609 | 0x60A | 0x66A | 0xFF03..=0xFF05 | 0xFFE0 | 0xFFE1 | 0xFFE5 | 0xFFE6 => ET,
        0x2C | 0x2E | 0x2F | 0x3A | 0xA0 | 0x202F | 0x60C | 0xFF0C | 0xFF0E | 0xFF0F | 0xFF1A => CS,
        0xB2 | 0xB3 | 0xB9 | 0x2070 | 0x2074..=0x2079 | 0x2080..=0x2089 | 0x6F0..=0x6F9 => EN,
        0x00..=0x2F | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E | 0xA1 | 0xA6..=0xA9 | 0xAB | 0xAC | 0xAE | 0xAF | 0xB4 | 0xB6..=0xB8 | 0xBB..=0xBF | 0xD7 | 0xF7 => ON,
        0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05BD | 0x05BF | 0x05C1 | 0x05C2 | 0x05C4 | 0x05C5 | 0x05C7 | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x06DF..=0x06E4 | 0x06E7 | 0x06E8 | 0x06EA..=0x06ED | 0x08D3..=0x08E1 | 0x08E3..=0x08FF | 0xFB1E | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F => NSM,
        0x05BE | 0x05C0 | 0x05C3 | 0x05C6 | 0x05D0..=0x05FF | 0x0590..=0x05CF | 0x07C0..=0x085F | 0xFB1D..=0xFB4F | 0x10800..=0x10FFF | 0x1E800..=0x1EDFF | 0x1EF00..=0x1EFFF | 0x200F => R,
        0x600..=0x605 | 0x660..=0x669 | 0x66B | 0x66C | 0x6DD | 0x8E2 => AN,
        0x606 | 0x607 | 0x60E | 0x60F | 0x6DE | 0x6E9 | 0x2010..=0x2027 | 0x2035..=0x205E | 0x2190..=0x2BFF | 0x3001..=0x3004 | 0x3008..=0x3020 | 0x3030 | 0xFF01..=0xFF02 | 0xFF06..=0xFF0B | 0xFF0D | 0xFF1B..=0xFF20 | 0xFF3B..=0xFF40 | 0xFF5B..=0xFF65 | 0xFDFD | 0x1F000..=0x1FAFF | 0xFE30..=0xFE6F => ON,
        0x0600..=0x07BF | 0x0860..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFE | 0x1EE00..=0x1EEFF => AL,
        0x200E => L,
        _ => L,
    }
}

/// The mirrored form of a bracket, for text that runs right to left.
pub fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        _ => c,
    }
}

/// Paragraph level from the first strong character: 1 for Hebrew and Arabic, otherwise 0.
pub fn base_level(chars: &[char]) -> u8 {
    for &c in chars {
        match class(c) {
            Bc::L => return 0,
            Bc::R | Bc::AL => return 1,
            _ => {}
        }
    }
    0
}

/// True when the text has anything that needs bidirectional handling.
pub fn has_rtl(chars: &[char]) -> bool {
    chars.iter().any(|&c| matches!(class(c), Bc::R | Bc::AL | Bc::AN) || matches!(c as u32, 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069))
}

pub struct Resolved {
    /// 0 or 1.
    pub base: u8,
    /// The embedding level of each character.
    pub levels: Vec<u8>,
}

fn strong(t: Bc) -> Option<Bc> {
    match t {
        Bc::L => Some(Bc::L),
        Bc::R | Bc::EN | Bc::AN => Some(Bc::R),
        _ => None,
    }
}

/// Resolves the levels of one paragraph. `force_base` overrides the paragraph level.
pub fn resolve(chars: &[char], force_base: Option<u8>) -> Resolved {
    let base = force_base.unwrap_or_else(|| base_level(chars));
    let e = if base == 0 { Bc::L } else { Bc::R };
    // Characters the rules ignore (BN) are left out and get their neighbour's level at the end.
    let keep: Vec<usize> = (0..chars.len()).filter(|&i| class(chars[i]) != Bc::BN).collect();
    let orig: Vec<Bc> = keep.iter().map(|&i| class(chars[i])).collect();
    let ch: Vec<char> = keep.iter().map(|&i| chars[i]).collect();
    let n = keep.len();
    let mut t = orig.clone();
    // W1
    for i in 0..n {
        if t[i] == Bc::NSM {
            t[i] = if i == 0 { e } else { t[i - 1] };
        }
    }
    // W2, W3
    let mut last = e;
    for x in t.iter_mut() {
        match *x {
            Bc::L | Bc::R | Bc::AL => last = *x,
            Bc::EN if last == Bc::AL => *x = Bc::AN,
            _ => {}
        }
    }
    for x in t.iter_mut() {
        if *x == Bc::AL {
            *x = Bc::R;
        }
    }
    // W4
    for i in 1..n.saturating_sub(1) {
        let (p, q) = (t[i - 1], t[i + 1]);
        match t[i] {
            Bc::ES if p == Bc::EN && q == Bc::EN => t[i] = Bc::EN,
            Bc::CS if p == Bc::EN && q == Bc::EN => t[i] = Bc::EN,
            Bc::CS if p == Bc::AN && q == Bc::AN => t[i] = Bc::AN,
            _ => {}
        }
    }
    // W5
    let mut i = 0;
    while i < n {
        if t[i] == Bc::ET {
            let start = i;
            while i < n && t[i] == Bc::ET {
                i += 1;
            }
            if (start > 0 && t[start - 1] == Bc::EN) || (i < n && t[i] == Bc::EN) {
                t[start..i].fill(Bc::EN);
            }
        } else {
            i += 1;
        }
    }
    // W6, W7
    for x in t.iter_mut() {
        if matches!(*x, Bc::ES | Bc::ET | Bc::CS) {
            *x = Bc::ON;
        }
    }
    let mut last = e;
    for x in t.iter_mut() {
        match *x {
            Bc::L | Bc::R => last = *x,
            Bc::EN if last == Bc::L => *x = Bc::L,
            _ => {}
        }
    }
    // N0: paired brackets.
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        if t[i] != Bc::ON {
            continue;
        }
        let close = |o: char| match o {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            _ => o,
        };
        if matches!(ch[i], '(' | '[' | '{') {
            if stack.len() < 63 {
                stack.push((close(ch[i]), i));
            }
        } else if matches!(ch[i], ')' | ']' | '}') {
            if let Some(k) = stack.iter().rposition(|(c, _)| *c == ch[i]) {
                pairs.push((stack[k].1, i));
                stack.truncate(k);
            }
        }
    }
    pairs.sort_unstable();
    let opposite = if e == Bc::L { Bc::R } else { Bc::L };
    for (a, b) in pairs {
        let inside: Vec<Bc> = t[a + 1..b].iter().filter_map(|&x| strong(x)).collect();
        let set = if inside.contains(&e) {
            Some(e)
        } else if inside.contains(&opposite) {
            let context = t[..a].iter().rev().find_map(|&x| strong(x)).unwrap_or(e);
            Some(if context == opposite { opposite } else { e })
        } else {
            None
        };
        if let Some(s) = set {
            t[a] = s;
            t[b] = s;
            // Marks that followed a bracket take its direction.
            for k in [a, b] {
                let mut j = k + 1;
                while j < n && orig[j] == Bc::NSM {
                    t[j] = s;
                    j += 1;
                }
            }
        }
    }
    // N1, N2
    let ni = |x: Bc| matches!(x, Bc::B | Bc::S | Bc::WS | Bc::ON);
    let mut i = 0;
    while i < n {
        if ni(t[i]) {
            let start = i;
            while i < n && ni(t[i]) {
                i += 1;
            }
            let before = if start == 0 { e } else { strong(t[start - 1]).unwrap_or(e) };
            let after = if i == n { e } else { strong(t[i]).unwrap_or(e) };
            let dir = if before == after { before } else { e };
            t[start..i].fill(dir);
        } else {
            i += 1;
        }
    }
    // I1, I2
    let mut lv: Vec<u8> = (0..n)
        .map(|i| match (base % 2 == 0, t[i]) {
            (true, Bc::R) => base + 1,
            (true, Bc::AN | Bc::EN) => base + 2,
            (false, Bc::L | Bc::EN | Bc::AN) => base + 1,
            _ => base,
        })
        .collect();
    // L1: separators, and the white space before them and at the end, go to the paragraph level.
    let mut reset = true;
    for i in (0..n).rev() {
        match orig[i] {
            Bc::S | Bc::B => {
                lv[i] = base;
                reset = true;
            }
            Bc::WS if reset => lv[i] = base,
            _ => reset = false,
        }
    }
    let mut levels = vec![base; chars.len()];
    let mut prev = base;
    let mut k = 0;
    for (i, slot) in levels.iter_mut().enumerate() {
        if k < n && keep[k] == i {
            prev = lv[k];
            k += 1;
        }
        *slot = prev;
    }
    Resolved { base, levels }
}

/// The order in which a line's characters are drawn, left to right, from their levels (L2).
pub fn visual_order(levels: &[u8]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..levels.len()).collect();
    let Some(&max) = levels.iter().max() else { return order };
    let min_odd = levels.iter().copied().filter(|l| l % 2 == 1).min();
    let Some(min_odd) = min_odd else { return order };
    let mut lvl = max;
    while lvl >= min_odd {
        let mut i = 0;
        while i < order.len() {
            if levels[order[i]] >= lvl {
                let start = i;
                while i < order.len() && levels[order[i]] >= lvl {
                    i += 1;
                }
                order[start..i].reverse();
            } else {
                i += 1;
            }
        }
        if lvl == 0 {
            break;
        }
        lvl -= 1;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels(s: &str) -> Vec<u8> {
        let c: Vec<char> = s.chars().collect();
        resolve(&c, None).levels
    }

    /// The text of a line in drawing order (mirrored brackets applied).
    fn visual(s: &str) -> String {
        let c: Vec<char> = s.chars().collect();
        let r = resolve(&c, None);
        visual_order(&r.levels).into_iter().map(|i| if r.levels[i] % 2 == 1 { mirror(c[i]) } else { c[i] }).collect()
    }

    #[test]
    fn classes() {
        assert_eq!(class('a'), Bc::L);
        assert_eq!(class('ب'), Bc::AL);
        assert_eq!(class('ש'), Bc::R);
        assert_eq!(class('7'), Bc::EN);
        assert_eq!(class('٧'), Bc::AN);
        assert_eq!(class('۷'), Bc::EN);
        assert_eq!(class('%'), Bc::ET);
        assert_eq!(class(','), Bc::CS);
        assert_eq!(class('+'), Bc::ES);
        assert_eq!(class('\u{064E}'), Bc::NSM);
        assert_eq!(class(' '), Bc::WS);
        assert_eq!(class('('), Bc::ON);
        assert_eq!(class('中'), Bc::L);
        assert_eq!(class('\u{FEFB}'), Bc::AL);
    }

    #[test]
    fn plain_left_to_right_text_has_level_zero_and_keeps_its_order() {
        assert_eq!(levels("Hello, world 42!"), vec![0; 16]);
        assert_eq!(base_level(&"Hello".chars().collect::<Vec<_>>()), 0);
        assert_eq!(visual("Hello 12"), "Hello 12");
        assert!(!has_rtl(&"Hello 12".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn hebrew_and_arabic_run_right_to_left() {
        assert_eq!(levels("שלום"), vec![1; 4]);
        assert_eq!(visual("שלום"), "םולש");
        assert_eq!(visual("مرحبا"), "ابحرم");
        // The paragraph level comes from the first strong character, even after neutrals.
        assert_eq!(base_level(&"  123 שלום".chars().collect::<Vec<_>>()), 1);
        assert_eq!(base_level(&"- abc שלום".chars().collect::<Vec<_>>()), 0);
    }

    #[test]
    fn english_inside_arabic_and_arabic_inside_english() {
        // Arabic paragraph, English word in the middle: the word stays left to right.
        assert_eq!(levels("بب cd بب"), vec![1, 1, 1, 2, 2, 1, 1, 1]);
        assert_eq!(visual("بب cd بب"), "بب cd بب");
        assert_eq!(visual("اب cd هو"), "وه cd با");
        // English paragraph with an Arabic word: the word is reversed, the order of words is not.
        assert_eq!(levels("ab بج ef"), vec![0, 0, 0, 1, 1, 0, 0, 0]);
        assert_eq!(visual("ab بج ef"), "ab جب ef");
        // Two Arabic words after English: the space between them belongs to the RTL run.
        assert_eq!(visual("ab بج دو ef"), "ab ود جب ef");
    }

    #[test]
    fn numbers_keep_their_order_and_sit_at_level_two() {
        // After Arabic letters a European digit becomes an Arabic number (W2).
        assert_eq!(levels("سعر 123"), vec![1, 1, 1, 1, 2, 2, 2]);
        assert_eq!(visual("سعر 123"), "123 رعس");
        // Hebrew keeps European numbers as they are (level 2).
        assert_eq!(visual("דוח 2024 סוף"), "ףוס 2024 חוד");
        // Arabic-Indic digits.
        assert_eq!(visual("عام ٢٠٢٤"), "٢٠٢٤ ماع");
        // A separator between two numbers belongs to the number: 1,234 and 3.14.
        assert_eq!(visual("ب 1,234"), "1,234 ب");
        assert_eq!(visual("ب 3.14"), "3.14 ب");
        // The percent sign after an Arabic number is a neutral and goes to the left of it (W5 only joins European numbers).
        assert_eq!(visual("ب 50%"), "%50 ب");
        // After a European number in a Hebrew paragraph it stays with the number.
        assert_eq!(visual("ש 50%"), "50% ש");
        // A number at the start of an English line stays plain.
        assert_eq!(levels("12 ab"), vec![0; 5]);
    }

    #[test]
    fn brackets_are_paired_and_mirrored() {
        // Hebrew paragraph, English in brackets: the brackets go with the Hebrew.
        assert_eq!(visual("שלום (abc) עולם"), "םלוע (abc) םולש");
        // Brackets around Hebrew in an English line.
        assert_eq!(visual("go (שלום) ok"), "go (םולש) ok");
        // Brackets around English at the end of an English line in front of Arabic: the context decides.
        assert_eq!(visual("abc (ب)"), "abc (ب)");
        // A closing bracket without an opening one is just a neutral.
        assert_eq!(visual("ب) ب"), "ب (ب");
    }

    #[test]
    fn marks_follow_their_letter_and_trailing_space_returns_to_the_paragraph_level() {
        assert_eq!(levels("بَ"), vec![1, 1]);
        let c: Vec<char> = "سلام ".chars().collect();
        let r = resolve(&c, None);
        assert_eq!(*r.levels.last().unwrap(), 1);
        let c: Vec<char> = "abc ".chars().collect();
        assert_eq!(resolve(&c, None).levels, vec![0; 4]);
        // A forced base level wins.
        let c: Vec<char> = "بب".chars().collect();
        assert_eq!(resolve(&c, Some(0)).levels, vec![1, 1]);
        assert_eq!(resolve(&"ab".chars().collect::<Vec<_>>(), Some(1)).levels, vec![2, 2]);
    }

    #[test]
    fn invisible_controls_take_the_level_of_their_neighbour() {
        let c: Vec<char> = "ب\u{200C}ب".chars().collect();
        assert_eq!(resolve(&c, None).levels, vec![1, 1, 1]);
    }

    #[test]
    fn reordering_runs_by_level() {
        assert_eq!(visual_order(&[0, 0, 0]), vec![0, 1, 2]);
        assert_eq!(visual_order(&[1, 1, 1]), vec![2, 1, 0]);
        assert_eq!(visual_order(&[0, 1, 1, 0]), vec![0, 2, 1, 3]);
        assert_eq!(visual_order(&[1, 1, 2, 2, 1]), vec![4, 2, 3, 1, 0]);
        assert_eq!(visual_order(&[]), Vec::<usize>::new());
    }
}
