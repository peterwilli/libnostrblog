use super::filters::PostsFiltersExt;
use crate::{objects::post::Post, Blog};
use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};

#[async_trait]
pub trait PollPostsExt {
    async fn poll_posts(&self) -> Result<mpsc::Receiver<Vec<Post<'_>>>>;
}

#[async_trait]
impl PollPostsExt for Blog<'_> {
    async fn poll_posts(&self) -> Result<mpsc::Receiver<Vec<Post<'_>>>> {
        let (tx, rx) = mpsc::channel(1);
        let mut handle = self
            .client
            .stream_events(vec![self.owner_filter()], Duration::from_secs(10))
            .await?;

        tokio::spawn(async move { while let Some(event) = handle.next().await {} });
        Ok(rx)
    }
}
