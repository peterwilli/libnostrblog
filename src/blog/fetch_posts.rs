use std::time::Duration;

use crate::blog::extensions::filters::FiltersExt;
use crate::blog::extensions::to_posts::ToPosts;
use crate::{Blog, objects::post::Post};
use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::Filter;
use tracing::debug;

#[async_trait(?Send)]
pub trait FetchPostExt {
    async fn fetch_posts<'a>(&self) -> Result<Vec<Post<'a>>>;
}

#[async_trait(?Send)]
impl FetchPostExt for Blog<'_> {
    async fn fetch_posts<'a>(&self) -> Result<Vec<Post<'a>>> {
        let owners = self
            .authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>();
        let events = self
            .client
            .fetch_events(
                Filter::new().posts_by_owners(owners),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
        debug!("Events: {:?}", events);
        let posts = events.into_iter().to_posts(self.authors.clone()).collect();
        Ok(posts)
    }
}
