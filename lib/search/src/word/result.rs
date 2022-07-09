use types::jotoba::{
    kanji::Kanji,
    words::{inflection::Inflection, Word},
};

#[derive(Default, Clone)]
pub struct AddResData {
    pub sentence: Option<SentenceInfo>,
    pub inflection: Option<InflectionInformation>,
    pub raw_query: String,
}

impl AddResData {
    pub fn has_sentence(&self) -> bool {
        self.sentence.is_some()
    }

    pub fn has_inflection(&self) -> bool {
        self.inflection.is_some()
    }

    pub fn sentence_parts(&self) -> Option<&sentence_reader::Sentence> {
        self.sentence.as_ref().and_then(|i| i.parts.as_ref())
    }

    pub fn sentence_index(&self) -> usize {
        self.sentence.as_ref().map(|i| i.index).unwrap_or(0)
    }
}

#[derive(Default, Clone)]
pub struct SentenceInfo {
    pub parts: Option<sentence_reader::Sentence>,
    pub index: usize,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordResult {
    pub items: Vec<Item>,
    pub count: usize,
    pub contains_kanji: bool,
    pub inflection_info: Option<InflectionInformation>,
    pub sentence_parts: Option<sentence_reader::Sentence>,
    pub sentence_index: usize,
    pub searched_query: String,
}

impl WordResult {
    #[inline]
    pub fn has_word(&self) -> bool {
        self.items.iter().any(|i| i.is_word())
    }

    /// Returns all words and kanji split in two separate lists
    pub fn get_items(&self) -> (Vec<&Word>, Vec<&Kanji>) {
        let mut words = vec![];
        let mut kanjis = vec![];

        for item in &self.items {
            match item {
                Item::Word(word) => words.push(word),
                Item::Kanji(kanji) => kanjis.push(kanji),
            }
        }

        (words, kanjis)
    }

    #[inline]
    pub fn words(&self) -> impl Iterator<Item = &Word> {
        self.items.iter().filter_map(|i| match i {
            Item::Word(w) => Some(w),
            Item::Kanji(_) => None,
        })
    }

    #[inline]
    pub fn kanji(&self) -> impl Iterator<Item = &Kanji> {
        self.items.iter().filter_map(|i| match i {
            Item::Word(_) => None,
            Item::Kanji(k) => Some(k),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InflectionInformation {
    /// Normalized form of the word
    pub lexeme: String,
    /// All inflections
    pub inflections: Vec<Inflection>,
}

impl InflectionInformation {
    pub fn from_part(part: &sentence_reader::Part) -> Option<Self> {
        if !part.has_inflections() {
            return None;
        }

        Some(InflectionInformation {
            lexeme: part.get_normalized(),
            inflections: part.inflections().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Word(Word),
    Kanji(Kanji),
}

impl Item {
    /// Returns `true` if the item is [`Word`].
    #[inline]
    pub fn is_word(&self) -> bool {
        matches!(self, Self::Word(..))
    }

    /// Returns `true` if the item is [`Kanji`].
    #[inline]
    pub fn is_kanji(&self) -> bool {
        matches!(self, Self::Kanji(..))
    }
}

impl From<Kanji> for Item {
    #[inline]
    fn from(k: Kanji) -> Self {
        Self::Kanji(k)
    }
}

impl From<Word> for Item {
    #[inline]
    fn from(w: Word) -> Self {
        Self::Word(w)
    }
}

pub fn selected(curr: usize, selected: usize) -> &'static str {
    if curr == selected {
        "selected"
    } else {
        ""
    }
}
