//! Fonts for the scripts the PDF standard fonts lack: Arabic and Hebrew come from two small
//! subsets of Noto Sans (SIL Open Font License, bundled in the program; see
//! `tools/make-pdf-fonts.py`), Chinese, Japanese and Korean from the fonts Windows already has
//! (read from the Windows font folder at export time, only when the text needs them). Only the
//! glyphs a document uses are embedded, as CID-keyed TrueType fonts with a ToUnicode map.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::PathBuf;

use super::deflate::zlib_compress;
use super::ttf::Face;

static ARABIC_FONT: &[u8] = include_bytes!("../../fonts/NotoSansArabic-PDF.ttf");
static HEBREW_FONT: &[u8] = include_bytes!("../../fonts/NotoSansHebrew-PDF.ttf");

/// The biggest font file read from the system (the Chinese and Japanese collections are 10 to 20 MB).
const MAX_FONT_FILE: u64 = 96 * 1024 * 1024;

/// Which script of a text line decides the font for Chinese characters shared by all three.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hint {
    Han,
    Kana,
    Hangul,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Script {
    Arabic,
    Hebrew,
    Cjk,
}

pub fn script(c: char) -> Option<Script> {
    match c as u32 {
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => Some(Script::Arabic),
        0x0590..=0x05FF | 0xFB1D..=0xFB4F => Some(Script::Hebrew),
        _ if is_cjk(c) => Some(Script::Cjk),
        _ => None,
    }
}

pub fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x2FDF | 0x3000..=0x303F | 0x3040..=0x30FF | 0x3100..=0x312F | 0x3130..=0x318F | 0x31C0..=0x31FF | 0x3200..=0x33FF
        | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xA960..=0xA97F | 0xAC00..=0xD7FF | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFFEF
        | 0x20000..=0x2FA1F | 0x30000..=0x3134F)
}

/// Fonts in the Windows font folder, best first, by what the text needs.
const FILES_HAN: [&str; 9] = ["msyh.ttc", "simsun.ttc", "Deng.ttf", "msjh.ttc", "mingliu.ttc", "YuGothR.ttc", "meiryo.ttc", "msgothic.ttc", "malgun.ttf"];
const FILES_KANA: [&str; 6] = ["YuGothR.ttc", "meiryo.ttc", "msgothic.ttc", "YuGothM.ttc", "msyh.ttc", "simsun.ttc"];
const FILES_HANGUL: [&str; 5] = ["malgun.ttf", "gulim.ttc", "batang.ttc", "msyh.ttc", "simsun.ttc"];

/// One glyph the writer can draw.
#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub face: usize,
    pub gid: u16,
    /// Advance in ems (1.0 = the font size); 0 for marks.
    pub adv: f32,
    pub mark: bool,
    /// Horizontal extent of the outline in ems.
    pub bounds: Option<(f32, f32)>,
}

struct Slot {
    face: Face,
    name: String,
    /// gid -> (advance in 1/1000, the text it stands for)
    used: BTreeMap<u16, (u32, String)>,
}

pub struct Pool {
    slots: Vec<Slot>,
    dir: Option<PathBuf>,
    arabic: Option<Option<usize>>,
    hebrew: Option<Option<usize>>,
    files: HashMap<&'static str, Option<usize>>,
}

impl Pool {
    pub fn new(fonts_dir: Option<PathBuf>) -> Pool {
        Pool { slots: Vec::new(), dir: fonts_dir, arabic: None, hebrew: None, files: HashMap::new() }
    }

    /// The pool for a real export: the Windows font folder from the WINDIR variable.
    pub fn system() -> Pool {
        let dir = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot")).filter(|v| !v.is_empty()).map(|w| PathBuf::from(w).join("Fonts"));
        Pool::new(dir)
    }

    fn add(&mut self, face: Face, fallback_name: &str) -> usize {
        let name = face.postscript_name().unwrap_or_else(|| fallback_name.to_string());
        self.slots.push(Slot { face, name, used: BTreeMap::new() });
        self.slots.len() - 1
    }

    fn bundled(&mut self, hebrew: bool) -> Option<usize> {
        let cached = if hebrew { self.hebrew } else { self.arabic };
        if let Some(c) = cached {
            return c;
        }
        let loaded = if hebrew { Face::parse(HEBREW_FONT, 0).map(|f| self.add(f, "NotoSansHebrew")) } else { Face::parse(ARABIC_FONT, 0).map(|f| self.add(f, "NotoSansArabic")) };
        if hebrew {
            self.hebrew = Some(loaded);
        } else {
            self.arabic = Some(loaded);
        }
        loaded
    }

    fn system_font(&mut self, file: &'static str) -> Option<usize> {
        if let Some(c) = self.files.get(file) {
            return *c;
        }
        let loaded = (|| {
            let path = self.dir.as_ref()?.join(file);
            let meta = std::fs::metadata(&path).ok()?;
            if !meta.is_file() || meta.len() > MAX_FONT_FILE {
                return None;
            }
            let face = Face::parse(std::fs::read(&path).ok()?, 0)?;
            Some(self.add(face, "CJKFont"))
        })();
        self.files.insert(file, loaded);
        loaded
    }

    /// True when some font file for Chinese, Japanese or Korean text can be used.
    pub fn has_cjk_font(&mut self) -> bool {
        FILES_HAN.iter().chain(&FILES_KANA).chain(&FILES_HANGUL).any(|f| self.system_font(f).is_some())
    }

    /// The glyph for a character in the font its script uses. `text` is what the glyph stands for
    /// (the source characters). `None` when no font has the character.
    pub fn glyph(&mut self, c: char, hint: Hint, text: &str) -> Option<GlyphInfo> {
        let idx_gid = match script(c)? {
            Script::Arabic => self.bundled(false).map(|i| (i, self.slots[i].face.glyph(c))),
            Script::Hebrew => self.bundled(true).map(|i| (i, self.slots[i].face.glyph(c))),
            Script::Cjk => {
                let kana = c as u32 >= 0x3040 && c as u32 <= 0x30FF;
                let hangul = matches!(c as u32, 0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7FF);
                let list: Vec<&'static str> = if kana || (hint == Hint::Kana && !hangul) {
                    FILES_KANA.iter().chain(&FILES_HAN).copied().collect()
                } else if hangul || hint == Hint::Hangul {
                    FILES_HANGUL.iter().chain(&FILES_HAN).copied().collect()
                } else {
                    FILES_HAN.iter().chain(&FILES_KANA).chain(&FILES_HANGUL).copied().collect()
                };
                let mut found = None;
                for f in list {
                    if let Some(i) = self.system_font(f) {
                        let g = self.slots[i].face.glyph(c);
                        if g != 0 {
                            found = Some((i, g));
                            break;
                        }
                    }
                }
                found
            }
        }?;
        let (face, gid) = idx_gid;
        if gid == 0 {
            return None;
        }
        let slot = &mut self.slots[face];
        let mark = super::arabic::is_mark(c);
        let adv_units = if mark { 0 } else { slot.face.advance(gid) };
        slot.used.entry(gid).or_insert_with(|| (adv_units, text.to_string()));
        Some(GlyphInfo {
            face,
            gid,
            adv: adv_units as f32 / 1000.0,
            mark,
            bounds: slot.face.bounds(gid).map(|(a, b)| (a as f32 / 1000.0, b as f32 / 1000.0)),
        })
    }

    /// The fonts that have glyphs in the document: (index in the pool, font resource number).
    pub fn used(&self) -> Vec<usize> {
        (0..self.slots.len()).filter(|&i| !self.slots[i].used.is_empty()).collect()
    }

    /// The PDF objects of one font, as the five objects Type0, CIDFont, FontDescriptor, font file and
    /// ToUnicode, numbered from `first`. `None` when the font cannot be subset.
    pub fn objects(&self, index: usize, first: usize) -> Option<Vec<Vec<u8>>> {
        let slot = &self.slots[index];
        let gids: BTreeSet<u16> = slot.used.keys().copied().collect();
        let file = slot.face.subset(&gids)?;
        let tag = subset_tag(&slot.name, &gids);
        let name = format!("{tag}+{}", slot.name);
        let f = &slot.face;
        // Widths: runs of consecutive glyph numbers share one entry.
        let mut w = String::new();
        let mut run: Vec<(u16, u32)> = Vec::new();
        let flush = |run: &mut Vec<(u16, u32)>, w: &mut String| {
            if let Some(&(g0, _)) = run.first() {
                let list: Vec<String> = run.iter().map(|(_, a)| a.to_string()).collect();
                let _ = write!(w, "{g0} [{}] ", list.join(" "));
            }
            run.clear();
        };
        for (&g, (adv, _)) in &slot.used {
            if run.last().is_some_and(|&(p, _)| p + 1 != g) {
                flush(&mut run, &mut w);
            }
            run.push((g, *adv));
        }
        flush(&mut run, &mut w);
        let mut cmap = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        let entries: Vec<(&u16, &(u32, String))> = slot.used.iter().collect();
        for chunk in entries.chunks(100) {
            let _ = writeln!(cmap, "{} beginbfchar", chunk.len());
            for (g, (_, text)) in chunk {
                let mut u = String::new();
                for unit in text.encode_utf16() {
                    let _ = write!(u, "{unit:04X}");
                }
                let _ = writeln!(cmap, "<{g:04X}> <{u}>");
            }
            cmap.push_str("endbfchar\n");
        }
        cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        let stream = |dict: &str, data: &[u8]| -> Vec<u8> {
            let mut v = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
            v.extend_from_slice(data);
            v.extend_from_slice(b"\nendstream");
            v
        };
        let (cid, desc, ffile, tounicode) = (first + 1, first + 2, first + 3, first + 4);
        let type0 = format!("<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H /DescendantFonts [{cid} 0 R] /ToUnicode {tounicode} 0 R >>");
        let cidfont = format!(
            "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{name} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {desc} 0 R /CIDToGIDMap /Identity /DW 1000 /W [{w}] >>"
        );
        let descriptor = format!(
            "<< /Type /FontDescriptor /FontName /{name} /Flags 4 /FontBBox [{} {} {} {}] /ItalicAngle {} /Ascent {} /Descent {} /CapHeight {} /StemV 80 /FontFile2 {ffile} 0 R >>",
            f.bbox[0], f.bbox[1], f.bbox[2], f.bbox[3], f.italic_angle, f.ascent, f.descent, f.cap_height
        );
        Some(vec![
            type0.into_bytes(),
            cidfont.into_bytes(),
            descriptor.into_bytes(),
            stream(&format!("/Length1 {} /Filter /FlateDecode", file.len()), &zlib_compress(&file)),
            stream("/Filter /FlateDecode", &zlib_compress(cmap.as_bytes())),
        ])
    }
}

/// Six capital letters made from the font and the glyphs, as PDF asks of subset fonts.
fn subset_tag(name: &str, gids: &BTreeSet<u16>) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in name.bytes().chain(gids.iter().flat_map(|g| g.to_be_bytes())) {
        h = (h ^ b as u32).wrapping_mul(16_777_619);
    }
    (0..6)
        .map(|_| {
            let c = (b'A' + (h % 26) as u8) as char;
            h = h / 26 + 0x9E37;
            c
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::ttf::tests::test_font;
    use super::*;

    #[test]
    fn scripts() {
        assert_eq!(script('م'), Some(Script::Arabic));
        assert_eq!(script('\u{FEFB}'), Some(Script::Arabic));
        assert_eq!(script('ש'), Some(Script::Hebrew));
        assert_eq!(script('中'), Some(Script::Cjk));
        assert_eq!(script('あ'), Some(Script::Cjk));
        assert_eq!(script('한'), Some(Script::Cjk));
        assert_eq!(script('，'), Some(Script::Cjk));
        assert_eq!(script('a'), None);
        assert_eq!(script('é'), None);
        assert_eq!(script('😀'), None);
    }

    #[test]
    fn the_bundled_fonts_load_and_have_the_forms_the_shaper_uses() {
        let mut p = Pool::new(None);
        let ar = p.bundled(false).expect("the Arabic font loads");
        let he = p.bundled(true).expect("the Hebrew font loads");
        let (a, h) = (&p.slots[ar].face, &p.slots[he].face);
        // Every presentation form the shaping table can produce is in the font.
        for c in (0xFE70u32..=0xFEFC).filter(|c| *c != 0xFE75).chain([0xFB56, 0xFB57, 0xFB58, 0xFB59, 0xFB7A, 0xFB7D, 0xFB8E, 0xFB91, 0xFB92, 0xFB95, 0xFBFC, 0xFBFF, 0xFB8A, 0xFB8B, 0xFBAE, 0xFBAF]) {
            assert!(a.glyph(char::from_u32(c).unwrap()) != 0, "U+{c:04X} in the Arabic font");
        }
        for c in "ءآأؤإئابةتثجحخدذرزسشصضطظعغفقكلمنهوىي\u{064E}\u{0651}ـ،؟٠٩".chars() {
            assert!(a.glyph(c) != 0, "{c:?} in the Arabic font");
        }
        for c in "אבגדהוזחטיכלמנסעפצקרשת".chars().chain(['ך', 'ם', 'ן', 'ף', 'ץ', '\u{05B4}', '\u{05BC}', '\u{05C1}', '־', '׳', '״']) {
            assert!(h.glyph(c) != 0, "{c:?} in the Hebrew font");
        }
        assert_eq!(a.units, 1000);
    }

    #[test]
    fn glyphs_come_from_the_font_of_their_script_and_are_recorded() {
        let mut p = Pool::new(None);
        let g = p.glyph('\u{FE91}', Hint::Han, "ب").unwrap();
        assert!(g.adv > 0.1 && !g.mark);
        let m = p.glyph('\u{064E}', Hint::Han, "\u{064E}").unwrap();
        assert!(m.mark && m.adv == 0.0 && m.bounds.is_some());
        let h = p.glyph('ש', Hint::Han, "ש").unwrap();
        assert_ne!(g.face, h.face);
        assert!(p.glyph('a', Hint::Han, "a").is_none(), "Latin letters are not this pool's job");
        assert!(p.glyph('中', Hint::Han, "中").is_none(), "no system font folder, no Chinese");
        assert_eq!(p.used().len(), 2);
        let objs = p.objects(g.face, 10).unwrap();
        assert_eq!(objs.len(), 5);
        let t0 = String::from_utf8_lossy(&objs[0]);
        assert!(t0.contains("/Subtype /Type0") && t0.contains("/Encoding /Identity-H") && t0.contains("/DescendantFonts [11 0 R]") && t0.contains("/ToUnicode 14 0 R"));
        let cid = String::from_utf8_lossy(&objs[1]);
        assert!(cid.contains("/CIDFontType2") && cid.contains("/CIDToGIDMap /Identity") && cid.contains("/FontDescriptor 12 0 R") && cid.contains("/W ["));
        let d = String::from_utf8_lossy(&objs[2]);
        assert!(d.contains("/FontFile2 13 0 R") && d.contains("/Flags 4"));
        assert!(String::from_utf8_lossy(&objs[3]).contains("/Length1 "));
    }

    #[test]
    fn chinese_japanese_and_korean_come_from_the_system_font_folder() {
        let dir = std::env::temp_dir().join(format!("notaya-pdf-fonts-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("msyh.ttc"), test_font(0)).unwrap();
        std::fs::write(dir.join("malgun.ttf"), test_font(0x0002)).unwrap(); // forbids embedding: skipped
        let mut p = Pool::new(Some(dir.clone()));
        assert!(p.has_cjk_font());
        let g = p.glyph('中', Hint::Han, "中").unwrap();
        assert_eq!(g.adv, 1.0);
        assert!(p.glyph('あ', Hint::Kana, "あ").is_some(), "Japanese falls back to the Chinese font when it has the glyph");
        assert!(p.system_font("malgun.ttf").is_none(), "a font that forbids embedding is not used");
        assert!(p.glyph('가', Hint::Hangul, "가").is_some(), "Korean falls back to another font that has the glyph");
        assert!(p.glyph('文', Hint::Han, "文").is_none(), "a character the font lacks");
        let objs = p.objects(g.face, 1).unwrap();
        let tu = zlib_decompress_for_test(&objs[4]);
        assert!(tu.contains("<0003> <4E2D>") && tu.contains("<0004> <3042>"), "{tu}");
        std::fs::remove_dir_all(&dir).ok();
    }

    fn zlib_decompress_for_test(obj: &[u8]) -> String {
        let a = obj.windows(7).position(|w| w == b"stream\n").unwrap() + 7;
        let data = &obj[a..obj.len() - "\nendstream".len()];
        String::from_utf8(super::super::deflate::zlib_decompress(data, 10_000_000).unwrap()).unwrap()
    }

    #[test]
    fn an_oversized_or_missing_font_file_is_ignored() {
        let dir = std::env::temp_dir().join(format!("notaya-pdf-fonts-b-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("msyh.ttc"), b"not a font").unwrap();
        let mut p = Pool::new(Some(dir.clone()));
        assert!(!p.has_cjk_font());
        assert!(Pool::new(None).glyph('中', Hint::Han, "中").is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn subset_tags_are_six_capitals() {
        let t = subset_tag("Test", &BTreeSet::from([1, 2]));
        assert_eq!(t.len(), 6);
        assert!(t.chars().all(|c| c.is_ascii_uppercase()));
        assert_ne!(t, subset_tag("Test", &BTreeSet::from([1, 3])));
    }
}
