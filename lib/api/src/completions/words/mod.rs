pub mod foreign;
pub mod hashtag;
pub mod kana_end_ext;
pub mod native;

use std::{cmp::Ordering, time::Instant};

use japanese::JapaneseExt;
use search::query::{Query, QueryLang};
use types::api::completions::{Response, WordPair};
use utils::bool_ord;
use wana_kana::{to_katakana::to_katakana, to_romaji::to_romaji};

/// Returns word suggestions based on the query. Applies various approaches to give better results
pub(crate) fn suggestions(query: Query, radicals: &[char]) -> Option<Response> {
    let response = try_word_suggestions(&query, radicals)?;

    // Tries to do a katakana search if nothing was found
    let result = if response.is_empty() && query.query_str.is_hiragana() {
        try_word_suggestions(&get_katakana_query(&query), radicals)?
    } else {
        response
    };

    Some(Response::new(result))
}

/// Returns Ok(suggestions) for the given query ordered and ready to display
fn try_word_suggestions(query: &Query, radicals: &[char]) -> Option<Vec<WordPair>> {
    let start = Instant::now();
    // Get sugesstions for matching language

    let romaji_query = to_romaji(query.query_str.as_str());

    let word_pairs = match query.q_lang {
        QueryLang::Japanese => native::suggestions(&query, &romaji_query, radicals)?,
        QueryLang::Foreign | QueryLang::Undetected | QueryLang::Korean => {
            let mut res = foreign::suggestions(&query, &query.query_str).unwrap_or_default();

            // Order: put exact matches to top
            res.sort_by(|a, b| word_pair_order(a, b, &query.query_str));
            res
        }
    };
    log::debug!("Suggestions took: {:?}", start.elapsed());

    Some(word_pairs)
}

/// Ordering for [`WordPair`]s which puts the exact matches to top
#[inline]
fn word_pair_order(a: &WordPair, b: &WordPair, query: &str) -> Ordering {
    bool_ord(a.has_reading(&query), b.has_reading(&query))
}

/// Returns an equivalent katakana query
fn get_katakana_query(query: &Query) -> Query {
    Query {
        query_str: to_katakana(query.query_str.as_str()),
        ..query.clone()
    }
}
