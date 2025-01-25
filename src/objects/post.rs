use std::borrow::Cow;

use nostr_sdk::Timestamp;

#[derive(Default, Clone, Debug)]
pub struct Author<'a> {
    pub username: Option<Cow<'a, str>>,
    pub display_name: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug)]
pub struct Post<'a> {
    pub author: Author<'a>,
    pub content: Cow<'a, str>,
    pub created_at: Timestamp,
    pub categories: Vec<String>,
}
