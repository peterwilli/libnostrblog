use std::borrow::Cow;

use nostr_sdk::{Client, PublicKey, Timestamp};
use objects::post::Post;

#[cfg(test)]
mod tests;

pub mod blog;
pub mod objects;

pub struct Blog<'a> {
    owner: PublicKey,
    client: Client,
    categories: Vec<Cow<'a, str>>,
    posts: Vec<Post<'a>>,
    last_pull: Timestamp,
}

impl<'a> Blog<'a> {
    pub fn new(client: Client, owner: PublicKey) -> Self {
        Self {
            client,
            owner,
            posts: vec![],
            categories: vec![],
            last_pull: Timestamp::zero(),
        }
    }
}
