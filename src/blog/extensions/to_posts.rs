use std::{borrow::Cow, sync::Arc};

use nostr_sdk::{Event, Kind};

use crate::{blog::utils::get_tag_values, objects::post::Post, types::Authors};

pub trait ToPosts {
    fn to_posts<'a>(self, authors: Authors) -> impl Iterator<Item = Post<'a>>;
}

// Implementation for owned events
impl<I> ToPosts for I
where
    I: Iterator<Item = Event>,
{
    fn to_posts<'a>(self, authors: Authors) -> impl Iterator<Item = Post<'a>> {
        self.filter_map(move |e| match e.kind {
            Kind::TextNote | Kind::LongFormTextNote => {
                let categories = get_tag_values(&e, "t");
                let title = get_tag_values(&e, "name")
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "No title".into());
                let excerpt = get_tag_values(&e, "description")
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "No title".into());
                let author = authors
                    .read()
                    .get(&e.pubkey)
                    .expect("There should always be an author for a event pubkey here")
                    .clone();
                Some(Post {
                    title,
                    author,
                    excerpt,
                    content: Cow::Owned(e.content),
                    created_at: e.created_at,
                    categories,
                })
            }
            _ => None,
        })
    }
}
