pub mod detailed;
pub mod short;

use types::api::news::NewsEntry;

fn ne_from_resource(src: &news::NewsEntry, short: bool) -> NewsEntry {
    let html = if short {
        src.short.clone()
    } else {
        src.long.clone()
    };

    NewsEntry {
        id: src.id,
        html,
        title: src.title.clone(),
        creation_time: src.creation_time,
        trimmed: src.was_trimmed && !short,
    }
}
