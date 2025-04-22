use std::{borrow::Cow, sync::Arc};

use nostr_sdk::{Event, Kind};
use serde_json::Value;

use crate::{
    Blog,
    objects::post::{Author, Post},
    types::Authors,
};

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
