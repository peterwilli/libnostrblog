use std::{borrow::Cow, collections::HashSet, sync::Arc};

use objects::post::{Author, Post};
use parking_lot::RwLock;
use types::Authors;

pub use nostr_sdk::*;

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
    settings: BlogSettings,
}

#[derive(Clone, Debug, Default)]
pub struct BlogSettings {
    pub require_comment_approval: bool,
}

impl Blog<'_> {
    pub fn new(client: Client, owners: Vec<PublicKey>) -> Self {
        Self::new_with_settings(client, owners, BlogSettings::default())
    }

    pub fn new_with_settings(
        client: Client,
        owners: Vec<PublicKey>,
        settings: BlogSettings,
    ) -> Self {
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
            settings,
        }
    }

    pub fn require_comment_approval(&self) -> bool {
        self.settings.require_comment_approval
    }

    pub(crate) fn owner_public_keys(&self) -> Vec<PublicKey> {
        self.authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>()
    }
}
