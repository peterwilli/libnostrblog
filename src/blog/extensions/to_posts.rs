use std::{borrow::Cow, sync::Arc};

use nostr_sdk::{Event, Kind};

use crate::{blog::utils::extract_header, objects::post::Post, types::Authors};

pub trait ToPosts {
    fn to_posts<'a>(self, authors: Authors) -> impl Iterator<Item = Post<'static>>;
}

// Implementation for owned events
impl<I> ToPosts for I
where
    I: Iterator<Item = Event>,
{
    fn to_posts<'a>(self, authors: Authors) -> impl Iterator<Item = Post<'static>> {
        self.filter_map(move |e| match e.kind {
            Kind::TextNote | Kind::LongFormTextNote => {
                let categories: Vec<Cow<'static, str>> = e
                    .tags
                    .iter()
                    .filter_map(|t| {
                        let [k, v] = t.as_slice() else {
                            return None;
                        };
                        if k == "t" {
                            return Some(Cow::Owned(v.to_owned()));
                        }
                        None
                    })
                    .collect();
                let author = authors
                    .read()
                    .get(&e.pubkey)
                    .expect("There should always be an author for a event pubkey here")
                    .clone();
                Some(Post {
                    title: Cow::Owned(extract_header(&e.content).unwrap_or("No title").to_owned()),
                    author,
                    content: Cow::Owned(e.content),
                    created_at: e.created_at,
                    categories,
                })
            }
            _ => None,
        })
    }
}
