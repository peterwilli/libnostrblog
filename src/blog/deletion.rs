use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::{Event, EventBuilder, EventId, Filter, Kind, Tag, TagStandard, ToBech32};

use crate::Blog;

#[async_trait(?Send)]
pub trait DeletionExt {
    async fn delete_post(
        &self,
        post_id: EventId,
        post_kind: Kind,
        reason: impl Into<String>,
    ) -> Result<String>;
}

#[async_trait(?Send)]
impl DeletionExt for Blog<'_> {
    async fn delete_post(
        &self,
        post_id: EventId,
        post_kind: Kind,
        reason: impl Into<String>,
    ) -> Result<String> {
        let event_id = self
            .client
            .send_event_builder(post_deletion_event_builder(post_id, post_kind, reason))
            .await?;
        Ok(event_id.id().to_bech32()?)
    }
}

pub fn post_deletion_event_builder(
    post_id: EventId,
    post_kind: Kind,
    reason: impl Into<String>,
) -> EventBuilder {
    EventBuilder::new(Kind::EventDeletion, reason.into())
        .tags([
            Tag::event(post_id),
            Tag::from_standardized_without_cell(TagStandard::Kind {
                kind: post_kind,
                uppercase: false,
            }),
        ])
        .dedup_tags()
}

pub fn deletion_aware_post_filter(owners: Vec<nostr_sdk::PublicKey>) -> Filter {
    Filter::new()
        .authors(owners)
        .kinds([Kind::LongFormTextNote, Kind::EventDeletion])
}

pub fn filter_deleted_events(events: Vec<Event>) -> Vec<Event> {
    let deleted_ids = deleted_event_ids(&events);
    events
        .into_iter()
        .filter(|event| event.kind != Kind::EventDeletion)
        .filter(|event| !deleted_ids.contains(&event.id))
        .collect()
}

fn deleted_event_ids(events: &[Event]) -> HashSet<EventId> {
    let posts = events
        .iter()
        .filter(|event| event.kind == Kind::LongFormTextNote)
        .map(|event| (event.id, (event.pubkey, event.kind)))
        .collect::<HashMap<_, _>>();

    events
        .iter()
        .filter(|event| event.kind == Kind::EventDeletion)
        .flat_map(|event| deleted_ids_from_request(event, &posts))
        .collect()
}

fn deleted_ids_from_request(
    event: &Event,
    posts: &HashMap<EventId, (nostr_sdk::PublicKey, Kind)>,
) -> Vec<EventId> {
    let requested_kinds = tag_values(event, "k")
        .map(str::to_owned)
        .collect::<HashSet<_>>();

    tag_values(event, "e")
        .filter_map(|value| EventId::from_hex(value).ok())
        .filter(|id| {
            posts.get(id).is_some_and(|(pubkey, kind)| {
                let kind = kind.as_u16().to_string();
                *pubkey == event.pubkey
                    && (requested_kinds.is_empty() || requested_kinds.contains(kind.as_str()))
            })
        })
        .collect()
}

fn tag_values<'a>(event: &'a Event, key: &'a str) -> impl Iterator<Item = &'a str> {
    event.tags.iter().filter_map(move |tag| {
        let [tag_key, value, ..] = tag.as_slice() else {
            return None;
        };
        (tag_key == key).then_some(value.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use nostr_sdk::Keys;

    #[test]
    fn deletion_request_removes_matching_author_long_form_post() -> Result<()> {
        let keys = Keys::generate();
        let post = EventBuilder::long_form_text_note("post").sign_with_keys(&keys)?;
        let deletion =
            post_deletion_event_builder(post.id, post.kind, "remove").sign_with_keys(&keys)?;

        let visible = filter_deleted_events(vec![post, deletion]);

        assert!(visible.is_empty());
        Ok(())
    }

    #[test]
    fn deletion_request_does_not_remove_other_author_long_form_post() -> Result<()> {
        let post_keys = Keys::generate();
        let other_keys = Keys::generate();
        let post = EventBuilder::long_form_text_note("post").sign_with_keys(&post_keys)?;
        let deletion = post_deletion_event_builder(post.id, post.kind, "remove")
            .sign_with_keys(&other_keys)?;

        let visible = filter_deleted_events(vec![post, deletion]);

        assert_eq!(visible.len(), 1);
        Ok(())
    }
}
