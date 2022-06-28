use crate::jotoba::languages::Language;

use super::{
    dialect::Dialect,
    field::Field,
    foreign_language::ForeignLanguage,
    gtype::GType,
    misc::Misc,
    part_of_speech::{PartOfSpeech, PosSimple},
    Word,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "jotoba_intern")]
use localization::{language::Language as LocLanguage, traits::Translatable, TranslationDict};

/// A single sense for a word. Represents one language,
/// one misc item and 1..n glosses
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Hash)]
pub struct Sense {
    pub id: u8,
    pub misc: Option<Misc>,
    pub field: Option<Field>,
    pub dialect: Option<Dialect>,
    pub glosses: Vec<Gloss>,
    pub xref: Option<String>,
    pub antonym: Option<String>,
    pub information: Option<String>,
    pub part_of_speech: Vec<PartOfSpeech>,
    pub language: Language,
    pub example_sentence: Option<u32>,
    pub gairaigo: Option<Gairaigo>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, Hash)]
pub struct Gairaigo {
    pub language: ForeignLanguage,
    pub fully_derived: bool,
    pub original: String,
}

impl Eq for Sense {}

/// A gloss value represents one word in the
/// translated language.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Hash)]
pub struct Gloss {
    pub id: u8,
    pub gloss: String,
    pub g_type: Option<GType>,
}

/// Converts sense and gloss id to a single u16
#[inline]
pub fn to_unique_id(sense_id: u8, gloss_id: u8) -> u16 {
    (sense_id as u16) << 8 | gloss_id as u16
}

/// Converts u16 to seq and gloss id
#[inline]
pub fn from_unique_id(id: u16) -> (u8, u8) {
    let gloss_id = id as u8;
    let sense_id = (id >> 8) as u8;
    (sense_id, gloss_id)
}

impl Sense {
    /// Get all pos_simple of a sense
    pub fn get_pos_simple(&self) -> Vec<PosSimple> {
        let mut pos_simple = self
            .part_of_speech
            .iter()
            .map(|i| i.to_pos_simple())
            .flatten()
            .collect::<Vec<_>>();

        pos_simple.sort_unstable();
        pos_simple.dedup();
        pos_simple
    }

    #[inline]
    pub fn gloss_by_id(&self, id: u8) -> Option<&Gloss> {
        self.glosses.iter().find(|i| i.id == id)
    }
}

// Jotoba intern only features
#[cfg(feature = "jotoba_intern")]
impl Sense {
    /// Get a senses tags prettified
    #[inline]
    pub fn get_glosses(&self) -> String {
        use itertools::Itertools;
        self.glosses.iter().map(|i| i.gloss.clone()).join("; ")
    }

    /// Returns an `xref` of the sense if available
    #[inline]
    pub fn get_xref(&self) -> Option<&str> {
        self.xref.as_ref().and_then(|xref| xref.split('・').next())
    }

    /// Returns an `antonym` of the sense if available
    #[inline]
    pub fn get_antonym(&self) -> Option<&str> {
        self.antonym
            .as_ref()
            .and_then(|antonym| antonym.split('・').next())
    }

    // Get a senses tags prettified
    pub fn get_parts_of_speech(&self, dict: &TranslationDict, language: LocLanguage) -> String {
        use itertools::Itertools;
        self.part_of_speech
            .iter()
            .map(|i| i.gettext_custom(dict, Some(language)))
            .join(", ")
    }

    pub fn get_infos(
        &self,
        dict: &TranslationDict,
        language: LocLanguage,
    ) -> Option<(
        Option<String>,
        Option<&str>,
        Option<&str>,
        Option<Dialect>,
        Option<String>,
    )> {
        let info_str = self.get_information_string(dict, language);
        let xref = self.get_xref();
        let antonym = self.get_antonym();
        let dialect = self.dialect;

        if xref.is_none() && info_str.is_none() && antonym.is_none() && self.gairaigo.is_none() {
            None
        } else {
            let gairaigo_txt = self.get_gairaigo(dict, language);
            Some((info_str, xref, antonym, dialect, gairaigo_txt))
        }
    }

    fn get_gairaigo(&self, dict: &TranslationDict, language: LocLanguage) -> Option<String> {
        self.gairaigo.as_ref().map(|gairaigo| {
            let lang = gairaigo
                .language
                .pgettext(dict, "foreign_lang", Some(language));
            dict.gettext_fmt("From {}: {}", &[lang, &gairaigo.original], Some(language))
        })
    }

    /// Return human readable information about a gloss
    pub fn get_information_string(
        &self,
        dict: &TranslationDict,
        language: LocLanguage,
    ) -> Option<String> {
        use itertools::Itertools;
        let arr: [Option<String>; 3] = [
            self.misc
                .map(|i| i.gettext(dict, Some(language)).to_owned()),
            self.field.map(|i| i.gettext_custom(dict, Some(language))),
            self.information.clone(),
        ];

        let res = arr
            .iter()
            .filter_map(|i| i.is_some().then(|| i.as_ref().unwrap()))
            .collect::<Vec<_>>();

        if res.is_empty() {
            return None;
        }

        if self.xref.is_some() || self.antonym.is_some() {
            Some(format!("{}.", res.iter().join(", ")))
        } else {
            Some(res.iter().join(", "))
        }
    }
}

/// Iterator over all Senses and its glosses
pub struct SenseGlossIter<'a> {
    word: &'a Word,
    sense_pos: usize,
    gloss_pos: usize,
}

impl<'a> SenseGlossIter<'a> {
    #[inline]
    pub(super) fn new(word: &'a Word) -> Self {
        SenseGlossIter {
            word,
            sense_pos: 0,
            gloss_pos: 0,
        }
    }
}

impl<'a> Iterator for SenseGlossIter<'a> {
    type Item = (&'a Sense, &'a Gloss);

    fn next(&mut self) -> Option<Self::Item> {
        let senses = &self.word.senses;
        if senses.len() <= self.sense_pos {
            return None;
        }

        let sense = &senses[self.sense_pos];
        assert!(!sense.glosses.is_empty());
        let gloss = &sense.glosses[self.gloss_pos];

        self.gloss_pos += 1;
        if self.gloss_pos >= sense.glosses.len() {
            self.gloss_pos = 0;
            self.sense_pos += 1;
        }

        Some((sense, gloss))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_gloss(word: &str) -> Gloss {
        Gloss {
            gloss: word.to_string(),
            ..Default::default()
        }
    }

    fn make_word(senses: &[&[&str]]) -> Word {
        let built_senses = senses
            .iter()
            .map(|sense| Sense {
                glosses: sense.iter().map(|i| make_gloss(i)).collect(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        Word {
            senses: built_senses,
            ..Default::default()
        }
    }

    #[test]
    fn test_sense_gloss_iter() {
        let word_empty = make_word(&[]);
        assert_eq!(word_empty.sense_gloss_iter().next(), None);

        let test_word = |data: &[&[&str]]| {
            let word1 = make_word(data);
            let mut iter1 = word1.sense_gloss_iter();

            for i in data.into_iter().map(|i| i.iter()).flatten() {
                assert_eq!(iter1.next().unwrap().1.gloss.as_str(), *i);
            }
            assert_eq!(iter1.next(), None);
        };

        let words = vec![
            vec![&["gloss0_0"][..]],
            vec![&["gloss0_0"][..], &["gloss1_0"][..]],
            vec![&["gloss0_0", "gloss0_1"][..], &["gloss1_0", "gloss1_1"][..]],
        ];

        for word in words {
            test_word(&word);
        }
    }

    #[test]
    fn test_unique_id() {
        let pairs = &[(1, 70), (10, 6), (0, 0), (255, 255), (1, 2)];

        for (seq, gloss) in pairs {
            let enc = to_unique_id(*seq, *gloss);
            let (seq_res, gloss_res) = from_unique_id(enc);
            assert_eq!(*seq, seq_res);
            assert_eq!(*gloss, gloss_res);
        }
    }
}
