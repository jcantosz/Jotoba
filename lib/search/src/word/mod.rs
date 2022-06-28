pub mod kanji;
pub mod order;
mod regex;
pub mod result;
pub mod tag_only;

use crate::{
    engine::{
        result::SearchResult,
        result_item::ResultItem,
        words::{
            foreign::{self, output::WordOutput},
            native,
        },
        SearchTask,
    },
    query::Form,
};

use self::result::{InflectionInformation, WordResult};
use super::query::{Query, QueryLang};
use error::Error;
use itertools::Itertools;
use japanese::JapaneseExt;
use result::Item;
use sentence_reader::igo_unidic::WordClass;
use sentence_reader::output::ParseResult;
use types::jotoba::{
    kanji::Kanji,
    languages::Language,
    search::guess::Guess,
    words::{filter_languages, part_of_speech::PosSimple, Word},
};
use utils::{real_string_len, to_option};

pub struct Search<'a> {
    query: &'a Query,
}

/// Search among all data based on the input query
#[inline]
pub fn search(query: &Query) -> Result<WordResult, Error> {
    Search { query }.do_search()
}

#[derive(Default)]
pub struct ResultData {
    pub words: Vec<Word>,
    pub infl_info: Option<InflectionInformation>,
    pub count: usize,
    pub sentence_index: usize,
    pub sentence_parts: Option<sentence_reader::Sentence>,
    pub searched_query: String,
}

impl<'a> Search<'a> {
    /// Do the search
    fn do_search(&self) -> Result<WordResult, Error> {
        let search_result = match self.query.form {
            Form::KanjiReading(_) => kanji::by_reading(self)?,
            Form::TagOnly => tag_only::search(self)?,
            _ => self.do_word_search()?,
        };

        let words = search_result.words;

        let kanji_results = kanji::load_word_kanji_info(&words);

        let res = WordResult {
            contains_kanji: !kanji_results.is_empty(),
            items: Self::merge_words_with_kanji(words, kanji_results),
            inflection_info: search_result.infl_info,
            count: search_result.count,
            sentence_parts: search_result.sentence_parts,
            sentence_index: search_result.sentence_index,
            searched_query: search_result.searched_query,
        };
        Ok(res)
    }

    /// Search by a word
    fn do_word_search(&self) -> Result<ResultData, Error> {
        let native_word_res = self.native_results(&self.query.query)?;
        let gloss_word_res = self.gloss_results()?;

        let sentence_parts = native_word_res
            .sentence_parts
            .or(gloss_word_res.sentence_parts);

        let infl_info = native_word_res.infl_info.or(gloss_word_res.infl_info);

        // Chain native and word results into one vector
        Ok(ResultData {
            words: native_word_res
                .words
                .into_iter()
                .chain(gloss_word_res.words)
                .collect_vec(),
            infl_info,
            count: native_word_res.count + gloss_word_res.count,
            sentence_parts,
            sentence_index: self.query.word_index,
            searched_query: native_word_res.searched_query,
        })
    }

    fn parse_sentence<'b>(
        &'b self,
        query_str: &'a str,
        allow_sentence: bool,
    ) -> (
        String,
        Option<sentence_reader::Sentence>,
        Option<sentence_reader::Part>,
    ) {
        if !self.query.parse_japanese || !allow_sentence {
            return (query_str.to_owned(), None, None);
        }

        let parse_res = sentence_reader::Parser::new(query_str).parse();

        if let ParseResult::Sentence(sentence) = parse_res {
            // Don't show sentence reader for words that are in DB
            let in_db = SearchTask::<native::Engine>::new(query_str).has_term();
            if in_db {
                return (query_str.to_string(), None, None);
            }

            let index = self.query.word_index.clamp(0, sentence.word_count() - 1);
            let word = sentence.get_at(index).unwrap().clone();
            return (word.get_normalized(), Some(sentence), Some(word));
        }

        if let ParseResult::InflectedWord(word) = parse_res {
            let s = word.get_normalized();
            return (s, None, Some(word));
        }

        (query_str.to_owned(), None, None)
    }

    /// Returns a `SearchTask` for the current query. This will be used to find all words for
    /// the search
    fn native_search_task<'b>(
        &self,
        query: &'b str,
        original_query: &str,
        sentence: bool,
    ) -> SearchTask<'b, native::Engine> {
        let mut search_task: SearchTask<native::Engine> = SearchTask::new(&query)
            .limit(self.query.settings.page_size as usize)
            .offset(self.query.page_offset)
            .threshold(0.04f32);

        // apply user filter
        let q_cloned = self.query.clone();
        let pos_filter = self.get_pos_filter(sentence);
        search_task.set_result_filter(move |word| Self::word_filter(&q_cloned, word, &pos_filter));

        // Set order function;
        let original_query = original_query.to_string();
        search_task.with_custom_order(move |item| {
            order::japanese_search_order(item, Some(&original_query))
        });

        search_task
    }

    fn native_search(
        &self,
        query_str: &str,
        allow_sentence: bool,
    ) -> Result<
        (
            SearchResult<&'static Word>,
            Option<InflectionInformation>,
            Option<sentence_reader::Sentence>,
            String,
        ),
        Error,
    > {
        if self.query.language != QueryLang::Japanese && !query_str.is_japanese() {
            return Err(Error::NotFound);
        }

        // Try regex search
        // prevent heavy queries
        if real_string_len(query_str) >= 2 || query_str.has_kanji() {
            if let Some(regex_query) = self.query.as_regex_query() {
                let limit = self.query.settings.page_size;
                let offset = self.query.page_offset;
                let res = regex::search(regex_query, limit as usize, offset);
                if !res.is_empty() {
                    return Ok((res, None, None, query_str.to_string()));
                }
            }
        }

        let fmt_query = japanese::to_halfwidth(&self.query.query);
        let (query, mut sentence, word_info) = self.parse_sentence(&fmt_query, allow_sentence);

        let original_query = if sentence.is_some() {
            word_info.as_ref().unwrap().get_inflected().clone()
        } else {
            self.query.query.clone()
        };

        let mut search_task = self.native_search_task(&query, &original_query, sentence.is_some());

        let inflected_version = word_info.as_ref().map(|i| i.get_inflected());
        if let Some(inflected_version) = &inflected_version {
            search_task.add_query(inflected_version);
        }

        // If query was modified (ie. through reflection), search for original too
        if query != query_str && sentence.is_none() {
            search_task.add_query(&self.query.query);
        }

        let res = search_task.find();
        /*
        println!("{:#?}", res.iter().next().unwrap());
        println!("{:#?}", res.iter().next().unwrap().item.get_furigana());
        */

        // Put furigana to sentence
        if let Some(sentence) = &mut sentence {
            for part in sentence.iter_mut() {
                let p = part.clone();
                part.set_furigana(|inp| furigana_by_reading(inp, &p))
            }
        }

        let infl_info = word_info
            .as_ref()
            .and_then(|i| InflectionInformation::from_part(i));

        let searched_query = word_info
            .as_ref()
            .map(|i| i.get_inflected())
            .unwrap_or(query);

        Ok((res, infl_info, sentence, searched_query))
    }

    /// Perform a native word search
    fn native_results(&self, query_str: &str) -> Result<ResultData, Error> {
        let (res, infl_info, sentence, searched_query) = match self.native_search(query_str, true) {
            Ok(v) => v,
            Err(err) => match err {
                Error::NotFound => return Ok(ResultData::default()),
                _ => return Err(err),
            },
        };

        let count = res.len();

        let mut wordresults = res.into_iter().cloned().collect::<Vec<_>>();

        filter_languages(
            wordresults.iter_mut(),
            self.query.settings.user_lang,
            self.query.settings.show_english,
        );

        Ok(ResultData {
            count,
            words: wordresults,
            infl_info,
            sentence_parts: sentence,
            sentence_index: self.query.word_index,
            searched_query,
            ..Default::default()
        })
    }

    /// Returns a `SearchTask` for the current query. This will be used to find all words for
    /// the search
    fn gloss_search_task(&self) -> SearchTask<foreign::Engine> {
        let used_lang = self.query.get_lang_with_override();

        let mut search_task: SearchTask<foreign::Engine> =
            SearchTask::with_language(&self.query.query, used_lang)
                .limit(self.query.settings.page_size as usize)
                .offset(self.query.page_offset)
                .threshold(0.4f32);

        //println!("searching in {}", used_lang);

        if self.query.settings.show_english && used_lang != Language::English
        // Don't show english results if user wants to search in a specified language
        //&& self.query.language_override.is_none()
        {
            search_task.add_language_query(&self.query.query, Language::English);
        }

        // Set user defined filter
        let pos_filter = to_option(self.query.get_part_of_speech_tags().copied().collect());
        let q_cloned = self.query.clone();
        search_task
            .set_result_filter(move |word| Self::word_filter(&q_cloned, &word.word, &pos_filter));

        // Set order function
        let orderer = order::foreign::ForeignOrder::new();
        search_task.with_custom_order(move |item| orderer.score(item, used_lang));

        search_task
    }

    /// Search for words by their translations
    fn gloss_results(&self) -> Result<ResultData, Error> {
        if !matches!(
            self.query.language,
            QueryLang::Foreign | QueryLang::Undetected
        ) {
            return Ok(ResultData::default());
        }

        let mut search_task = self.gloss_search_task();

        let could_be_romaji = japanese::guessing::could_be_romaji(&self.query.query);

        // TODO: fix aligning
        search_task.set_align(false);
        /*
        if could_be_romaji {
            search_task.set_align(false);
        }
        */

        // Do the search
        let mut res = search_task.find();
        let mut count = res.len();

        let mut infl_info = None;
        let mut sentence = None;
        let mut searched_query = self.query.query.clone();

        if !self.query.use_original
            && count < 50
            && could_be_romaji
            && !SearchTask::<foreign::Engine>::with_language(
                &self.query.query.to_lowercase(),
                self.query.get_lang_with_override(),
            )
            .has_term()
        {
            let hg_query = utils::format_romaji_nn(&self.query.query).to_hiragana();
            let hg_search = self.native_search(&hg_query, false);
            if let Ok((native_res, inflection_info, sent, sq)) = hg_search {
                infl_info = inflection_info;
                sentence = sent;
                searched_query = sq;
                // hacky but works (I guess)
                res.total_items += native_res.total_items;
                count = res.len();
                let other = native_res.into_inner().into_iter().map(|i| {
                    let word = i.item;
                    let relevance = i.relevance;
                    let language = i.language;
                    ResultItem::new_raw(WordOutput::new(word, vec![]), relevance, language)
                });
                res.merge(other);
            }
        }

        // If there aren't any results, check if there is another language
        if count == 0 {
            return self.check_other_lang();
        }

        /*
        println!("----------\n+++++++++++++++++++++++\n==============");
        for i in res.iter() {
            println!("{:?}: {}", i.item.word.get_reading().reading, i.relevance);
        }
        */

        let mut wordresults = res.into_iter().map(|i| i.word.clone()).collect::<Vec<_>>();

        filter_languages(
            wordresults.iter_mut(),
            self.query.get_lang_with_override(),
            self.query.settings.show_english,
        );

        Ok(ResultData {
            count,
            words: wordresults,
            infl_info,
            sentence_parts: sentence,
            sentence_index: self.query.word_index,
            searched_query,
            ..Default::default()
        })
    }

    fn check_other_lang(&self) -> Result<ResultData, Error> {
        let guessed_langs = foreign::guess_language(&self.query.query)
            .into_iter()
            .filter(|i| *i != self.query.get_lang_with_override())
            .collect::<Vec<_>>();

        if guessed_langs.len() == 1 {
            let mut new_query = self.query.clone();
            new_query.language_override = Some(guessed_langs[0]);
            return Search { query: &new_query }.gloss_results();
        }

        Ok(ResultData::default())
    }

    fn get_pos_filter(&self, is_sentence: bool) -> Option<Vec<PosSimple>> {
        let pos_filter_tags = self
            .query
            .get_part_of_speech_tags()
            .copied()
            .collect::<Vec<_>>();

        (!pos_filter_tags.is_empty() && !is_sentence).then(|| pos_filter_tags)
    }

    #[inline]
    fn merge_words_with_kanji(words: Vec<Word>, kanji: Vec<Kanji>) -> Vec<Item> {
        kanji
            .into_iter()
            .map(|i| i.into())
            .chain(words.into_iter().map(|i| i.into()).collect_vec())
            .collect_vec()
    }

    /// Returns false if a word should be filtered out of results
    fn word_filter(query: &Query, word: &Word, pos_filter: &Option<Vec<PosSimple>>) -> bool {
        // Apply pos tag filter
        if !pos_filter
            .as_ref()
            .map(|filter| word.has_pos(&filter))
            .unwrap_or(true)
        {
            return false;
        }

        if !word.has_language(query.settings.user_lang, query.settings.show_english) {
            return false;
        }

        // Apply misc filter
        for misc_filter in query.get_misc_tags() {
            if !word.has_misc(*misc_filter) {
                return false;
            }
        }

        true
    }
}

/// Returns furigana of the given `morpheme` if available
fn furigana_by_reading(morpheme: &str, part: &sentence_reader::Part) -> Option<String> {
    let word_storage = resources::get().words();

    let mut st = SearchTask::<native::Engine>::new(morpheme)
        .threshold(0.7)
        .limit(10);

    let pos = wc_to_simple_pos(&part.word_class_raw());
    let morph = morpheme.to_string();
    st.with_custom_order(move |item| furi_order(item.item(), &pos, &morph));

    let found = st.find();
    if found.is_empty() {
        return None;
    }
    let word = word_storage.by_sequence(found[0].item.sequence as u32)?;
    word.furigana.as_ref().cloned()
}

fn furi_order(i: &Word, pos: &Option<PosSimple>, morph: &str) -> usize {
    let mut score: usize = 100;
    if i.get_reading().reading != morph {
        score = 0;
    }
    if let Some(pos) = pos {
        if i.has_pos(&[*pos]) {
            score += 20;
        } else {
            score = score.saturating_sub(30);
        }
    }
    if i.is_common() {
        score += 2;
    }

    if i.get_jlpt_lvl().is_some() {
        score += 2;
    }
    score
}

pub fn wc_to_simple_pos(wc: &WordClass) -> Option<PosSimple> {
    Some(match wc {
        WordClass::Particle(_) => PosSimple::Particle,
        WordClass::Verb(_) => PosSimple::Verb,
        WordClass::Adjective(_) => PosSimple::Adjective,
        WordClass::Adverb => PosSimple::Adverb,
        WordClass::Noun(_) => PosSimple::Noun,
        WordClass::Pronoun => PosSimple::Pronoun,
        WordClass::Interjection => PosSimple::Interjection,
        WordClass::Conjungtion => PosSimple::Conjungation,
        WordClass::Suffix => PosSimple::Suffix,
        WordClass::Prefix => PosSimple::Prefix,
        _ => return None,
    })
}

pub fn guess_inp_language(query: &Query) -> Vec<Language> {
    foreign::guess_language(&query.query)
        .into_iter()
        .filter(|i| *i != query.get_lang_with_override())
        .collect()
}

/// Guesses the amount of results a search would return with given `query`
pub fn guess_result(query: &Query) -> Option<Guess> {
    let search = Search { query };

    if query.language == QueryLang::Japanese {
        guess_native(search)
    } else {
        guess_foreign(search)
    }
}

fn guess_native(search: Search) -> Option<Guess> {
    let (query, sentence, _) = search.parse_sentence(&search.query.query, false);
    search
        .native_search_task(&query, &search.query.query, sentence.is_some())
        .estimate_result_count()
        .ok()
}

fn guess_foreign(search: Search) -> Option<Guess> {
    let mut task = search.gloss_search_task();
    task.set_align(false);
    task.estimate_result_count().ok()
}

impl ResultData {
    #[inline]
    pub fn new(words: Vec<Word>, count: usize) -> Self {
        Self {
            words,
            count,
            ..Default::default()
        }
    }
}
