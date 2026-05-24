use std::{collections::HashSet, time::Duration};

use anyhow::{Result, bail};
use async_trait::async_trait;
use nostr_sdk::{
    Alphabet, Event, EventBuilder, EventId, Filter, Kind, PublicKey, SingleLetterTag, Tag,
    TagStandard, ToBech32,
};

use crate::{Blog, objects::comment::Comment};

pub const MODERATION_LABEL_NAMESPACE: &str = "libnostrblog/moderation";
pub const APPROVED_LABEL: &str = "approved";
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[async_trait(?Send)]
pub trait CommentsExt {
    async fn fetch_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>>;
    async fn fetch_all_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>>;
    async fn list_unapproved_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>>;
    async fn list_all_unapproved_comments<'a>(&self) -> Result<Vec<Comment<'a>>>;
    async fn approve_comment(&self, comment_id: EventId) -> Result<String>;
}

#[async_trait(?Send)]
impl CommentsExt for Blog<'_> {
    async fn fetch_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>> {
        let comments = self.fetch_all_comments(post_id).await?;

        if self.require_comment_approval() {
            Ok(comments.into_iter().filter(|c| c.approved).collect())
        } else {
            Ok(comments)
        }
    }

    async fn fetch_all_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>> {
        let events = fetch_comment_events(self, post_id).await?;
        let comment_ids = events.iter().map(|event| event.id).collect::<Vec<_>>();
        let approved = fetch_approved_comment_ids(self, comment_ids).await?;

        Ok(events
            .into_iter()
            .map(|event| {
                let is_approved = approved.contains(&event.id);
                Comment::from_event(event, is_approved)
            })
            .collect())
    }

    async fn list_unapproved_comments<'a>(&self, post_id: EventId) -> Result<Vec<Comment<'a>>> {
        Ok(self
            .fetch_all_comments(post_id)
            .await?
            .into_iter()
            .filter(|comment| !comment.approved)
            .collect())
    }

    async fn list_all_unapproved_comments<'a>(&self) -> Result<Vec<Comment<'a>>> {
        let events = fetch_all_comment_events(self).await?;
        let comment_ids = events.iter().map(|event| event.id).collect::<Vec<_>>();
        let approved = fetch_approved_comment_ids(self, comment_ids).await?;

        Ok(events
            .into_iter()
            .filter(|event| !approved.contains(&event.id))
            .map(|event| Comment::from_event(event, false))
            .collect())
    }

    async fn approve_comment(&self, comment_id: EventId) -> Result<String> {
        let signer = self.client.signer().await?;
        let signer_pubkey = signer.get_public_key().await?;

        if !self.owner_public_keys().contains(&signer_pubkey) {
            bail!("comment moderation events must be signed by one of the configured owner keys");
        }

        let builder = approval_event_builder(comment_id);
        let event_id = self.client.send_event_builder(builder).await?;
        Ok(event_id.to_bech32()?)
    }
}

pub fn approval_event_builder(comment_id: EventId) -> EventBuilder {
    EventBuilder::new(Kind::Custom(1985), "")
        .tag(Tag::event(comment_id))
        .tag(Tag::from_standardized_without_cell(
            TagStandard::LabelNamespace(MODERATION_LABEL_NAMESPACE.to_owned()),
        ))
        .tag(Tag::from_standardized_without_cell(TagStandard::Label {
            value: APPROVED_LABEL.to_owned(),
            namespace: Some(MODERATION_LABEL_NAMESPACE.to_owned()),
        }))
}

async fn fetch_comment_events(blog: &Blog<'_>, post_id: EventId) -> Result<Vec<Event>> {
    let lowercase_events = blog
        .client
        .fetch_events(
            Filter::new().kind(Kind::Comment).event(post_id),
            FETCH_TIMEOUT,
        )
        .await?;
    let uppercase_events = blog
        .client
        .fetch_events(
            Filter::new()
                .kind(Kind::Comment)
                .custom_tag(SingleLetterTag::uppercase(Alphabet::E), post_id),
            FETCH_TIMEOUT,
        )
        .await?;

    let mut seen = HashSet::new();
    Ok(lowercase_events
        .into_iter()
        .chain(uppercase_events.into_iter())
        .filter(|event| seen.insert(event.id))
        .collect())
}

async fn fetch_all_comment_events(blog: &Blog<'_>) -> Result<Vec<Event>> {
    Ok(blog
        .client
        .fetch_events(Filter::new().kind(Kind::Comment), FETCH_TIMEOUT)
        .await?
        .into_iter()
        .collect())
}

async fn fetch_approved_comment_ids(
    blog: &Blog<'_>,
    comment_ids: Vec<EventId>,
) -> Result<HashSet<EventId>> {
    if comment_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let owners = blog.owner_public_keys();
    if owners.is_empty() {
        return Ok(HashSet::new());
    }

    let events = blog
        .client
        .fetch_events(
            Filter::new()
                .kind(Kind::Custom(1985))
                .authors(owners.clone())
                .events(comment_ids)
                .custom_tag(SingleLetterTag::lowercase(Alphabet::L), APPROVED_LABEL),
            FETCH_TIMEOUT,
        )
        .await?;
    let owner_set = owners.into_iter().collect::<HashSet<PublicKey>>();

    Ok(events
        .into_iter()
        .filter(|event| owner_set.contains(&event.pubkey) && has_approved_label(event))
        .filter_map(|event| approved_comment_id(&event))
        .collect())
}

fn has_approved_label(event: &Event) -> bool {
    event.tags.iter().any(|tag| {
        let values = tag.as_slice();
        if values.len() < 2 || values[0] != "l" || values[1] != APPROVED_LABEL {
            return false;
        }

        values
            .get(2)
            .map(|namespace| namespace == MODERATION_LABEL_NAMESPACE)
            .unwrap_or(true)
    })
}

fn approved_comment_id(event: &Event) -> Option<EventId> {
    event.tags.iter().find_map(|tag| {
        let [name, value, ..] = tag.as_slice() else {
            return None;
        };

        if name == "e" {
            EventId::from_hex(value).ok()
        } else {
            None
        }
    })
}
