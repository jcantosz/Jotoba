pub mod furigana;
pub mod guessing;
pub mod radicals;

pub use romaji;

use itertools::Itertools;
use std::{iter, ops::Range};
use utils;

const RADICALS: &[char] = &[
    '｜', 'ノ', '⺅', 'ハ', '⺉', 'マ', 'ユ', '⻌', '⺌', 'ヨ', '⺖', '⺘', '⺡', '⺨', '⺾', '⻏',
    '⻖', '⺹', '⺣', '⺭', '⻂', '⺲',
];

pub trait JapaneseExt {
    /// Returns true if self is of type ct
    fn is_of_type(&self, ct: CharType) -> bool;

    /// Get the CharType of a character
    fn get_text_type(&self) -> CharType;

    /// Returns true if self contains at least one kana character
    fn has_kana(&self) -> bool;

    /// Returns true if self is entirely written in kana
    fn is_kana(&self) -> bool;

    /// Returns true if inp is entirely written with kanji
    fn is_kanji(&self) -> bool;

    /// Returns true if inp has at least one kanji
    fn has_kanji(&self) -> bool;

    /// Returns true if inp is build with kanji and kana only
    fn is_japanese(&self) -> bool;

    /// Returns true if inp contains japanese characters
    fn has_japanese(&self) -> bool;

    /// Returns true if self is written in katakana
    fn is_katakana(&self) -> bool;

    /// Returns true if self is written in hiragana
    fn is_hiragana(&self) -> bool;

    /// Returns the amount of kanji self has
    fn kanji_count(&self) -> usize;

    /// Returns true if self is a (cjk) symbol
    fn is_symbol(&self) -> bool;

    /// Returns true if self is a (cjk) symbol
    fn has_symbol(&self) -> bool;

    fn to_hiragana(&self) -> String;

    fn has_roman_letter(&self) -> bool;

    fn is_roman_letter(&self) -> bool;

    /// Returns true if self is a small katakana letter
    fn is_small_katakana(&self) -> bool;

    /// Returns true if self is a small hiragana letter
    fn is_small_hiragana(&self) -> bool;

    /// Returns true if self is a small hiragana letter
    fn is_small_kana(&self) -> bool;

    fn is_radical(&self) -> bool;

    fn is_particle(&self) -> bool;

    fn starts_with_ct(&self, ct: CharType) -> bool;
}

impl JapaneseExt for char {
    #[inline]
    fn is_of_type(&self, ct: CharType) -> bool {
        self.get_text_type() == ct
    }

    #[inline]
    fn get_text_type(&self) -> CharType {
        if self.is_symbol() {
            CharType::Symbol
        } else if self.is_kana() {
            CharType::Kana
        } else if self.is_kanji() || self.is_roman_letter() {
            CharType::Kanji
        } else {
            CharType::Other
        }
    }

    #[inline]
    fn has_kana(&self) -> bool {
        self.is_kana()
    }

    #[inline]
    fn is_kana(&self) -> bool {
        self.is_hiragana() || self.is_katakana()
    }

    #[inline]
    fn is_kanji(&self) -> bool {
        ((*self) >= '\u{3400}' && (*self) <= '\u{4DBF}')
            || ((*self) >= '\u{4E00}' && (*self) <= '\u{9FFF}')
            || ((*self) >= '\u{F900}' && (*self) <= '\u{FAFF}')
            || ((*self) >= '\u{FF10}' && (*self) <= '\u{FF19}')
            || ((*self) >= '\u{20000}' && (*self) <= '\u{2A6DF}')
            || (*self) == '\u{29E8A}'
    }

    #[inline]
    fn has_kanji(&self) -> bool {
        self.is_kanji()
    }

    #[inline]
    fn is_japanese(&self) -> bool {
        self.is_kana() || self.is_kanji() || self.is_symbol() || self.is_roman_letter()
    }

    #[inline]
    fn has_japanese(&self) -> bool {
        self.is_japanese()
    }

    #[inline]
    fn is_katakana(&self) -> bool {
        (*self) >= '\u{30A0}' && (*self) <= '\u{30FF}'
    }

    #[inline]
    fn is_hiragana(&self) -> bool {
        (*self) >= '\u{3040}' && (*self) <= '\u{309F}'
    }

    #[inline]
    fn kanji_count(&self) -> usize {
        if self.is_kanji() {
            1
        } else {
            0
        }
    }

    #[inline]
    fn is_symbol(&self) -> bool {
        // https://www.htmlsymbols.xyz/ascii-symbols/fullwidth-ascii-variants
        ((*self) >= '\u{3000}' && (*self) <= '\u{303F}')
            || ((*self) >= '\u{0370}' && (*self) <= '\u{03FF}')
            || ((*self) >= '\u{25A0}' && (*self) <= '\u{25FF}')
            || ((*self) >= '\u{FF01}' && (*self) <= '\u{FF0F}')
            || ((*self) >= '\u{FF1A}' && (*self) <= '\u{FF20}')
            || ((*self) >= '\u{FF3B}' && (*self) <= '\u{FF40}')
            || ((*self) >= '\u{FF5B}' && (*self) <= '\u{FF5E}')
            || (*self) == '\u{002D}'
            || (*self) == '\u{3005}'
            || (*self) == '\u{00D7}'
    }

    #[inline]
    fn has_symbol(&self) -> bool {
        self.is_symbol()
    }

    #[inline]
    fn to_hiragana(&self) -> String {
        romaji::RomajiExt::to_hiragana(self.to_string().as_str())
    }

    #[inline]
    fn has_roman_letter(&self) -> bool {
        self.is_roman_letter()
    }

    #[inline]
    fn is_roman_letter(&self) -> bool {
        (*self) >= '\u{FF01}' && (*self) <= '\u{FF5A}'
            || ((*self) >= '\u{2000}' && (*self) <= '\u{206F}')
            || ((*self) >= '\u{20000}' && (*self) <= '\u{2A6DF}')
            || (*self) == '\u{2010}'
            || (*self) == '\u{2212}'
    }

    #[inline]
    fn is_small_katakana(&self) -> bool {
        *self == '\u{30E3}' || *self == '\u{30E5}' || *self == '\u{30E7}'
    }

    #[inline]
    fn is_small_hiragana(&self) -> bool {
        *self == '\u{3083}' || *self == '\u{3085}' || *self == '\u{3087}'
    }

    #[inline]
    fn is_small_kana(&self) -> bool {
        self.is_small_katakana() || self.is_small_hiragana()
    }

    #[inline]
    fn is_radical(&self) -> bool {
        self.is_kanji() || RADICALS.iter().any(|i| *i == *self)
    }

    #[inline]
    fn is_particle(&self) -> bool {
        matches!(
            self,
            'を' | 'の' | 'に' | 'と' | 'が' | 'か' | 'は' | 'も' | 'で' | 'へ' | 'や'
        )
    }

    #[inline]
    fn starts_with_ct(&self, ct: CharType) -> bool {
        self.is_of_type(ct)
    }
}

/// Convert Wide-alphanumeric into normal ASCII  [Ａ -> A]
#[inline]
pub fn to_halfwidth(s: &str) -> String {
    shift_unicode(s, 0xff01..0xff5f, |x| x - 0xfee0)
}

/// Convert Wide-alphanumeric into normal ASCII  [Ａ -> A]
#[inline]
pub fn to_fullwidth(s: &str) -> String {
    shift_unicode(s, 0x0021..0x007f, |x| x + 0xfee0)
}

#[inline]
fn shift_unicode<D, S: AsRef<str>>(s: S, range: Range<u32>, conv: D) -> String
where
    D: Fn(u32) -> u32,
{
    s.as_ref()
        .chars()
        .map(|c| map_char(c, range.clone(), &conv))
        .collect()
}

#[inline]
fn map_char<D>(c: char, range: Range<u32>, conv: D) -> char
where
    D: FnOnce(u32) -> u32,
{
    let n = c as u32;
    if range.contains(&n) {
        char::from_u32(conv(n)).unwrap()
    } else {
        c
    }
}

impl JapaneseExt for str {
    #[inline]
    fn is_of_type(&self, ct: CharType) -> bool {
        self.get_text_type() == ct
    }

    #[inline]
    fn get_text_type(&self) -> CharType {
        if self.is_kanji() || self.is_symbol() {
            CharType::Kanji
        } else if self.is_kana() {
            CharType::Kana
        } else {
            CharType::Other
        }
    }

    #[inline]
    fn has_kana(&self) -> bool {
        self.chars().any(|s| s.is_kana())
    }

    #[inline]
    fn is_kana(&self) -> bool {
        self.chars().all(|s| s.is_kana())
    }

    #[inline]
    fn is_kanji(&self) -> bool {
        self.chars().all(|s| s.is_kanji())
    }

    #[inline]
    fn has_kanji(&self) -> bool {
        self.chars().any(|s| s.is_kanji())
    }

    #[inline]
    fn is_japanese(&self) -> bool {
        let mut buf = [0; 16];
        !self.chars().any(|c| {
            let s = c.encode_utf8(&mut buf);
            !s.is_kana() && !s.is_kanji() && !s.is_symbol() && !s.is_roman_letter()
        })
    }

    #[inline]
    fn has_japanese(&self) -> bool {
        let mut buf = [0; 16];
        self.chars().any(|c| {
            let s = c.encode_utf8(&mut buf);
            s.is_kana() || s.is_kanji() || s.is_symbol() || s.is_roman_letter()
        })
    }

    #[inline]
    fn is_katakana(&self) -> bool {
        self.chars().all(|s| s.is_katakana())
    }

    #[inline]
    fn is_hiragana(&self) -> bool {
        self.chars().all(|s| s.is_hiragana())
    }

    #[inline]
    fn kanji_count(&self) -> usize {
        self.chars().filter(|i| i.is_kanji()).count()
    }

    #[inline]
    fn is_symbol(&self) -> bool {
        self.chars().all(|s| s.is_symbol())
    }

    #[inline]
    fn has_symbol(&self) -> bool {
        self.chars().any(|s| s.is_symbol())
    }

    #[inline]
    fn to_hiragana(&self) -> String {
        romaji::RomajiExt::to_hiragana(self)
    }

    #[inline]
    fn has_roman_letter(&self) -> bool {
        self.chars().any(|s| s.is_roman_letter())
    }

    #[inline]
    fn is_roman_letter(&self) -> bool {
        self.chars().all(|s| s.is_roman_letter())
    }

    #[inline]

    fn is_small_katakana(&self) -> bool {
        self.chars().all(|s| s.is_small_katakana())
    }
    #[inline]
    fn is_small_hiragana(&self) -> bool {
        self.chars().all(|s| s.is_small_hiragana())
    }

    #[inline]
    fn is_small_kana(&self) -> bool {
        self.is_small_katakana() || self.is_small_hiragana()
    }

    #[inline]
    fn is_radical(&self) -> bool {
        self.chars().all(|s| s.is_radical())
    }

    #[inline]
    fn is_particle(&self) -> bool {
        self.chars().all(|s| s.is_particle())
    }

    #[inline]
    fn starts_with_ct(&self, ct: CharType) -> bool {
        let first = self.chars().nth(0);
        match first {
            Some(s) => s.is_of_type(ct),
            None => false,
        }
    }
}

/// Returns `true` if `romaji` is a prefix of `hira` where romaji is romaji text and `hira` is text written in hiragana
#[inline]
pub fn romaji_prefix(romaji: &str, hira: &str) -> bool {
    romaji::RomajiExt::to_romaji(hira)
        .to_lowercase()
        .starts_with(&romaji.to_lowercase())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CharType {
    Kana,
    Kanji,
    Symbol,
    Other,
}

/// Return all words of chartype ct
pub fn all_words_with_ct(inp: &str, ct: CharType) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    let mut curr = String::new();
    let mut iter = inp.chars();
    while let Some(c) = iter.next() {
        if c.is_of_type(ct) {
            curr.push(c);
            continue;
        } else {
            if !curr.is_empty() {
                all.push(curr.clone());
            }
            curr.clear();
            iter.take_while_ref(|i| !i.is_of_type(ct)).count();
        }
    }
    if !curr.is_empty() {
        all.push(curr);
    }
    all
}

/// Returns an iterator over all kanji / kana pairs
pub fn text_parts<'a>(kanji: &'a str) -> impl Iterator<Item = &'a str> {
    let mut kanji_indices = kanji.char_indices().peekable();

    iter::from_fn(move || {
        let (curr_c_pos, curr_char) = kanji_indices.next()?;
        while let Some((pos, c)) = kanji_indices.peek() {
            if curr_char.get_text_type() != c.get_text_type() {
                return Some(&kanji[curr_c_pos..*pos]);
            }
            kanji_indices.next();
        }

        Some(&kanji[curr_c_pos..])
    })
}

/// Returns an iterator over kanji occurences having the reading [`reading`]
pub fn has_reading<'a>(
    furigana: &'a str,
    kanji_literal: char,
    reading: &'a str,
) -> impl Iterator<Item = bool> + 'a {
    furigana::parse::from_str(furigana)
        .filter_map(move |i| i.kanji.and_then(|k| k.contains(kanji_literal).then(|| i)))
        .map(move |i| match_reading(i.kanji.unwrap(), i.kana, kanji_literal, reading))
}

/// Returns true if [`k_literal`] has the reading [`reading`] within a (possible) compound and its
/// kana mapping
fn match_reading(comp: &str, comp_reading: &str, k_literal: char, reading: &str) -> bool {
    if utils::char_eq_str(k_literal, comp) {
        return comp_reading == reading;
    }

    let compound_len = comp.chars().count();
    let reading_len = reading.chars().count();
    let comp_reading_len = comp_reading.chars().count();
    if compound_len - 1 > comp_reading_len - reading_len {
        // Kanji mapping is impossible. [`reading`] needs more kanji syllables which are available for
        // the given kanji.
        return false;
    }

    if comp.ends_with(k_literal) {
        return comp_reading.ends_with(reading);
    }

    if comp.starts_with(k_literal) {
        return comp_reading.starts_with(reading);
    }

    // Impossible to check against other cases
    false
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{text_parts, to_fullwidth, to_halfwidth, JapaneseExt};

    #[test_case("音",true; "音")]
    #[test_case("あ",false; "Kana 'a'")]
    #[test_case("、",false; "Special japanese char")]
    fn is_kanji(inp: &str, expcected: bool) {
        assert_eq!(inp.is_kanji(), expcected);
    }

    #[test_case("、",true; "Symbol")]
    #[test_case("音",false; "Kanji")]
    #[test_case("あ",false; "Kana")]
    fn is_symbol(inp: &str, expcected: bool) {
        assert_eq!(inp.is_symbol(), expcected);
    }

    #[test_case("これは漢字で書いたテキストです", &["これは", "漢字", "で", "書", "いたテキストです"]; "Simple")]
    #[test_case("このテキストはかなだけでかいた", &["このテキストはかなだけでかいた"]; "Kana only")]
    #[test_case("朝に道を聞かば、夕べに死すとも可なり", &["朝", "に", "道", "を", "聞", "かば","、", "夕", "べに", "死", "すとも", "可", "なり"]; "Special char")]
    fn test_text_parts(inp: &str, exp: &[&str]) {
        let pairs: Vec<_> = text_parts(inp).collect();
        let exp: Vec<_> = exp.iter().map(|i| i.to_string()).collect();
        assert_eq!(pairs, exp);
    }

    #[test_case("1234","１２３４"; "To fullwidth")]
    fn test_to_fullwidth(inp: &str, exp: &str) {
        assert_eq!(to_fullwidth(inp).as_str(), exp);
    }

    #[test_case("１２３４","1234"; "To halfwidth")]
    #[test_case("５日","5日"; "With kanji")]
    fn test_to_halfwidth(inp: &str, exp: &str) {
        assert_eq!(to_halfwidth(inp).as_str(), exp);
    }
}
