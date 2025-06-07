use super::extensions::{filters::FiltersExt, to_posts::ToPosts};
use crate::{Blog, objects::post::Post};
use any_spawner::Executor;
use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;

#[async_trait(?Send)]
pub trait PollPostsExt {
    async fn poll_posts(&self, from: Option<Timestamp>) -> Result<mpsc::Receiver<Post<'static>>>;
}

#[async_trait(?Send)]
impl PollPostsExt for Blog<'_> {
    async fn poll_posts(&self, from: Option<Timestamp>) -> Result<mpsc::Receiver<Post<'static>>> {
        let (tx, rx) = mpsc::channel(1);
        let owners = self
            .authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>();

        // Clone only what's needed and can live long enough
        let authors = self.authors.clone();
        let client = self.client.clone();
        let mut filter = Filter::new().posts_by_owners(owners);
        if let Some(from) = from {
            filter = filter.since(from);
        }
        let mut handle = client
            .stream_events(filter, Duration::from_secs(10))
            .await?;
        Executor::spawn_local(async move {
            while let Some(event) = handle.next().await {
                let post = [event]
                    .into_iter()
                    .to_posts(authors.clone())
                    .next()
                    .unwrap();
                if tx.send(post).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }
}
