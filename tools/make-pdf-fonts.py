#!/usr/bin/env python3
"""Makes the two small fonts that Notaya embeds in PDF exports (src-tauri/fonts/).

Usage: python3 tools/make-pdf-fonts.py NotoSansArabic_400Regular.ttf NotoSansHebrew_400Regular.ttf OUT_DIR

The inputs are the unmodified static "Regular" files of Noto Sans Arabic and Noto Sans Hebrew
(Copyright 2022 The Noto Project Authors, SIL Open Font License 1.1, no Reserved Font Name), taken
from the npm packages @expo-google-fonts/noto-sans-arabic 0.4.3 and @expo-google-fonts/noto-sans-hebrew 0.4.1.
The script only removes what Notaya does not use (OpenType layout tables, hinting, glyphs outside the
blocks listed below) with fontTools 4.68.0 (pip install fonttools). No outline is changed. The
shaping of Arabic is done by Notaya itself with the Presentation Forms-B glyphs, so GSUB/GPOS are not needed.
"""
import sys
from fontTools import subset
from fontTools.ttLib import TTFont

ARABIC = list(range(0x0600, 0x0700)) + list(range(0xFB50, 0xFC00)) + list(range(0xFE70, 0xFF00)) + [0x200C, 0x200D, 0x00A0, 0x0020]
HEBREW = list(range(0x0590, 0x0600)) + list(range(0xFB1D, 0xFB50)) + [0x00A0, 0x0020]


def make(src, dst, codes):
    opts = subset.Options()
    opts.layout_features = []
    opts.hinting = False
    opts.glyph_names = False
    opts.notdef_outline = True
    opts.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14]
    opts.drop_tables += ["GSUB", "GPOS", "GDEF", "DSIG", "STAT", "avar", "fvar", "gvar", "HVAR", "MVAR", "gasp", "meta"]
    font = TTFont(src)
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=codes)
    sub.subset(font)
    font.save(dst)
    print(dst, len(open(dst, "rb").read()), "bytes")


if __name__ == "__main__":
    ar, he, out = sys.argv[1:4]
    make(ar, out + "/NotoSansArabic-PDF.ttf", ARABIC)
    make(he, out + "/NotoSansHebrew-PDF.ttf", HEBREW)
