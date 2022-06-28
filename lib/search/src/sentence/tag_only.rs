use error::Error;
use types::jotoba::languages::Language;

use crate::query::tags::Tag;
use crate::query::Query;

use super::result::SentenceResult;

pub(super) fn search(query: &Query) -> Result<SentenceResult, Error> {
    let filter_tag = query
        .tags
        .iter()
        .find(|i| i.is_empty_allowed())
        // We expect to find one since this function should only be called if there is one
        .ok_or(Error::Unexpected)?;

    if let Tag::Jlpt(jlpt) = filter_tag {
        return jlpt_search(query, *jlpt);
    } else {
        return Ok(SentenceResult::default());
    }
}

fn jlpt_search(query: &Query, jlpt: u8) -> Result<SentenceResult, Error> {
    assert!(jlpt > 0 && jlpt < 6);

    let resources = resources::get().sentences();

    let senences = resources
        .by_jlpt(jlpt)
        .filter(|sentence| {
            sentence.has_translation(query.settings.user_lang)
                && (sentence.has_translation(Language::English) && query.settings.show_english)
        })
        .take(10000)
        .collect::<Vec<_>>();

    let len = senences.len();

    let show_english = query.settings.show_english;
    let sentences = senences
        .into_iter()
        .skip(query.page_offset)
        .take(query.settings.page_size as usize)
        .filter_map(|i| super::map_sentence_to_item(i, query.settings.user_lang, show_english))
        .collect::<Vec<_>>();

    let hidden = query.has_tag(Tag::Hidden);
    Ok(SentenceResult {
        items: sentences,
        len,
        hidden,
    })
}
