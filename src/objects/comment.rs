use nostr_sdk::{Event, EventId, PublicKey, Timestamp};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment<'a> {
    pub id: EventId,
    pub author: PublicKey,
    pub content: Cow<'a, str>,
    pub created_at: Timestamp,
    pub root: Option<EventId>,
    pub parent: Option<EventId>,
    pub approved: bool,
}

impl Comment<'_> {
    pub fn from_event<'a>(event: Event, approved: bool) -> Comment<'a> {
        let root = event_id_tag_value(&event, "E");
        let parent = event_id_tag_value(&event, "e");

        Comment {
            id: event.id,
            author: event.pubkey,
            content: Cow::Owned(event.content),
            created_at: event.created_at,
            root,
            parent,
            approved,
        }
    }
}

fn event_id_tag_value(event: &Event, tag_name: &str) -> Option<EventId> {
    event.tags.iter().find_map(|tag| {
        let [name, value, ..] = tag.as_slice() else {
            return None;
        };

        if name == tag_name {
            EventId::from_hex(value).ok()
        } else {
            None
        }
    })
}
