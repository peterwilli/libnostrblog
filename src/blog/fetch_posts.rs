use std::time::Duration;

use crate::blog::deletion::{deletion_aware_post_filter, filter_deleted_events};
use crate::blog::extensions::to_posts::ToPosts;
use crate::{Blog, objects::post::Post};
use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::Timestamp;
use tracing::debug;

#[async_trait(?Send)]
pub trait FetchPostExt {
    async fn fetch_posts<'a>(&self, from: Option<Timestamp>) -> Result<Vec<Post<'a>>>;
}

#[async_trait(?Send)]
impl FetchPostExt for Blog<'_> {
    async fn fetch_posts<'a>(&self, from: Option<Timestamp>) -> Result<Vec<Post<'a>>> {
        let owners = self
            .authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>();
        let mut filter = deletion_aware_post_filter(owners);
        if let Some(from) = from {
            filter = filter.since(from);
        }

        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .unwrap();
        debug!("Events: {:?}", events);
        let posts = filter_deleted_events(events.into_iter().collect())
            .into_iter()
            .to_posts(self.authors.clone())
            .collect();
        Ok(posts)
    }
}
