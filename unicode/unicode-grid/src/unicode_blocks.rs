#[derive(Debug, Clone, Copy)]
pub struct UnicodeBlock {
    pub name: &'static str,
    pub start: u32,
    pub end: u32,
}

#[allow(unused)]
pub const BASIC_LATIN: UnicodeBlock = UnicodeBlock {
    name: "Basic Latin",
    start: 0x0000,
    end: 0x007F,
};

#[allow(unused)]
pub const LATIN_1_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Latin-1 Supplement",
    start: 0x0080,
    end: 0x00FF,
};

#[allow(unused)]
pub const LATIN_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended-A",
    start: 0x0100,
    end: 0x017F,
};

#[allow(unused)]
pub const LATIN_EXTENDED_B: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended-B",
    start: 0x0180,
    end: 0x024F,
};

#[allow(unused)]
pub const IPA_EXTENSIONS: UnicodeBlock = UnicodeBlock {
    name: "IPA Extensions",
    start: 0x0250,
    end: 0x02AF,
};

#[allow(unused)]
pub const SPACING_MODIFIER_LETTERS: UnicodeBlock = UnicodeBlock {
    name: "Spacing Modifier Letters",
    start: 0x02B0,
    end: 0x02FF,
};

#[allow(unused)]
pub const COMBINING_DIACRITICAL_MARKS: UnicodeBlock = UnicodeBlock {
    name: "Combining Diacritical Marks",
    start: 0x0300,
    end: 0x036F,
};

#[allow(unused)]
pub const GREEK_AND_COPTIC: UnicodeBlock = UnicodeBlock {
    name: "Greek and Coptic",
    start: 0x0370,
    end: 0x03FF,
};

#[allow(unused)]
pub const CYRILLIC: UnicodeBlock = UnicodeBlock {
    name: "Cyrillic",
    start: 0x0400,
    end: 0x04FF,
};

#[allow(unused)]
pub const CYRILLIC_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Cyrillic Supplement",
    start: 0x0500,
    end: 0x052F,
};

#[allow(unused)]
pub const ARMENIAN: UnicodeBlock = UnicodeBlock {
    name: "Armenian",
    start: 0x0530,
    end: 0x058F,
};

#[allow(unused)]
pub const HEBREW: UnicodeBlock = UnicodeBlock {
    name: "Hebrew",
    start: 0x0590,
    end: 0x05FF,
};

#[allow(unused)]
pub const ARABIC: UnicodeBlock = UnicodeBlock {
    name: "Arabic",
    start: 0x0600,
    end: 0x06FF,
};

#[allow(unused)]
pub const SYRIAC: UnicodeBlock = UnicodeBlock {
    name: "Syriac",
    start: 0x0700,
    end: 0x074F,
};

#[allow(unused)]
pub const ARABIC_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Arabic Supplement",
    start: 0x0750,
    end: 0x077F,
};

#[allow(unused)]
pub const THAANA: UnicodeBlock = UnicodeBlock {
    name: "Thaana",
    start: 0x0780,
    end: 0x07BF,
};

#[allow(unused)]
pub const NKO: UnicodeBlock = UnicodeBlock {
    name: "NKo",
    start: 0x07C0,
    end: 0x07FF,
};


#[allow(unused)]
pub const SAMARITAN: UnicodeBlock = UnicodeBlock {
    name: "Samaritan",
    start: 0x0800,
    end: 0x083F,
};

#[allow(unused)]
pub const MANDAIC: UnicodeBlock = UnicodeBlock {
    name: "Mandaic",
    start: 0x0840,
    end: 0x085F,
};

#[allow(unused)]
pub const SYRIAC_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Syriac Supplement",
    start: 0x0860,
    end: 0x086F,
};

#[allow(unused)]
pub const ARABIC_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Arabic Extended-A",
    start: 0x08A0,
    end: 0x08FF,
};

#[allow(unused)]
pub const DEVANAGARI: UnicodeBlock = UnicodeBlock {
    name: "Devanagari",
    start: 0x0900,
    end: 0x097F,
};

#[allow(unused)]
pub const BENGALI: UnicodeBlock = UnicodeBlock {
    name: "Bengali",
    start: 0x0980,
    end: 0x09FF,
};

#[allow(unused)]
pub const GURMUKHI: UnicodeBlock = UnicodeBlock {
    name: "Gurmukhi",
    start: 0x0A00,
    end: 0x0A7F,
};

#[allow(unused)]
pub const GUJARATI: UnicodeBlock = UnicodeBlock {
    name: "Gujarati",
    start: 0x0A80,
    end: 0x0AFF,
};

#[allow(unused)]
pub const ORIYA: UnicodeBlock = UnicodeBlock {
    name: "Oriya",
    start: 0x0B00,
    end: 0x0B7F,
};

#[allow(unused)]
pub const TAMIL: UnicodeBlock = UnicodeBlock {
    name: "Tamil",
    start: 0x0B80,
    end: 0x0BFF,
};

#[allow(unused)]
pub const TELUGU: UnicodeBlock = UnicodeBlock {
    name: "Telugu",
    start: 0x0C00,
    end: 0x0C7F,
};

#[allow(unused)]
pub const KANNADA: UnicodeBlock = UnicodeBlock {
    name: "Kannada",
    start: 0x0C80,
    end: 0x0CFF,
};

#[allow(unused)]
pub const MALAYALAM: UnicodeBlock = UnicodeBlock {
    name: "Malayalam",
    start: 0x0D00,
    end: 0x0D7F,
};

#[allow(unused)]
pub const SINHALA: UnicodeBlock = UnicodeBlock {
    name: "Sinhala",
    start: 0x0D80,
    end: 0x0DFF,
};

#[allow(unused)]
pub const THAI: UnicodeBlock = UnicodeBlock {
    name: "Thai",
    start: 0x0E00,
    end: 0x0E7F,
};

#[allow(unused)]
pub const LAO: UnicodeBlock = UnicodeBlock {
    name: "Lao",
    start: 0x0E80,
    end: 0x0EFF,
};

#[allow(unused)]
pub const TIBETAN: UnicodeBlock = UnicodeBlock {
    name: "Tibetan",
    start: 0x0F00,
    end: 0x0FFF,
};

#[allow(unused)]
pub const MYANMAR: UnicodeBlock = UnicodeBlock {
    name: "Myanmar",
    start: 0x1000,
    end: 0x109F,
};

#[allow(unused)]
pub const GEORGIAN: UnicodeBlock = UnicodeBlock {
    name: "Georgian",
    start: 0x10A0,
    end: 0x10FF,
};

#[allow(unused)]
pub const HANGUL_JAMO: UnicodeBlock = UnicodeBlock {
    name: "Hangul Jamo",
    start: 0x1100,
    end: 0x11FF,
};

#[allow(unused)]
pub const ETHIOPIC: UnicodeBlock = UnicodeBlock {
    name: "Ethiopic",
    start: 0x1200,
    end: 0x137F,
};

#[allow(unused)]
pub const ETHIOPIC_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Ethiopic Supplement",
    start: 0x1380,
    end: 0x139F,
};

#[allow(unused)]
pub const CHEROKEE: UnicodeBlock = UnicodeBlock {
    name: "Cherokee",
    start: 0x13A0,
    end: 0x13FF,
};

#[allow(unused)]
pub const UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS: UnicodeBlock = UnicodeBlock {
    name: "Unified Canadian Aboriginal Syllabics",
    start: 0x1400,
    end: 0x167F,
};

#[allow(unused)]
pub const OGHAM: UnicodeBlock = UnicodeBlock {
    name: "Ogham",
    start: 0x1680,
    end: 0x169F,
};

#[allow(unused)]
pub const RUNIC: UnicodeBlock = UnicodeBlock {
    name: "Runic",
    start: 0x16A0,
    end: 0x16FF,
};

#[allow(unused)]
pub const TAGALOG: UnicodeBlock = UnicodeBlock {
    name: "Tagalog",
    start: 0x1700,
    end: 0x171F,
};

#[allow(unused)]
pub const HANUNOO: UnicodeBlock = UnicodeBlock {
    name: "Hanunoo",
    start: 0x1720,
    end: 0x173F,
};

#[allow(unused)]
pub const BUHID: UnicodeBlock = UnicodeBlock {
    name: "Buhid",
    start: 0x1740,
    end: 0x175F,
};

#[allow(unused)]
pub const TAGBANWA: UnicodeBlock = UnicodeBlock {
    name: "Tagbanwa",
    start: 0x1760,
    end: 0x177F,
};

#[allow(unused)]
pub const KHMER: UnicodeBlock = UnicodeBlock {
    name: "Khmer",
    start: 0x1780,
    end: 0x17FF,
};

#[allow(unused)]
pub const MONGOLIAN: UnicodeBlock = UnicodeBlock {
    name: "Mongolian",
    start: 0x1800,
    end: 0x18AF,
};

#[allow(unused)]
pub const UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Unified Canadian Aboriginal Syllabics Extended",
    start: 0x18B0,
    end: 0x18FF,
};

#[allow(unused)]
pub const LIMBU: UnicodeBlock = UnicodeBlock {
    name: "Limbu",
    start: 0x1900,
    end: 0x194F,
};

#[allow(unused)]
pub const TAI_LE: UnicodeBlock = UnicodeBlock {
    name: "Tai Le",
    start: 0x1950,
    end: 0x197F,
};

#[allow(unused)]
pub const NEW_TAI_LUE: UnicodeBlock = UnicodeBlock {
    name: "New Tai Lue",
    start: 0x1980,
    end: 0x19DF,
};

#[allow(unused)]
pub const KHMER_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Khmer Symbols",
    start: 0x19E0,
    end: 0x19FF,
};

#[allow(unused)]
pub const BUGINESE: UnicodeBlock = UnicodeBlock {
    name: "Buginese",
    start: 0x1A00,
    end: 0x1A1F,
};

#[allow(unused)]
pub const TAI_THAM: UnicodeBlock = UnicodeBlock {
    name: "Tai Tham",
    start: 0x1A20,
    end: 0x1AAF,
};


#[allow(unused)]
pub const COMBINING_DIACRITICAL_MARKS_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Combining Diacritical Marks Extended",
    start: 0x1AB0,
    end: 0x1AFF,
};

#[allow(unused)]
pub const BALINESE: UnicodeBlock = UnicodeBlock {
    name: "Balinese",
    start: 0x1B00,
    end: 0x1B7F,
};

#[allow(unused)]
pub const SUNDANESE: UnicodeBlock = UnicodeBlock {
    name: "Sundanese",
    start: 0x1B80,
    end: 0x1BBF,
};

#[allow(unused)]
pub const BATAK: UnicodeBlock = UnicodeBlock {
    name: "Batak",
    start: 0x1BC0,
    end: 0x1BFF,
};

#[allow(unused)]
pub const LEPCHA: UnicodeBlock = UnicodeBlock {
    name: "Lepcha",
    start: 0x1C00,
    end: 0x1C4F,
};

#[allow(unused)]
pub const OL_CHIKI: UnicodeBlock = UnicodeBlock {
    name: "Ol Chiki",
    start: 0x1C50,
    end: 0x1C7F,
};

#[allow(unused)]
pub const CYRILLIC_EXTENDED_C: UnicodeBlock = UnicodeBlock {
    name: "Cyrillic Extended-C",
    start: 0x1C80,
    end: 0x1C8F,
};

#[allow(unused)]
pub const GEORGIAN_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Georgian Extended",
    start: 0x1C90,
    end: 0x1CBF,
};

#[allow(unused)]
pub const SUNDANESE_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Sundanese Supplement",
    start: 0x1CC0,
    end: 0x1CCF,
};

#[allow(unused)]
pub const VEDIC_EXTENSIONS: UnicodeBlock = UnicodeBlock {
    name: "Vedic Extensions",
    start: 0x1CD0,
    end: 0x1CFF,
};

#[allow(unused)]
pub const PHONETIC_EXTENSIONS: UnicodeBlock = UnicodeBlock {
    name: "Phonetic Extensions",
    start: 0x1D00,
    end: 0x1D7F,
};

#[allow(unused)]
pub const PHONETIC_EXTENSIONS_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Phonetic Extensions Supplement",
    start: 0x1D80,
    end: 0x1DBF,
};

#[allow(unused)]
pub const COMBINING_DIACRITICAL_MARKS_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Combining Diacritical Marks Supplement",
    start: 0x1DC0,
    end: 0x1DFF,
};

#[allow(unused)]
pub const LATIN_EXTENDED_ADDITIONAL: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended Additional",
    start: 0x1E00,
    end: 0x1EFF,
};

#[allow(unused)]
pub const GREEK_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Greek Extended",
    start: 0x1F00,
    end: 0x1FFF,
};

#[allow(unused)]
pub const GENERAL_PUNCTUATION: UnicodeBlock = UnicodeBlock {
    name: "General Punctuation",
    start: 0x2000,
    end: 0x206F,
};

#[allow(unused)]
pub const SUPERSCRIPTS_AND_SUBSCRIPTS: UnicodeBlock = UnicodeBlock {
    name: "Superscripts and Subscripts",
    start: 0x2070,
    end: 0x209F,
};

#[allow(unused)]
pub const CURRENCY_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Currency Symbols",
    start: 0x20A0,
    end: 0x20CF,
};

#[allow(unused)]
pub const COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Combining Diacritical Marks for Symbols",
    start: 0x20D0,
    end: 0x20FF,
};

#[allow(unused)]
pub const LETTERLIKE_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Letterlike Symbols",
    start: 0x2100,
    end: 0x214F,
};

#[allow(unused)]
pub const NUMBER_FORMS: UnicodeBlock = UnicodeBlock {
    name: "Number Forms",
    start: 0x2150,
    end: 0x218F,
};

#[allow(unused)]
pub const ARROWS: UnicodeBlock = UnicodeBlock {
    name: "Arrows",
    start: 0x2190,
    end: 0x21FF,
};

#[allow(unused)]
pub const MATHEMATICAL_OPERATORS: UnicodeBlock = UnicodeBlock {
    name: "Mathematical Operators",
    start: 0x2200,
    end: 0x22FF,
};

#[allow(unused)]
pub const MISCELLANEOUS_TECHNICAL: UnicodeBlock = UnicodeBlock {
    name: "Miscellaneous Technical",
    start: 0x2300,
    end: 0x23FF,
};

#[allow(unused)]
pub const CONTROL_PICTURES: UnicodeBlock = UnicodeBlock {
    name: "Control Pictures",
    start: 0x2400,
    end: 0x243F,
};

#[allow(unused)]
pub const OPTICAL_CHARACTER_RECOGNITION: UnicodeBlock = UnicodeBlock {
    name: "Optical Character Recognition",
    start: 0x2440,
    end: 0x245F,
};

#[allow(unused)]
pub const ENCLOSED_ALPHANUMERICS: UnicodeBlock = UnicodeBlock {
    name: "Enclosed Alphanumerics",
    start: 0x2460,
    end: 0x24FF,
};

#[allow(unused)]
pub const BOX_DRAWING: UnicodeBlock = UnicodeBlock {
    name: "Box Drawing",
    start: 0x2500,
    end: 0x257F,
};

#[allow(unused)]
pub const BLOCK_ELEMENTS: UnicodeBlock = UnicodeBlock {
    name: "Block Elements",
    start: 0x2580,
    end: 0x259F,
};

#[allow(unused)]
pub const GEOMETRIC_SHAPES: UnicodeBlock = UnicodeBlock {
    name: "Geometric Shapes",
    start: 0x25A0,
    end: 0x25FF,
};

#[allow(unused)]
pub const MISCELLANEOUS_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Miscellaneous Symbols",
    start: 0x2600,
    end: 0x26FF,
};

#[allow(unused)]
pub const DINGBATS: UnicodeBlock = UnicodeBlock {
    name: "Dingbats",
    start: 0x2700,
    end: 0x27BF,
};

#[allow(unused)]
pub const MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A: UnicodeBlock = UnicodeBlock {
    name: "Miscellaneous Mathematical Symbols-A",
    start: 0x27C0,
    end: 0x27EF,
};

#[allow(unused)]
pub const SUPPLEMENTAL_ARROWS_A: UnicodeBlock = UnicodeBlock {
    name: "Supplemental Arrows-A",
    start: 0x27F0,
    end: 0x27FF,
};

#[allow(unused)]
pub const BRAILLE_PATTERNS: UnicodeBlock = UnicodeBlock {
    name: "Braille Patterns",
    start: 0x2800,
    end: 0x28FF,
};

#[allow(unused)]
pub const SUPPLEMENTAL_ARROWS_B: UnicodeBlock = UnicodeBlock {
    name: "Supplemental Arrows-B",
    start: 0x2900,
    end: 0x297F,
};

#[allow(unused)]
pub const MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B: UnicodeBlock = UnicodeBlock {
    name: "Miscellaneous Mathematical Symbols-B",
    start: 0x2980,
    end: 0x29FF,
};

#[allow(unused)]
pub const SUPPLEMENTAL_MATHEMATICAL_OPERATORS: UnicodeBlock = UnicodeBlock {
    name: "Supplemental Mathematical Operators",
    start: 0x2A00,
    end: 0x2AFF,
};

#[allow(unused)]
pub const MISCELLANEOUS_SYMBOLS_AND_ARROWS: UnicodeBlock = UnicodeBlock {
    name: "Miscellaneous Symbols and Arrows",
    start: 0x2B00,
    end: 0x2BFF,
};

#[allow(unused)]
pub const GLAGOLITIC: UnicodeBlock = UnicodeBlock {
    name: "Glagolitic",
    start: 0x2C00,
    end: 0x2C5F,
};

#[allow(unused)]
pub const LATIN_EXTENDED_C: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended-C",
    start: 0x2C60,
    end: 0x2C7F,
};

#[allow(unused)]
pub const COPTIC: UnicodeBlock = UnicodeBlock {
    name: "Coptic",
    start: 0x2C80,
    end: 0x2CFF,
};

#[allow(unused)]
pub const GEORGIAN_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Georgian Supplement",
    start: 0x2D00,
    end: 0x2D2F,
};

#[allow(unused)]
pub const TIFINAGH: UnicodeBlock = UnicodeBlock {
    name: "Tifinagh",
    start: 0x2D30,
    end: 0x2D7F,
};

#[allow(unused)]
pub const ETHIOPIC_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Ethiopic Extended",
    start: 0x2D80,
    end: 0x2DDF,
};

#[allow(unused)]
pub const CYRILLIC_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Cyrillic Extended-A",
    start: 0x2DE0,
    end: 0x2DFF,
};

#[allow(unused)]
pub const SUPPLEMENTAL_PUNCTUATION: UnicodeBlock = UnicodeBlock {
    name: "Supplemental Punctuation",
    start: 0x2E00,
    end: 0x2E7F,
};

#[allow(unused)]
pub const CJK_RADICALS_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "CJK Radicals Supplement",
    start: 0x2E80,
    end: 0x2EFF,
};

#[allow(unused)]
pub const KANGXI_RADICALS: UnicodeBlock = UnicodeBlock {
    name: "Kangxi Radicals",
    start: 0x2F00,
    end: 0x2FDF,
};

#[allow(unused)]
pub const IDEOGRAPHIC_DESCRIPTION_CHARACTERS: UnicodeBlock = UnicodeBlock {
    name: "Ideographic Description Characters",
    start: 0x2FF0,
    end: 0x2FFF,
};

#[allow(unused)]
pub const CJK_SYMBOLS_AND_PUNCTUATION: UnicodeBlock = UnicodeBlock {
    name: "CJK Symbols and Punctuation",
    start: 0x3000,
    end: 0x303F,
};

#[allow(unused)]
pub const HIRAGANA: UnicodeBlock = UnicodeBlock {
    name: "Hiragana",
    start: 0x3040,
    end: 0x309F,
};

#[allow(unused)]
pub const KATAKANA: UnicodeBlock = UnicodeBlock {
    name: "Katakana",
    start: 0x30A0,
    end: 0x30FF,
};

#[allow(unused)]
pub const BOPOMOFO: UnicodeBlock = UnicodeBlock {
    name: "Bopomofo",
    start: 0x3100,
    end: 0x312F,
};

#[allow(unused)]
pub const HANGUL_COMPATIBILITY_JAMO: UnicodeBlock = UnicodeBlock {
    name: "Hangul Compatibility Jamo",
    start: 0x3130,
    end: 0x318F,
};

#[allow(unused)]
pub const KANBUN: UnicodeBlock = UnicodeBlock {
    name: "Kanbun",
    start: 0x3190,
    end: 0x319F,
};

#[allow(unused)]
pub const BOPOMOFO_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Bopomofo Extended",
    start: 0x31A0,
    end: 0x31BF,
};

#[allow(unused)]
pub const CJK_STROKES: UnicodeBlock = UnicodeBlock {
    name: "CJK Strokes",
    start: 0x31C0,
    end: 0x31EF,
};

#[allow(unused)]
pub const KATAKANA_PHONETIC_EXTENSIONS: UnicodeBlock = UnicodeBlock {
    name: "Katakana Phonetic Extensions",
    start: 0x31F0,
    end: 0x31FF,
};

#[allow(unused)]
pub const ENCLOSED_CJK_LETTERS_AND_MONTHS: UnicodeBlock = UnicodeBlock {
    name: "Enclosed CJK Letters and Months",
    start: 0x3200,
    end: 0x32FF,
};

#[allow(unused)]
pub const CJK_COMPATIBILITY: UnicodeBlock = UnicodeBlock {
    name: "CJK Compatibility",
    start: 0x3300,
    end: 0x33FF,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension A",
    start: 0x3400,
    end: 0x4DBF,
};

#[allow(unused)]
pub const YIJING_HEXAGRAM_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Yijing Hexagram Symbols",
    start: 0x4DC0,
    end: 0x4DFF,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs",
    start: 0x4E00,
    end: 0x9FFF,
};


#[allow(unused)]
pub const YI_SYLLABLES: UnicodeBlock = UnicodeBlock {
    name: "Yi Syllables",
    start: 0xA000,
    end: 0xA48F,
};

#[allow(unused)]
pub const YI_RADICALS: UnicodeBlock = UnicodeBlock {
    name: "Yi Radicals",
    start: 0xA490,
    end: 0xA4CF,
};

#[allow(unused)]
pub const LISU: UnicodeBlock = UnicodeBlock {
    name: "Lisu",
    start: 0xA4D0,
    end: 0xA4FF,
};

#[allow(unused)]
pub const VAI: UnicodeBlock = UnicodeBlock {
    name: "Vai",
    start: 0xA500,
    end: 0xA63F,
};

#[allow(unused)]
pub const CYRILLIC_EXTENDED_B: UnicodeBlock = UnicodeBlock {
    name: "Cyrillic Extended-B",
    start: 0xA640,
    end: 0xA69F,
};

#[allow(unused)]
pub const BAMUM: UnicodeBlock = UnicodeBlock {
    name: "Bamum",
    start: 0xA6A0,
    end: 0xA6FF,
};

#[allow(unused)]
pub const MODIFIER_TONE_LETTERS: UnicodeBlock = UnicodeBlock {
    name: "Modifier Tone Letters",
    start: 0xA700,
    end: 0xA71F,
};

#[allow(unused)]
pub const LATIN_EXTENDED_D: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended-D",
    start: 0xA720,
    end: 0xA7FF,
};

#[allow(unused)]
pub const SYLOTI_NAGRI: UnicodeBlock = UnicodeBlock {
    name: "Syloti Nagri",
    start: 0xA800,
    end: 0xA82F,
};

#[allow(unused)]
pub const COMMON_INDIC_NUMBER_FORMS: UnicodeBlock = UnicodeBlock {
    name: "Common Indic Number Forms",
    start: 0xA830,
    end: 0xA83F,
};

#[allow(unused)]
pub const PHAGS_PA: UnicodeBlock = UnicodeBlock {
    name: "Phags-pa",
    start: 0xA840,
    end: 0xA87F,
};

#[allow(unused)]
pub const SAURASHTRA: UnicodeBlock = UnicodeBlock {
    name: "Saurashtra",
    start: 0xA880,
    end: 0xA8DF,
};

#[allow(unused)]
pub const DEVANAGARI_EXTENDED: UnicodeBlock = UnicodeBlock {
    name: "Devanagari Extended",
    start: 0xA8E0,
    end: 0xA8FF,
};

#[allow(unused)]
pub const KAYAH_LI: UnicodeBlock = UnicodeBlock {
    name: "Kayah Li",
    start: 0xA900,
    end: 0xA92F,
};

#[allow(unused)]
pub const REJANG: UnicodeBlock = UnicodeBlock {
    name: "Rejang",
    start: 0xA930,
    end: 0xA95F,
};

#[allow(unused)]
pub const HANGUL_JAMO_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Hangul Jamo Extended-A",
    start: 0xA960,
    end: 0xA97F,
};

#[allow(unused)]
pub const JAVANESE: UnicodeBlock = UnicodeBlock {
    name: "Javanese",
    start: 0xA980,
    end: 0xA9DF,
};

#[allow(unused)]
pub const MYANMAR_EXTENDED_B: UnicodeBlock = UnicodeBlock {
    name: "Myanmar Extended-B",
    start: 0xA9E0,
    end: 0xA9FF,
};

#[allow(unused)]
pub const CHAM: UnicodeBlock = UnicodeBlock {
    name: "Cham",
    start: 0xAA00,
    end: 0xAA5F,
};

#[allow(unused)]
pub const MYANMAR_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Myanmar Extended-A",
    start: 0xAA60,
    end: 0xAA7F,
};

#[allow(unused)]
pub const TAI_VIET: UnicodeBlock = UnicodeBlock {
    name: "Tai Viet",
    start: 0xAA80,
    end: 0xAADF,
};

#[allow(unused)]
pub const MEETEI_MAYEK_EXTENSIONS: UnicodeBlock = UnicodeBlock {
    name: "Meetei Mayek Extensions",
    start: 0xAAE0,
    end: 0xAAFF,
};

#[allow(unused)]
pub const ETHIOPIC_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Ethiopic Extended-A",
    start: 0xAB00,
    end: 0xAB2F,
};

#[allow(unused)]
pub const LATIN_EXTENDED_E: UnicodeBlock = UnicodeBlock {
    name: "Latin Extended-E",
    start: 0xAB30,
    end: 0xAB6F,
};

#[allow(unused)]
pub const CHEROKEE_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Cherokee Supplement",
    start: 0xAB70,
    end: 0xABBF,
};

#[allow(unused)]
pub const MEETEI_MAYEK: UnicodeBlock = UnicodeBlock {
    name: "Meetei Mayek",
    start: 0xABC0,
    end: 0xABFF,
};

#[allow(unused)]
pub const HANGUL_SYLLABLES: UnicodeBlock = UnicodeBlock {
    name: "Hangul Syllables",
    start: 0xAC00,
    end: 0xD7AF,
};

#[allow(unused)]
pub const HANGUL_JAMO_EXTENDED_B: UnicodeBlock = UnicodeBlock {
    name: "Hangul Jamo Extended-B",
    start: 0xD7B0,
    end: 0xD7FF,
};

#[allow(unused)]
pub const HIGH_SURROGATES: UnicodeBlock = UnicodeBlock {
    name: "High Surrogates",
    start: 0xD800,
    end: 0xDB7F,
};

#[allow(unused)]
pub const HIGH_PRIVATE_USE_SURROGATES: UnicodeBlock = UnicodeBlock {
    name: "High Private Use Surrogates",
    start: 0xDB80,
    end: 0xDBFF,
};

#[allow(unused)]
pub const LOW_SURROGATES: UnicodeBlock = UnicodeBlock {
    name: "Low Surrogates",
    start: 0xDC00,
    end: 0xDFFF,
};

#[allow(unused)]
pub const PRIVATE_USE_AREA: UnicodeBlock = UnicodeBlock {
    name: "Private Use Area",
    start: 0xE000,
    end: 0xF8FF,
};

#[allow(unused)]
pub const CJK_COMPATIBILITY_IDEOGRAPHS: UnicodeBlock = UnicodeBlock {
    name: "CJK Compatibility Ideographs",
    start: 0xF900,
    end: 0xFAFF,
};

#[allow(unused)]
pub const ALPHABETIC_PRESENTATION_FORMS: UnicodeBlock = UnicodeBlock {
    name: "Alphabetic Presentation Forms",
    start: 0xFB00,
    end: 0xFB4F,
};

#[allow(unused)]
pub const ARABIC_PRESENTATION_FORMS_A: UnicodeBlock = UnicodeBlock {
    name: "Arabic Presentation Forms-A",
    start: 0xFB50,
    end: 0xFDFF,
};

#[allow(unused)]
pub const VARIATION_SELECTORS: UnicodeBlock = UnicodeBlock {
    name: "Variation Selectors",
    start: 0xFE00,
    end: 0xFE0F,
};

#[allow(unused)]
pub const VERTICAL_FORMS: UnicodeBlock = UnicodeBlock {
    name: "Vertical Forms",
    start: 0xFE10,
    end: 0xFE1F,
};

#[allow(unused)]
pub const COMBINING_HALF_MARKS: UnicodeBlock = UnicodeBlock {
    name: "Combining Half Marks",
    start: 0xFE20,
    end: 0xFE2F,
};

#[allow(unused)]
pub const CJK_COMPATIBILITY_FORMS: UnicodeBlock = UnicodeBlock {
    name: "CJK Compatibility Forms",
    start: 0xFE30,
    end: 0xFE4F,
};

#[allow(unused)]
pub const SMALL_FORM_VARIANTS: UnicodeBlock = UnicodeBlock {
    name: "Small Form Variants",
    start: 0xFE50,
    end: 0xFE6F,
};

#[allow(unused)]
pub const ARABIC_PRESENTATION_FORMS_B: UnicodeBlock = UnicodeBlock {
    name: "Arabic Presentation Forms-B",
    start: 0xFE70,
    end: 0xFEFF,
};

#[allow(unused)]
pub const HALFWIDTH_AND_FULLWIDTH_FORMS: UnicodeBlock = UnicodeBlock {
    name: "Halfwidth and Fullwidth Forms",
    start: 0xFF00,
    end: 0xFFEF,
};

#[allow(unused)]
pub const SPECIALS: UnicodeBlock = UnicodeBlock {
    name: "Specials",
    start: 0xFFF0,
    end: 0xFFFF,
};

#[allow(unused)]
pub const LINEAR_B_SYLLABARY: UnicodeBlock = UnicodeBlock {
    name: "Linear B Syllabary",
    start: 0x10000,
    end: 0x1007F,
};

#[allow(unused)]
pub const LINEAR_B_IDEOGRAMS: UnicodeBlock = UnicodeBlock {
    name: "Linear B Ideograms",
    start: 0x10080,
    end: 0x100FF,
};

#[allow(unused)]
pub const AEGEAN_NUMBERS: UnicodeBlock = UnicodeBlock {
    name: "Aegean Numbers",
    start: 0x10100,
    end: 0x1013F,
};

#[allow(unused)]
pub const ANCIENT_GREEK_NUMBERS: UnicodeBlock = UnicodeBlock {
    name: "Ancient Greek Numbers",
    start: 0x10140,
    end: 0x1018F,
};

#[allow(unused)]
pub const ANCIENT_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Ancient Symbols",
    start: 0x10190,
    end: 0x101CF,
};

#[allow(unused)]
pub const PHAISTOS_DISC: UnicodeBlock = UnicodeBlock {
    name: "Phaistos Disc",
    start: 0x101D0,
    end: 0x101FF,
};

#[allow(unused)]
pub const LYCIAN: UnicodeBlock = UnicodeBlock {
    name: "Lycian",
    start: 0x10280,
    end: 0x1029F,
};

#[allow(unused)]
pub const CARIAN: UnicodeBlock = UnicodeBlock {
    name: "Carian",
    start: 0x102A0,
    end: 0x102DF,
};

#[allow(unused)]
pub const COPTIC_EPACT_NUMBERS: UnicodeBlock = UnicodeBlock {
    name: "Coptic Epact Numbers",
    start: 0x102E0,
    end: 0x102FF,
};

#[allow(unused)]
pub const OLD_ITALIC: UnicodeBlock = UnicodeBlock {
    name: "Old Italic",
    start: 0x10300,
    end: 0x1032F,
};

#[allow(unused)]
pub const GOTHIC: UnicodeBlock = UnicodeBlock {
    name: "Gothic",
    start: 0x10330,
    end: 0x1034F,
};

#[allow(unused)]
pub const OLD_PERMIC: UnicodeBlock = UnicodeBlock {
    name: "Old Permic",
    start: 0x10350,
    end: 0x1037F,
};

#[allow(unused)]
pub const UGARITIC: UnicodeBlock = UnicodeBlock {
    name: "Ugaritic",
    start: 0x10380,
    end: 0x1039F,
};

#[allow(unused)]
pub const OLD_PERSIAN: UnicodeBlock = UnicodeBlock {
    name: "Old Persian",
    start: 0x103A0,
    end: 0x103DF,
};

#[allow(unused)]
pub const DESERET: UnicodeBlock = UnicodeBlock {
    name: "Deseret",
    start: 0x10400,
    end: 0x1044F,
};

#[allow(unused)]
pub const SHAVIAN: UnicodeBlock = UnicodeBlock {
    name: "Shavian",
    start: 0x10450,
    end: 0x1047F,
};

#[allow(unused)]
pub const OSMANYA: UnicodeBlock = UnicodeBlock {
    name: "Osmanya",
    start: 0x10480,
    end: 0x104AF,
};

#[allow(unused)]
pub const OSAGE: UnicodeBlock = UnicodeBlock {
    name: "Osage",
    start: 0x104B0,
    end: 0x104FF,
};

#[allow(unused)]
pub const ELBASAN: UnicodeBlock = UnicodeBlock {
    name: "Elbasan",
    start: 0x10500,
    end: 0x1052F,
};

#[allow(unused)]
pub const CAUCASIAN_ALBANIAN: UnicodeBlock = UnicodeBlock {
    name: "Caucasian Albanian",
    start: 0x10530,
    end: 0x1056F,
};

#[allow(unused)]
pub const LINEAR_A: UnicodeBlock = UnicodeBlock {
    name: "Linear A",
    start: 0x10600,
    end: 0x1077F,
};

#[allow(unused)]
pub const CYPRIOT_SYLLABARY: UnicodeBlock = UnicodeBlock {
    name: "Cypriot Syllabary",
    start: 0x10800,
    end: 0x1083F,
};

#[allow(unused)]
pub const IMPERIAL_ARAMAIC: UnicodeBlock = UnicodeBlock {
    name: "Imperial Aramaic",
    start: 0x10840,
    end: 0x1085F,
};

#[allow(unused)]
pub const PALMYRENE: UnicodeBlock = UnicodeBlock {
    name: "Palmyrene",
    start: 0x10860,
    end: 0x1087F,
};

#[allow(unused)]
pub const NABATAEAN: UnicodeBlock = UnicodeBlock {
    name: "Nabataean",
    start: 0x10880,
    end: 0x108AF,
};

#[allow(unused)]
pub const HATRAN: UnicodeBlock = UnicodeBlock {
    name: "Hatran",
    start: 0x108E0,
    end: 0x108FF,
};

#[allow(unused)]
pub const PHOENICIAN: UnicodeBlock = UnicodeBlock {
    name: "Phoenician",
    start: 0x10900,
    end: 0x1091F,
};

#[allow(unused)]
pub const LYDIAN: UnicodeBlock = UnicodeBlock {
    name: "Lydian",
    start: 0x10920,
    end: 0x1093F,
};

#[allow(unused)]
pub const MEROITIC_HIEROGLYPHS: UnicodeBlock = UnicodeBlock {
    name: "Meroitic Hieroglyphs",
    start: 0x10980,
    end: 0x1099F,
};

#[allow(unused)]
pub const MEROITIC_CURSIVE: UnicodeBlock = UnicodeBlock {
    name: "Meroitic Cursive",
    start: 0x109A0,
    end: 0x109FF,
};

#[allow(unused)]
pub const KHAROSHTHI: UnicodeBlock = UnicodeBlock {
    name: "Kharoshthi",
    start: 0x10A00,
    end: 0x10A5F,
};

#[allow(unused)]
pub const OLD_SOUTH_ARABIAN: UnicodeBlock = UnicodeBlock {
    name: "Old South Arabian",
    start: 0x10A60,
    end: 0x10A7F,
};

#[allow(unused)]
pub const OLD_NORTH_ARABIAN: UnicodeBlock = UnicodeBlock {
    name: "Old North Arabian",
    start: 0x10A80,
    end: 0x10A9F,
};

#[allow(unused)]
pub const MANICHAEAN: UnicodeBlock = UnicodeBlock {
    name: "Manichaean",
    start: 0x10AC0,
    end: 0x10AFF,
};

#[allow(unused)]
pub const AVESTAN: UnicodeBlock = UnicodeBlock {
    name: "Avestan",
    start: 0x10B00,
    end: 0x10B3F,
};

#[allow(unused)]
pub const INSCRIPTIONAL_PARTHIAN: UnicodeBlock = UnicodeBlock {
    name: "Inscriptional Parthian",
    start: 0x10B40,
    end: 0x10B5F,
};

#[allow(unused)]
pub const INSCRIPTIONAL_PAHLAVI: UnicodeBlock = UnicodeBlock {
    name: "Inscriptional Pahlavi",
    start: 0x10B60,
    end: 0x10B7F,
};

#[allow(unused)]
pub const PSALTER_PAHLAVI: UnicodeBlock = UnicodeBlock {
    name: "Psalter Pahlavi",
    start: 0x10B80,
    end: 0x10BAF,
};

#[allow(unused)]
pub const OLD_TURKIC: UnicodeBlock = UnicodeBlock {
    name: "Old Turkic",
    start: 0x10C00,
    end: 0x10C4F,
};

#[allow(unused)]
pub const OLD_HUNGARIAN: UnicodeBlock = UnicodeBlock {
    name: "Old Hungarian",
    start: 0x10C80,
    end: 0x10CFF,
};


#[allow(unused)]
pub const HANIFI_ROHINGYA: UnicodeBlock = UnicodeBlock {
    name: "Hanifi Rohingya",
    start: 0x10D00,
    end: 0x10D3F,
};

#[allow(unused)]
pub const RUMI_NUMERAL_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Rumi Numeral Symbols",
    start: 0x10E60,
    end: 0x10E7F,
};

#[allow(unused)]
pub const YEZIDI: UnicodeBlock = UnicodeBlock {
    name: "Yezidi",
    start: 0x10E80,
    end: 0x10EBF,
};

#[allow(unused)]
pub const OLD_SOGDIAN: UnicodeBlock = UnicodeBlock {
    name: "Old Sogdian",
    start: 0x10F00,
    end: 0x10F2F,
};

#[allow(unused)]
pub const SOGDIAN: UnicodeBlock = UnicodeBlock {
    name: "Sogdian",
    start: 0x10F30,
    end: 0x10F6F,
};

#[allow(unused)]
pub const CHORASMIAN: UnicodeBlock = UnicodeBlock {
    name: "Chorasmian",
    start: 0x10FB0,
    end: 0x10FDF,
};

#[allow(unused)]
pub const ELYMAIC: UnicodeBlock = UnicodeBlock {
    name: "Elymaic",
    start: 0x10FE0,
    end: 0x10FFF,
};

#[allow(unused)]
pub const BRAHMI: UnicodeBlock = UnicodeBlock {
    name: "Brahmi",
    start: 0x11000,
    end: 0x1107F,
};

#[allow(unused)]
pub const KAITHI: UnicodeBlock = UnicodeBlock {
    name: "Kaithi",
    start: 0x11080,
    end: 0x110CF,
};

#[allow(unused)]
pub const SORA_SOMPENG: UnicodeBlock = UnicodeBlock {
    name: "Sora Sompeng",
    start: 0x110D0,
    end: 0x110FF,
};

#[allow(unused)]
pub const CHAKMA: UnicodeBlock = UnicodeBlock {
    name: "Chakma",
    start: 0x11100,
    end: 0x1114F,
};

#[allow(unused)]
pub const MAHAJANI: UnicodeBlock = UnicodeBlock {
    name: "Mahajani",
    start: 0x11150,
    end: 0x1117F,
};

#[allow(unused)]
pub const SHARADA: UnicodeBlock = UnicodeBlock {
    name: "Sharada",
    start: 0x11180,
    end: 0x111DF,
};

#[allow(unused)]
pub const SINHALA_ARCHAIC_NUMBERS: UnicodeBlock = UnicodeBlock {
    name: "Sinhala Archaic Numbers",
    start: 0x111E0,
    end: 0x111FF,
};

#[allow(unused)]
pub const KHOJKI: UnicodeBlock = UnicodeBlock {
    name: "Khojki",
    start: 0x11200,
    end: 0x1124F,
};

#[allow(unused)]
pub const MULTANI: UnicodeBlock = UnicodeBlock {
    name: "Multani",
    start: 0x11280,
    end: 0x112AF,
};

#[allow(unused)]
pub const KHUDAWADI: UnicodeBlock = UnicodeBlock {
    name: "Khudawadi",
    start: 0x112B0,
    end: 0x112FF,
};

#[allow(unused)]
pub const GRANTHA: UnicodeBlock = UnicodeBlock {
    name: "Grantha",
    start: 0x11300,
    end: 0x1137F,
};

#[allow(unused)]
pub const NEWA: UnicodeBlock = UnicodeBlock {
    name: "Newa",
    start: 0x11400,
    end: 0x1147F,
};

#[allow(unused)]
pub const TIRHUTA: UnicodeBlock = UnicodeBlock {
    name: "Tirhuta",
    start: 0x11480,
    end: 0x114DF,
};

#[allow(unused)]
pub const SIDDHAM: UnicodeBlock = UnicodeBlock {
    name: "Siddham",
    start: 0x11580,
    end: 0x115FF,
};

#[allow(unused)]
pub const MODI: UnicodeBlock = UnicodeBlock {
    name: "Modi",
    start: 0x11600,
    end: 0x1165F,
};

#[allow(unused)]
pub const MONGOLIAN_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Mongolian Supplement",
    start: 0x11660,
    end: 0x1167F,
};

#[allow(unused)]
pub const TAKRI: UnicodeBlock = UnicodeBlock {
    name: "Takri",
    start: 0x11680,
    end: 0x116CF,
};

#[allow(unused)]
pub const AHOM: UnicodeBlock = UnicodeBlock {
    name: "Ahom",
    start: 0x11700,
    end: 0x1173F,
};

#[allow(unused)]
pub const DOGRA: UnicodeBlock = UnicodeBlock {
    name: "Dogra",
    start: 0x11800,
    end: 0x1184F,
};

#[allow(unused)]
pub const WARANG_CITI: UnicodeBlock = UnicodeBlock {
    name: "Warang Citi",
    start: 0x118A0,
    end: 0x118FF,
};

#[allow(unused)]
pub const DIVES_AKURU: UnicodeBlock = UnicodeBlock {
    name: "Dives Akuru",
    start: 0x11900,
    end: 0x1195F,
};

#[allow(unused)]
pub const NANDINAGARI: UnicodeBlock = UnicodeBlock {
    name: "Nandinagari",
    start: 0x119A0,
    end: 0x119FF,
};

#[allow(unused)]
pub const ZANABAZAR_SQUARE: UnicodeBlock = UnicodeBlock {
    name: "Zanabazar Square",
    start: 0x11A00,
    end: 0x11A4F,
};

#[allow(unused)]
pub const SOYOMBO: UnicodeBlock = UnicodeBlock {
    name: "Soyombo",
    start: 0x11A50,
    end: 0x11AAF,
};

#[allow(unused)]
pub const PAU_CIN_HAU: UnicodeBlock = UnicodeBlock {
    name: "Pau Cin Hau",
    start: 0x11AC0,
    end: 0x11AFF,
};

#[allow(unused)]
pub const BHAIKSUKI: UnicodeBlock = UnicodeBlock {
    name: "Bhaiksuki",
    start: 0x11C00,
    end: 0x11C6F,
};

#[allow(unused)]
pub const MARCHEN: UnicodeBlock = UnicodeBlock {
    name: "Marchen",
    start: 0x11C70,
    end: 0x11CBF,
};

#[allow(unused)]
pub const MASARAM_GONDI: UnicodeBlock = UnicodeBlock {
    name: "Masaram Gondi",
    start: 0x11D00,
    end: 0x11D5F,
};

#[allow(unused)]
pub const GUNJALA_GONDI: UnicodeBlock = UnicodeBlock {
    name: "Gunjala Gondi",
    start: 0x11D60,
    end: 0x11DAF,
};

#[allow(unused)]
pub const MAKASAR: UnicodeBlock = UnicodeBlock {
    name: "Makasar",
    start: 0x11EE0,
    end: 0x11EFF,
};

#[allow(unused)]
pub const LISU_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Lisu Supplement",
    start: 0x11FB0,
    end: 0x11FBF,
};

#[allow(unused)]
pub const TAMIL_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Tamil Supplement",
    start: 0x11FC0,
    end: 0x11FFF,
};

#[allow(unused)]
pub const CUNEIFORM: UnicodeBlock = UnicodeBlock {
    name: "Cuneiform",
    start: 0x12000,
    end: 0x123FF,
};

#[allow(unused)]
pub const CUNEIFORM_NUMBERS_AND_PUNCTUATION: UnicodeBlock = UnicodeBlock {
    name: "Cuneiform Numbers and Punctuation",
    start: 0x12400,
    end: 0x1247F,
};

#[allow(unused)]
pub const EARLY_DYNASTIC_CUNEIFORM: UnicodeBlock = UnicodeBlock {
    name: "Early Dynastic Cuneiform",
    start: 0x12480,
    end: 0x1254F,
};

#[allow(unused)]
pub const EGYPTIAN_HIEROGLYPHS: UnicodeBlock = UnicodeBlock {
    name: "Egyptian Hieroglyphs",
    start: 0x13000,
    end: 0x1342F,
};

#[allow(unused)]
pub const EGYPTIAN_HIEROGLYPH_FORMAT_CONTROLS: UnicodeBlock = UnicodeBlock {
    name: "Egyptian Hieroglyph Format Controls",
    start: 0x13430,
    end: 0x1343F,
};

#[allow(unused)]
pub const ANATOLIAN_HIEROGLYPHS: UnicodeBlock = UnicodeBlock {
    name: "Anatolian Hieroglyphs",
    start: 0x14400,
    end: 0x1467F,
};

#[allow(unused)]
pub const BAMUM_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Bamum Supplement",
    start: 0x16800,
    end: 0x16A3F,
};

#[allow(unused)]
pub const MRO: UnicodeBlock = UnicodeBlock {
    name: "Mro",
    start: 0x16A40,
    end: 0x16A6F,
};

#[allow(unused)]
pub const BASSA_VAH: UnicodeBlock = UnicodeBlock {
    name: "Bassa Vah",
    start: 0x16AD0,
    end: 0x16AFF,
};

#[allow(unused)]
pub const PAHAWH_HMONG: UnicodeBlock = UnicodeBlock {
    name: "Pahawh Hmong",
    start: 0x16B00,
    end: 0x16B8F,
};

#[allow(unused)]
pub const MEDEFAIDRIN: UnicodeBlock = UnicodeBlock {
    name: "Medefaidrin",
    start: 0x16E40,
    end: 0x16E9F,
};

#[allow(unused)]
pub const MIAO: UnicodeBlock = UnicodeBlock {
    name: "Miao",
    start: 0x16F00,
    end: 0x16F9F,
};

#[allow(unused)]
pub const IDEOGRAPHIC_SYMBOLS_AND_PUNCTUATION: UnicodeBlock = UnicodeBlock {
    name: "Ideographic Symbols and Punctuation",
    start: 0x16FE0,
    end: 0x16FFF,
};

#[allow(unused)]
pub const TANGUT: UnicodeBlock = UnicodeBlock {
    name: "Tangut",
    start: 0x17000,
    end: 0x187FF,
};

#[allow(unused)]
pub const TANGUT_COMPONENTS: UnicodeBlock = UnicodeBlock {
    name: "Tangut Components",
    start: 0x18800,
    end: 0x18AFF,
};

#[allow(unused)]
pub const KHITAN_SMALL_SCRIPT: UnicodeBlock = UnicodeBlock {
    name: "Khitan Small Script",
    start: 0x18B00,
    end: 0x18CFF,
};

#[allow(unused)]
pub const TANGUT_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Tangut Supplement",
    start: 0x18D00,
    end: 0x18D8F,
};

#[allow(unused)]
pub const KANA_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Kana Supplement",
    start: 0x1B000,
    end: 0x1B0FF,
};

#[allow(unused)]
pub const KANA_EXTENDED_A: UnicodeBlock = UnicodeBlock {
    name: "Kana Extended-A",
    start: 0x1B100,
    end: 0x1B12F,
};

#[allow(unused)]
pub const SMALL_KANA_EXTENSION: UnicodeBlock = UnicodeBlock {
    name: "Small Kana Extension",
    start: 0x1B130,
    end: 0x1B16F,
};

#[allow(unused)]
pub const NUSHU: UnicodeBlock = UnicodeBlock {
    name: "Nushu",
    start: 0x1B170,
    end: 0x1B2FF,
};

#[allow(unused)]
pub const DUPLOYAN: UnicodeBlock = UnicodeBlock {
    name: "Duployan",
    start: 0x1BC00,
    end: 0x1BC9F,
};

#[allow(unused)]
pub const SHORTHAND_FORMAT_CONTROLS: UnicodeBlock = UnicodeBlock {
    name: "Shorthand Format Controls",
    start: 0x1BCA0,
    end: 0x1BCAF,
};

#[allow(unused)]
pub const BYZANTINE_MUSICAL_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Byzantine Musical Symbols",
    start: 0x1D000,
    end: 0x1D0FF,
};

#[allow(unused)]
pub const MUSICAL_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Musical Symbols",
    start: 0x1D100,
    end: 0x1D1FF,
};

#[allow(unused)]
pub const ANCIENT_GREEK_MUSICAL_NOTATION: UnicodeBlock = UnicodeBlock {
    name: "Ancient Greek Musical Notation",
    start: 0x1D200,
    end: 0x1D24F,
};

#[allow(unused)]
pub const MAYAN_NUMERALS: UnicodeBlock = UnicodeBlock {
    name: "Mayan Numerals",
    start: 0x1D2E0,
    end: 0x1D2FF,
};

#[allow(unused)]
pub const TAI_XUAN_JING_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Tai Xuan Jing Symbols",
    start: 0x1D300,
    end: 0x1D35F,
};

#[allow(unused)]
pub const COUNTING_ROD_NUMERALS: UnicodeBlock = UnicodeBlock {
    name: "Counting Rod Numerals",
    start: 0x1D360,
    end: 0x1D37F,
};

#[allow(unused)]
pub const MATHEMATICAL_ALPHANUMERIC_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Mathematical Alphanumeric Symbols",
    start: 0x1D400,
    end: 0x1D7FF,
};

#[allow(unused)]
pub const SUTTON_SIGNWRITING: UnicodeBlock = UnicodeBlock {
    name: "Sutton SignWriting",
    start: 0x1D800,
    end: 0x1DAAF,
};

#[allow(unused)]
pub const EMOTICONS: UnicodeBlock = UnicodeBlock {
    name: "Emoticons",
    start: 0x1F600,
    end: 0x1F64F,
};

#[allow(unused)]
pub const TRANSPORT_AND_MAP_SYMBOLS: UnicodeBlock = UnicodeBlock {
    name: "Transport and Map Symbols",
    start: 0x1F680,
    end: 0x1F6FF,
};

#[allow(unused)]
pub const SYMBOLS_FOR_LEGACY_COMPUTING: UnicodeBlock = UnicodeBlock {
    name: "Symbols for Legacy Computing",
    start: 0x1FB00,
    end: 0x1FBFF,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension B",
    start: 0x20000,
    end: 0x2A6DF,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension C",
    start: 0x2A700,
    end: 0x2B73F,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension D",
    start: 0x2B740,
    end: 0x2B81F,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_E: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension E",
    start: 0x2B820,
    end: 0x2CEAF,
};

#[allow(unused)]
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_F: UnicodeBlock = UnicodeBlock {
    name: "CJK Unified Ideographs Extension F",
    start: 0x2CEB0,
    end: 0x2EBEF,
};

#[allow(unused)]
pub const TAGS: UnicodeBlock = UnicodeBlock {
    name: "Tags",
    start: 0xE0000,
    end: 0xE007F,
};

#[allow(unused)]
pub const VARIATION_SELECTORS_SUPPLEMENT: UnicodeBlock = UnicodeBlock {
    name: "Variation Selectors Supplement",
    start: 0xE0100,
    end: 0xE01EF,
};

#[allow(unused)]
pub const SUPPLEMENTARY_PRIVATE_USE_AREA_A: UnicodeBlock = UnicodeBlock {
    name: "Supplementary Private Use Area-A",
    start: 0xF0000,
    end: 0xFFFFF,
};

#[allow(unused)]
pub const SUPPLEMENTARY_PRIVATE_USE_AREA_B: UnicodeBlock = UnicodeBlock {
    name: "Supplementary Private Use Area-B",
    start: 0x100000,
    end: 0x10FFFF,
};



#[allow(unused)]
pub const ALL_BLOCKS: &[UnicodeBlock] = &[
    BASIC_LATIN,
    LATIN_1_SUPPLEMENT,
    LATIN_EXTENDED_A,
    LATIN_EXTENDED_B,
    IPA_EXTENSIONS,
    SPACING_MODIFIER_LETTERS,
    COMBINING_DIACRITICAL_MARKS,
    GREEK_AND_COPTIC,
    CYRILLIC,
    CYRILLIC_SUPPLEMENT,
    ARMENIAN,
    HEBREW,
    ARABIC,
    SYRIAC,
    ARABIC_SUPPLEMENT,
    THAANA,
    NKO,
    SAMARITAN,
    MANDAIC,
    SYRIAC_SUPPLEMENT,
    ARABIC_EXTENDED_A,
    DEVANAGARI,
    BENGALI,
    GURMUKHI,
    GUJARATI,
    ORIYA,
    TAMIL,
    TELUGU,
    KANNADA,
    MALAYALAM,
    SINHALA,
    THAI,
    LAO,
    TIBETAN,
    MYANMAR,
    GEORGIAN,
    HANGUL_JAMO,
    ETHIOPIC,
    ETHIOPIC_SUPPLEMENT,
    CHEROKEE,
    UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS,
    OGHAM,
    RUNIC,
    TAGALOG,
    HANUNOO,
    BUHID,
    TAGBANWA,
    KHMER,
    MONGOLIAN,
    UNIFIED_CANADIAN_ABORIGINAL_SYLLABICS_EXTENDED,
    LIMBU,
    TAI_LE,
    NEW_TAI_LUE,
    KHMER_SYMBOLS,
    BUGINESE,
    TAI_THAM,
    COMBINING_DIACRITICAL_MARKS_EXTENDED,
    BALINESE,
    SUNDANESE,
    BATAK,
    LEPCHA,
    OL_CHIKI,
    CYRILLIC_EXTENDED_C,
    GEORGIAN_EXTENDED,
    SUNDANESE_SUPPLEMENT,
    VEDIC_EXTENSIONS,
    PHONETIC_EXTENSIONS,
    PHONETIC_EXTENSIONS_SUPPLEMENT,
    COMBINING_DIACRITICAL_MARKS_SUPPLEMENT,
    LATIN_EXTENDED_ADDITIONAL,
    GREEK_EXTENDED,
    GENERAL_PUNCTUATION,
    SUPERSCRIPTS_AND_SUBSCRIPTS,
    CURRENCY_SYMBOLS,
    COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS,
    LETTERLIKE_SYMBOLS,
    NUMBER_FORMS,
    ARROWS,
    MATHEMATICAL_OPERATORS,
    MISCELLANEOUS_TECHNICAL,
    CONTROL_PICTURES,
    OPTICAL_CHARACTER_RECOGNITION,
    ENCLOSED_ALPHANUMERICS,
    BOX_DRAWING,
    BLOCK_ELEMENTS,
    GEOMETRIC_SHAPES,
    MISCELLANEOUS_SYMBOLS,
    DINGBATS,
    // (continues — you can extend if desired)
];


