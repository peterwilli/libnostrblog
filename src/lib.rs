use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use nostr_sdk::{Client, PublicKey, Timestamp};
use objects::post::{Author, Post};
use parking_lot::RwLock;
use types::Authors;

#[cfg(test)]
mod tests;

pub mod blog;
pub mod objects;
pub mod types;

pub struct Blog<'a> {
    authors: Authors,
    client: Client,
    categories: RwLock<HashSet<Cow<'a, str>>>,
    posts: RwLock<Vec<Post<'a>>>,
    last_pull: Timestamp,
}

impl Blog<'_> {
    pub fn new(client: Client, owners: Vec<PublicKey>) -> Self {
        Self {
            client,
            authors: Arc::new(RwLock::new(
                owners
                    .into_iter()
                    .map(|pk| {
                        let pk = Arc::new(pk);
                        (pk.clone(), Author::from_pubkey(pk))
                    })
                    .collect(),
            )),
            posts: Default::default(),
            categories: Default::default(),
            last_pull: Timestamp::zero(),
        }
    }
}
