use std::borrow::Cow;

use nostr_sdk::Timestamp;

use crate::types::CheapClonePubkey;

#[derive(Clone, Debug)]
pub struct Author<'a> {
    pub username: Option<Cow<'a, str>>,
    pub display_name: Option<Cow<'a, str>>,
    pub pubkey: CheapClonePubkey,
}

impl Author<'_> {
    pub fn from_pubkey(key: CheapClonePubkey) -> Self {
        Self {
            username: None,
            display_name: None,
            pubkey: key,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Post<'a> {
    pub author: Author<'a>,
    pub title: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub excerpt: Cow<'a, str>,
    pub created_at: Timestamp,
    pub categories: Vec<Cow<'a, str>>,
}
