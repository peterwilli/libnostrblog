use super::{
    deletion::{deletion_aware_post_filter, filter_deleted_events},
    extensions::to_posts::ToPosts,
};
use crate::{Blog, objects::post::Post};
use any_spawner::Executor;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info};

#[async_trait(?Send)]
pub trait StreamPostsExt {
    async fn stream_posts(&self, from: Option<Timestamp>) -> Result<mpsc::Receiver<Post<'static>>>;
}

#[async_trait(?Send)]
impl StreamPostsExt for Blog<'_> {
    async fn stream_posts(&self, from: Option<Timestamp>) -> Result<mpsc::Receiver<Post<'static>>> {
        let (tx, rx) = mpsc::channel(1);
        let owners = self
            .authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>();

        let authors = self.authors.clone();
        let client = self.client.clone();
        let mut filter = deletion_aware_post_filter(owners);
        if let Some(from) = from {
            filter = filter.since(from);
        }
        let mut events = client
            .stream_events(filter, Duration::from_secs(10))
            .await?;
        Executor::spawn_local(async move {
            while let Some(event) = events.next().await {
                if event.kind == Kind::EventDeletion {
                    debug!("stream received post deletion event: {:?}", event.id);
                    continue;
                }

                let posts = filter_deleted_events(vec![event])
                    .into_iter()
                    .to_posts(authors.clone());
                for post in posts {
                    if tx.send(post).await.is_err() {
                        return;
                    }
                }
            }
            info!("post stream done");
        });

        Ok(rx)
    }
}
