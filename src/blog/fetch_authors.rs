use std::time::Duration;

use crate::Blog;
use crate::blog::extensions::filters::FiltersExt;
use anyhow::Result;
use async_trait::async_trait;
use nostr_sdk::Filter;
use serde_json::Value;
use tracing::debug;
use url::Url;

#[async_trait(?Send)]
pub trait FetchAuthorsExt {
    async fn fetch_authors(&self) -> Result<()>;
}

#[async_trait(?Send)]
impl FetchAuthorsExt for Blog<'_> {
    async fn fetch_authors(&self) -> Result<()> {
        let owners = self
            .authors
            .read()
            .iter()
            .map(|(pk, _a)| **pk)
            .collect::<Vec<_>>();
        let events = self
            .client
            .fetch_events(
                Filter::new().metadata_by_owners(owners),
                Duration::from_secs(10),
            )
            .await?;
        debug!("Events: {:?}", events);
        let mut authors = self.authors.write();
        for event in events.iter() {
            let Ok(json) = serde_json::from_str::<Value>(&event.content) else {
                continue;
            };
            let Some(author) = authors.get_mut(&event.pubkey) else {
                continue;
            };
            author.about = json["about"].as_str().map(|x| x.to_owned().into());
            author.display_name = json["display_name"].as_str().map(|x| x.to_owned().into());
            author.username = json["name"].as_str().map(|x| x.to_owned().into());
            author.picture = json["picture"]
                .as_str()
                .or_else(|| json["image"].as_str())
                .and_then(|picture| Url::parse(picture).ok());
        }
        Ok(())
    }
}
