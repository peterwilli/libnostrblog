use nostr_sdk::Event;
use std::borrow::Cow;

pub fn get_tag_values<'a>(event: &Event, key: &str) -> Vec<Cow<'a, str>> {
    event
        .tags
        .iter()
        .filter_map(|t| {
            let [k, v] = t.as_slice() else {
                return None;
            };
            if k == key {
                Some(Cow::Owned(v.to_owned()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
