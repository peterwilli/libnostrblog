use std::{borrow::Cow, str::FromStr, sync::Arc};

use nostr_sdk::{Event, Kind};
use url::Url;

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
                let title = get_tag_values(&e, "title")
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "No title".into());
                let excerpt = get_tag_values(&e, "summary")
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "No excerpt".into());
                let featured_image = get_tag_values(&e, "image")
                    .first()
                    .map(|x| Url::from_str(x).ok())
                    .unwrap();
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
                    featured_image,
                })
            }
            _ => None,
        })
    }
}
