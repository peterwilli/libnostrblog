use std::time::Duration;

use crate::blog::extensions::to_posts::ToPosts;
use crate::blog::filters::PostsFiltersExt;
use crate::{objects::post::Post, Blog};
use anyhow::Result;
use async_trait::async_trait;
use tracing::debug;

#[async_trait]
pub trait FetchPostExt {
    async fn fetch_posts(&self) -> Result<Vec<Post<'_>>>;
}

#[async_trait]
impl FetchPostExt for Blog<'_> {
    async fn fetch_posts(&self) -> Result<Vec<Post<'_>>> {
        let events = self
            .client
            .fetch_events(vec![self.owner_filter()], Duration::from_secs(10))
            .await?;
        debug!("Events: {:?}", events);
        let posts = events.iter().to_posts().collect();
        Ok(posts)
    }
}
