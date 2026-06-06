use crate::types::CheapClonePubkey;
use nostr_sdk::{EventId, Timestamp};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Author<'a> {
    pub username: Option<Cow<'a, str>>,
    pub display_name: Option<Cow<'a, str>>,
    pub picture: Option<Url>,
    pub pubkey: CheapClonePubkey,
    pub about: Option<Cow<'a, str>>,
}

impl Author<'_> {
    pub fn from_pubkey(key: CheapClonePubkey) -> Self {
        Self {
            username: None,
            display_name: None,
            picture: None,
            about: None,
            pubkey: key,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post<'a> {
    pub id: EventId,
    pub author: Author<'a>,
    pub title: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub excerpt: Cow<'a, str>,
    pub created_at: Timestamp,
    pub categories: Vec<Cow<'a, str>>,
    pub featured_image: Option<Url>,
}
