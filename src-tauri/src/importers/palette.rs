//! Notaya's own text and highlight colors (src/lib/palette.ts), and how a color read from a file
//! (an inline `style`, a Word run's shading) becomes one of them.
//!
//! Notaya's exports write a highlight as its translucent value blended onto white (`#EEE5C4` for
//! Butter), because paper, Word and most viewers have no translucency. Several palette entries
//! blend to values only a few steps apart (Cocoa and Clay are two apart), so a nearest-color search
//! alone sent some of them to a neighbour (Butter came back as Honey). The exact blended value of
//! every entry is therefore matched first; only a color that is none of them is matched to the
//! nearest entry. What comes back is always the exact value the editor's menus write, never the
//! file's own text, so nothing but these fixed strings reaches the editor.

/// The editor's text colors (TEXT_COLORS in palette.ts, without "Default"), exactly as written.
pub const TEXT_COLORS: [(&str, &str); 18] = [
    ("Ocean", "#4a8bc2"),
    ("Brick", "#d04f4f"),
    ("Honey", "#cb912f"),
    ("Sage", "#43905f"),
    ("Orchid", "#c14c8a"),
    ("Slate", "#78838f"),
    ("Amber", "#d9730d"),
    ("Lavender", "#8d6bc2"),
    ("Cocoa", "#9f6b53"),
    ("Mustard", "#a67c1c"),
    ("Denim", "#337ea9"),
    ("Rose", "#c9557f"),
    ("Moss", "#448361"),
    ("Clay", "#a8765a"),
    ("Plum", "#9065b0"),
    ("Terracotta", "#c8683a"),
    ("Cherry", "#d44c47"),
    ("Ash", "#787774"),
];

/// The editor's highlight colors (HIGHLIGHTS in palette.ts), exactly as written.
pub const HIGHLIGHTS: [(&str, &str); 18] = [
    ("Mist", "rgba(100, 150, 190, 0.32)"),
    ("Brick", "rgba(200, 105, 100, 0.32)"),
    ("Honey", "rgba(203, 145, 47, 0.30)"),
    ("Sage", "rgba(110, 160, 125, 0.32)"),
    ("Orchid", "rgba(193, 76, 138, 0.28)"),
    ("Stone", "rgba(140, 142, 148, 0.30)"),
    ("Amber", "rgba(217, 115, 13, 0.28)"),
    ("Lavender", "rgba(150, 125, 190, 0.32)"),
    ("Cocoa", "rgba(159, 107, 83, 0.28)"),
    ("Butter", "rgba(206, 178, 80, 0.34)"),
    ("Denim", "rgba(51, 126, 169, 0.28)"),
    ("Blush", "rgba(200, 120, 150, 0.32)"),
    ("Moss", "rgba(68, 131, 97, 0.28)"),
    ("Clay", "rgba(176, 130, 105, 0.32)"),
    ("Plum", "rgba(144, 101, 176, 0.28)"),
    ("Peach", "rgba(214, 140, 90, 0.32)"),
    ("Cherry", "rgba(212, 76, 71, 0.28)"),
    ("Ash", "rgba(120, 119, 116, 0.28)"),
];

/// What Notaya's exports write for a highlight that has no color (the editor's default yellow,
/// `DEFAULT_HIGHLIGHT` in export/doc.rs). It comes back as the default highlight.
const DEFAULT_HIGHLIGHT: [u8; 3] = [0xFE, 0xF0, 0x8A];

type Rgb = [u8; 3];

/// A CSS or Word color: `#rgb`, `#rrggbb`, `#rrggbbaa` (alpha ignored), `rrggbb` (Word),
/// `rgb(r, g, b)` and `rgba(r, g, b, a)` (blended onto white, the same arithmetic as the export's
/// `Rgb::parse`, so a palette value gives exactly the blended value the export wrote). Anything
/// else (names, `transparent`, `var(...)`, `url(...)`, long text) is `None`.
pub fn parse(s: &str) -> Option<Rgb> {
    let s = s.trim();
    if s.len() > 64 {
        return None;
    }
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.chars().all(|c| c.is_ascii_hexdigit()) {
        let h = |i: usize, n: usize| u8::from_str_radix(hex.get(i..i + n)?, 16).ok();
        return match (hex.len(), s.starts_with('#')) {
            (3, true) => Some([h(0, 1)? * 17, h(1, 1)? * 17, h(2, 1)? * 17]),
            (6, _) | (8, true) => Some([h(0, 2)?, h(2, 2)?, h(4, 2)?]),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    let inner = lower.strip_prefix("rgba(").or_else(|| lower.strip_prefix("rgb("))?.strip_suffix(')')?;
    let parts: Vec<f32> = inner.split(',').map(|p| p.trim().parse::<f32>()).collect::<Result<_, _>>().ok()?;
    if parts.len() < 3 || parts.len() > 4 || parts.iter().any(|p| !p.is_finite()) {
        return None;
    }
    let a = parts.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0);
    if a == 0.0 {
        return None; // fully transparent: no color at all
    }
    let blend = |c: f32| (255.0 - a * (255.0 - c.clamp(0.0, 255.0))).round() as u8;
    Some([blend(parts[0]), blend(parts[1]), blend(parts[2])])
}

/// The un-blended color of a palette value (`rgba(r, g, b, a)` without the alpha).
fn base(value: &str) -> Rgb {
    let inner = value.trim_start_matches("rgba(").trim_end_matches(')');
    let mut it = inner.split(',').map(|p| p.trim().parse::<f32>().unwrap_or(0.0) as u8);
    [it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0)]
}

fn dist(a: Rgb, b: Rgb) -> u32 {
    a.iter().zip(b).map(|(x, y)| (*x as i32 - y as i32).pow(2) as u32).sum()
}

fn spread(c: Rgb) -> (u8, u8) {
    (*c.iter().max().unwrap_or(&0), *c.iter().min().unwrap_or(&0))
}

/// A highlight (background) color as the editor's value: `Some("")` is the default highlight,
/// `Some(value)` one of [`HIGHLIGHTS`], `None` "this is no highlight" (white, near-white and
/// light grays such as Word's shading or the gray of inline code, dark fills).
pub fn highlight(c: Rgb) -> Option<String> {
    if let Some((_, v)) = HIGHLIGHTS.iter().find(|(_, v)| parse(v) == Some(c)) {
        return Some((*v).to_string());
    }
    if c == DEFAULT_HIGHLIGHT {
        return Some(String::new());
    }
    let (max, min) = spread(c);
    let gray = max - min < 24;
    // Grays: only the mid light grays that Notaya's gray highlights (Stone, Ash) come out as.
    if max < 100 || (gray && !(200..=236).contains(&min)) {
        return None;
    }
    // The nearest entry, by its blended value (a pale fill) or its own color (a strong one).
    HIGHLIGHTS
        .iter()
        .filter(|(_, v)| !gray || spread(base(v)).0 - spread(base(v)).1 < 24)
        .min_by_key(|(_, v)| dist(parse(v).unwrap_or([255; 3]), c).min(dist(base(v), c)))
        .map(|(_, v)| (*v).to_string())
}

/// A text color as the editor's value (one of [`TEXT_COLORS`]), or `None` for black, white,
/// dark and very light colors (the ordinary text color of most documents, which is no color).
pub fn text_color(c: Rgb) -> Option<String> {
    let hex = format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
    if let Some((_, v)) = TEXT_COLORS.iter().find(|(_, v)| *v == hex) {
        return Some((*v).to_string());
    }
    let (max, min) = spread(c);
    let gray = max - min < 24;
    if max < 60 || min > 230 || (gray && !(min >= 100 && max <= 160)) {
        return None;
    }
    TEXT_COLORS.iter().min_by_key(|(_, v)| dist(parse(v).unwrap_or([0; 3]), c)).map(|(_, v)| (*v).to_string())
}

/// The text color and highlight in a `style` attribute (`color:…`, `background:…`,
/// `background-color:…`), as editor values. Unknown or unreadable values are left out.
pub fn from_style(style: &str) -> (Option<String>, Option<String>) {
    let (mut color, mut bg) = (None, None);
    for decl in style.split(';').take(64) {
        let Some((k, v)) = decl.split_once(':') else { continue };
        let v = v.trim().trim_end_matches("!important").trim();
        // CSS colors start with # or rgb (Word's bare RRGGBB is not CSS).
        let css = |x: &str| if x.starts_with('#') || x.to_ascii_lowercase().starts_with("rgb") { parse(x) } else { None };
        match k.trim().to_ascii_lowercase().as_str() {
            "color" => {
                if let Some(c) = css(v) {
                    color = text_color(c);
                }
            }
            // `background` may hold more than a color (`#fff url(x) no-repeat`): its first part.
            "background" | "background-color" => {
                let first = if v.to_ascii_lowercase().starts_with("rgb") {
                    v.split(')').next().map(|x| format!("{x})")).unwrap_or_default()
                } else {
                    v.split_whitespace().next().unwrap_or("").to_string()
                };
                if let Some(c) = css(&first) {
                    bg = highlight(c);
                }
            }
            _ => {}
        }
    }
    (color, bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::doc::Rgb as ExportRgb;

    #[test]
    fn every_palette_color_comes_back_as_itself() {
        for (name, v) in HIGHLIGHTS {
            // The value the exports write (export/doc.rs blends it the same way).
            let written = ExportRgb::parse(v).unwrap();
            assert_eq!(parse(&format!("#{}", written.hex())), Some([written.0, written.1, written.2]), "{name}");
            assert_eq!(highlight([written.0, written.1, written.2]).as_deref(), Some(v), "{name} via {}", written.hex());
            assert_eq!(highlight(parse(v).unwrap()).as_deref(), Some(v), "{name} as rgba");
        }
        for (name, v) in TEXT_COLORS {
            let written = ExportRgb::parse(v).unwrap();
            assert_eq!(text_color([written.0, written.1, written.2]).as_deref(), Some(v), "{name}");
        }
        assert_eq!(highlight(DEFAULT_HIGHLIGHT).as_deref(), Some(""), "the default yellow stays the default");
        // The nine colors the Notion importer writes are palette entries.
        for v in crate::importers::doc::HIGHLIGHTS {
            assert!(HIGHLIGHTS.iter().any(|(_, x)| *x == v), "{v}");
        }
        for v in crate::importers::doc::TEXT_COLORS {
            assert!(TEXT_COLORS.iter().any(|(_, x)| *x == v), "{v}");
        }
    }

    #[test]
    fn other_colors_go_to_the_nearest_entry_or_to_none() {
        // Butter used to come back as Honey (the nearest of only nine).
        assert_eq!(highlight([0xEE, 0xE5, 0xC4]).as_deref(), Some("rgba(206, 178, 80, 0.34)"));
        // Strong and pale yellows are a yellow highlight; a pure red text is a red.
        let yellowish = ["rgba(206, 178, 80, 0.34)", "rgba(203, 145, 47, 0.30)"];
        assert!(yellowish.contains(&highlight([0xFF, 0xFF, 0x00]).unwrap().as_str()));
        assert!(text_color([0xFF, 0, 0]).is_some_and(|c| ["#d04f4f", "#d44c47"].contains(&c.as_str())));
        for none in [[255, 255, 255], [0xF1, 0xF3, 0xF5], [0, 0, 0], [0x1F, 0x29, 0x37], [250, 250, 250]] {
            assert_eq!(highlight(none), None, "{none:?}");
        }
        for none in [[0, 0, 0], [0x33, 0x33, 0x33], [255, 255, 255], [0xF0, 0xF0, 0xF0], [0x20, 0x10, 0x10]] {
            assert_eq!(text_color(none), None, "{none:?}");
        }
        // A light gray is a gray highlight (Stone or Ash), never a colored one.
        let g = highlight([0xDA, 0xDA, 0xDA]).unwrap();
        assert!(g.contains("140, 142, 148") || g.contains("120, 119, 116"), "{g}");
    }

    #[test]
    fn styles_and_values_are_read_strictly() {
        assert_eq!(parse("#abc"), Some([0xAA, 0xBB, 0xCC]));
        assert_eq!(parse("D9D9D8"), Some([0xD9, 0xD9, 0xD8]));
        assert_eq!(parse("abc"), None, "three digits only with #");
        assert_eq!(parse("rgba(1, 2, 3, 0)"), None);
        for bad in ["transparent", "red", "var(--x)", "url(http://evil.example)", "expression(alert(1))", "#12", "rgb(1,2)", "rgb(nan,1,2)"] {
            assert_eq!(parse(bad), None, "{bad}");
        }
        assert_eq!(parse(&"f".repeat(100)), None);
        assert_eq!(from_style("color:c04040"), (None, None), "CSS needs the #");
        let (c, h) = from_style("COLOR: #D04F4F !important; background: #EEE5C4 url(x.png) no-repeat; font-weight:bold");
        assert_eq!(c.as_deref(), Some("#d04f4f"));
        assert_eq!(h.as_deref(), Some("rgba(206, 178, 80, 0.34)"));
        let (c, h) = from_style("color:#000000;background-color:transparent");
        assert_eq!((c, h), (None, None), "Google Docs' black on transparent is no color");
        let (_, h) = from_style("background-color: rgba(203, 145, 47, 0.30)");
        assert_eq!(h.as_deref(), Some("rgba(203, 145, 47, 0.30)"));
        let (c, h) = from_style("color:red;background:url(javascript:alert(1))");
        assert_eq!((c, h), (None, None));
    }
}
