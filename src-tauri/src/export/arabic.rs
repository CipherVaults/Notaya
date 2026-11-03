//! Arabic shaping for the PDF export: each letter is replaced by its isolated, final, initial or
//! medial form from Unicode's Arabic Presentation Forms (blocks FB50 and FE70), and lam followed
//! by alef becomes one ligature. The marks (tashkeel) are kept as they are. This is the part of
//! OpenType shaping that matters for ordinary text (Arabic, Persian, Urdu); other features
//! (optional ligatures, kashida, stylistic sets) are not done.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Join {
    /// Does not join (digits, hamza, punctuation, other scripts).
    None,
    /// Joins to the letter before it only (alef, dal, reh, waw ...).
    Right,
    /// Joins on both sides (beh, seen, lam ...).
    Dual,
    /// Tatweel and ZWJ: make their neighbours join.
    Causing,
    /// Marks: do not interrupt joining.
    Transparent,
}

pub fn joining(c: char) -> Join {
    match c as u32 {
        0x200D | 0x0640 => Join::Causing,
        0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x06DF..=0x06E4 | 0x06E7 | 0x06E8 | 0x06EA..=0x06ED | 0x200B | 0x200E | 0x200F | 0x0300..=0x036F | 0x0591..=0x05BD | 0x05BF | 0x05C1 | 0x05C2 | 0x05C4 | 0x05C5 | 0x05C7 => Join::Transparent,
        0x0622..=0x0625 | 0x0627 | 0x0629 | 0x062F..=0x0632 | 0x0648 | 0x0671..=0x0673 | 0x0675..=0x0677 | 0x0688..=0x0699 | 0x06C0 | 0x06C3..=0x06CB | 0x06CD | 0x06CF | 0x06D2 | 0x06D3 | 0x06D5 | 0x06EE | 0x06EF => Join::Right,
        0x0626 | 0x0628 | 0x062A..=0x063F | 0x0641..=0x0647 | 0x0649 | 0x064A | 0x066E | 0x066F | 0x0678..=0x0687 | 0x069A..=0x06BF | 0x06C1 | 0x06C2 | 0x06CC | 0x06CE | 0x06D0 | 0x06D1 | 0x06FA..=0x06FC | 0x06FF => Join::Dual,
        _ => Join::None,
    }
}

/// True for the marks that sit on or under a letter and take no room of their own.
pub fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x0591..=0x05BD | 0x05BF | 0x05C1 | 0x05C2 | 0x05C4 | 0x05C5 | 0x05C7 | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x06DF..=0x06E4 | 0x06E7 | 0x06E8 | 0x06EA..=0x06ED)
}

/// Marks below the letter (the rest are above): kasra, kasratan, and the Hebrew points under letters.
pub fn is_below(c: char) -> bool {
    matches!(c as u32, 0x061A | 0x064D | 0x0650 | 0x0655 | 0x0656 | 0x065C | 0x05B0..=0x05BB | 0x05BD | 0x05C5 | 0x05C7)
}

/// (isolated, final, initial, medial); 0 where the letter has no such form.
type Forms = [u32; 4];

fn table() -> &'static HashMap<u32, Forms> {
    static T: OnceLock<HashMap<u32, Forms>> = OnceLock::new();
    T.get_or_init(|| {
        let mut t: HashMap<u32, Forms> = HashMap::new();
        // Letters with four forms: (letter, isolated form); final, initial and medial follow it.
        let dual = [
            (0x0626, 0xFE89), (0x0628, 0xFE8F), (0x062A, 0xFE95), (0x062B, 0xFE99), (0x062C, 0xFE9D), (0x062D, 0xFEA1), (0x062E, 0xFEA5),
            (0x0633, 0xFEB1), (0x0634, 0xFEB5), (0x0635, 0xFEB9), (0x0636, 0xFEBD), (0x0637, 0xFEC1), (0x0638, 0xFEC5), (0x0639, 0xFEC9),
            (0x063A, 0xFECD), (0x0641, 0xFED1), (0x0642, 0xFED5), (0x0643, 0xFED9), (0x0644, 0xFEDD), (0x0645, 0xFEE1), (0x0646, 0xFEE5),
            (0x0647, 0xFEE9), (0x064A, 0xFEF1),
            // Persian, Urdu and others (Presentation Forms-A)
            (0x067E, 0xFB56), (0x0686, 0xFB7A), (0x06A9, 0xFB8E), (0x06AF, 0xFB92), (0x06CC, 0xFBFC), (0x0679, 0xFB66), (0x06BE, 0xFBAA), (0x06C1, 0xFBA6),
        ];
        for (c, f) in dual {
            t.insert(c, [f, f + 1, f + 2, f + 3]);
        }
        // Letters that join only to the letter before: isolated and final form.
        for (c, f) in [
            (0x0622u32, 0xFE81u32), (0x0623, 0xFE83), (0x0624, 0xFE85), (0x0625, 0xFE87), (0x0627, 0xFE8D), (0x0629, 0xFE93), (0x062F, 0xFEA9),
            (0x0630, 0xFEAB), (0x0631, 0xFEAD), (0x0632, 0xFEAF), (0x0648, 0xFEED), (0x0649, 0xFEEF),
            (0x0671, 0xFB50), (0x0688, 0xFB88), (0x0691, 0xFB8C), (0x0698, 0xFB8A), (0x06BA, 0xFB9E), (0x06C0, 0xFBA4), (0x06D2, 0xFBAE),
        ] {
            t.insert(c, [f, f + 1, 0, 0]);
        }
        t
    })
}

/// The lam-alef ligature (isolated, final) for the letter after a lam.
fn lam_alef(next: char) -> Option<(u32, u32)> {
    Some(match next as u32 {
        0x0622 => (0xFEF5, 0xFEF6),
        0x0623 => (0xFEF7, 0xFEF8),
        0x0625 => (0xFEF9, 0xFEFA),
        0x0627 => (0xFEFB, 0xFEFC),
        _ => return None,
    })
}

/// One output character: the shaped form and the source characters (`from..to`) it stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shaped {
    pub ch: char,
    pub from: usize,
    pub to: usize,
}

/// Shapes a run of text in logical order. Other characters pass through. ZWJ and ZWNJ are used
/// for joining and then left out.
pub fn shape(chars: &[char]) -> Vec<Shaped> {
    let n = chars.len();
    let kind: Vec<Join> = chars.iter().map(|c| joining(*c)).collect();
    // The nearest letter before and after that is not a mark.
    let prev_of = |i: usize| (0..i).rev().find(|&j| kind[j] != Join::Transparent);
    let next_of = |i: usize| (i + 1..n).find(|&j| kind[j] != Join::Transparent);
    let joins_forward = |j: usize| matches!(kind[j], Join::Dual | Join::Causing);
    let joins_backward = |j: usize| matches!(kind[j], Join::Dual | Join::Right | Join::Causing);
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if matches!(c, '\u{200C}' | '\u{200D}') {
            i += 1;
            continue;
        }
        let from_prev = prev_of(i).is_some_and(joins_forward);
        let to_next = next_of(i).is_some_and(joins_backward);
        // Lam + alef.
        if c == '\u{0644}' {
            if let Some(j) = next_of(i).filter(|&j| j == i + 1) {
                if let Some((iso, fin)) = lam_alef(chars[j]) {
                    let ch = char::from_u32(if from_prev { fin } else { iso }).unwrap_or(c);
                    out.push(Shaped { ch, from: i, to: j + 1 });
                    i = j + 1;
                    continue;
                }
            }
        }
        let mut ch = c;
        if matches!(kind[i], Join::Dual | Join::Right) {
            if let Some(f) = table().get(&(c as u32)) {
                let form = match (kind[i], from_prev, to_next) {
                    (Join::Dual, true, true) => f[3],
                    (Join::Dual, true, false) | (Join::Right, true, _) => f[1],
                    (Join::Dual, false, true) => f[2],
                    _ => f[0],
                };
                if let Some(s) = char::from_u32(form).filter(|_| form != 0) {
                    ch = s;
                } else if let Some(s) = char::from_u32(f[0]).filter(|_| f[0] != 0) {
                    ch = s;
                }
            }
        }
        out.push(Shaped { ch, from: i, to: i + 1 });
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shaped(s: &str) -> String {
        shape(&s.chars().collect::<Vec<_>>()).iter().map(|x| x.ch).collect()
    }

    #[test]
    fn joining_types() {
        assert_eq!(joining('ب'), Join::Dual);
        assert_eq!(joining('ا'), Join::Right);
        assert_eq!(joining('ء'), Join::None);
        assert_eq!(joining('\u{064E}'), Join::Transparent);
        assert_eq!(joining('ـ'), Join::Causing);
        assert_eq!(joining('1'), Join::None);
        assert_eq!(joining('a'), Join::None);
        assert_eq!(joining('پ'), Join::Dual);
    }

    #[test]
    fn letters_take_the_form_their_neighbours_ask_for() {
        // A single letter is isolated.
        assert_eq!(shaped("ب"), "\u{FE8F}");
        // bism: beh initial, seen medial, meem final.
        assert_eq!(shaped("بسم"), "\u{FE91}\u{FEB4}\u{FEE2}");
        // "kitab": kaf initial, teh medial, alef final, beh... (كتاب)
        assert_eq!(shaped("كتاب"), "\u{FEDB}\u{FE98}\u{FE8E}\u{FE8F}");
        // Alef does not join forward: the letter after it starts again (initial/isolated).
        assert_eq!(shaped("اب"), "\u{FE8D}\u{FE8F}");
        // A right-joining letter at the end of a word is final.
        assert_eq!(shaped("بد"), "\u{FE91}\u{FEAA}");
        // A word ends at a space.
        assert_eq!(shaped("با با"), "\u{FE91}\u{FE8E} \u{FE91}\u{FE8E}");
        // Hamza does not join in either direction.
        assert_eq!(shaped("بءب"), "\u{FE8F}ء\u{FE8F}");
        // Latin letters and digits pass through and break joining.
        assert_eq!(shaped("ب1ب"), "\u{FE8F}1\u{FE8F}");
        // Persian letters.
        assert_eq!(shaped("پ"), "\u{FB56}");
        assert_eq!(shaped("پس"), "\u{FB58}\u{FEB2}");
        assert_eq!(shaped("می"), "\u{FEE3}\u{FBFD}");
    }

    #[test]
    fn lam_alef_makes_one_ligature() {
        let s = shape(&"لا".chars().collect::<Vec<_>>());
        assert_eq!(s, vec![Shaped { ch: '\u{FEFB}', from: 0, to: 2 }]);
        // After a joining letter the final form is used: "bla" (بلا).
        assert_eq!(shaped("بلا"), "\u{FE91}\u{FEFC}");
        assert_eq!(shaped("لأ"), "\u{FEF7}");
        assert_eq!(shaped("لإ"), "\u{FEF9}");
        assert_eq!(shaped("لآ"), "\u{FEF5}");
        // Lam before another letter is an ordinary lam (initial).
        assert_eq!(shaped("لب"), "\u{FEDF}\u{FE90}");
    }

    #[test]
    fn marks_do_not_break_joining_and_stay_as_they_are() {
        // beh + fatha + seen: the fatha is skipped when looking for neighbours.
        assert_eq!(shaped("بَس"), "\u{FE91}\u{064E}\u{FEB2}");
        // shadda between two letters.
        assert_eq!(shaped("مّد"), "\u{FEE3}\u{0651}\u{FEAA}");
        // A mark between lam and alef does not make a ligature (kept simple).
        assert_eq!(shaped("لَا"), "\u{FEDF}\u{064E}\u{FE8E}");
    }

    #[test]
    fn tatweel_and_zero_width_joiners() {
        assert_eq!(shaped("بـب"), "\u{FE91}\u{0640}\u{FE90}");
        // ZWNJ stops joining and is left out; ZWJ forces it.
        assert_eq!(shaped("ب\u{200C}ب"), "\u{FE8F}\u{FE8F}");
        assert_eq!(shaped("ب\u{200D}"), "\u{FE91}");
    }

    #[test]
    fn marks_below_and_above() {
        assert!(is_mark('\u{064E}') && is_mark('\u{05B0}') && !is_mark('ب'));
        assert!(is_below('\u{0650}') && is_below('\u{05B4}'));
        assert!(!is_below('\u{064E}') && !is_below('\u{0651}') && !is_below('\u{05C1}'));
    }
}
