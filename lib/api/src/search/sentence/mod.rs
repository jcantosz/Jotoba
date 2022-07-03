use actix_web::web::{self, Json};
use types::{
    api::search::sentence::{Response, Sentence},
    jotoba::search::SearchTarget,
};

use super::{Result, SearchRequest};

/// Do a Sentence search via API
pub async fn sentence_search(payload: Json<SearchRequest>) -> Result<Json<Response>> {
    let query = super::parse_query(payload, SearchTarget::Kanji)?;

    let result = web::block(move || search::sentence::Search::new(&query).search())
        .await??
        .items
        .into_iter()
        .map(|i| search_to_sentence(i))
        .collect::<Vec<_>>();

    Ok(Json(result.into()))
}

#[inline]
fn search_to_sentence(sentence: search::sentence::result::Sentence) -> Sentence {
    Sentence {
        eng: sentence.get_english().map(|i| i.to_owned()),
        content: sentence.content,
        furigana: sentence.furigana,
        translation: sentence.translation,
        language: sentence.language,
    }
}
