use std::borrow::Cow;

use nostr_sdk::{Event, Kind};
use serde_json::Value;

use crate::objects::post::{Author, Post};

pub trait ToPosts<'a> {
    fn to_posts(self) -> impl Iterator<Item = Post<'a>>;
}

impl<'a, 'e, I> ToPosts<'a> for I
where
    I: Iterator<Item = &'e Event>,
{
    fn to_posts(mut self) -> impl Iterator<Item = Post<'a>> {
        // First get author information
        let author = self
            .find(|e| e.kind == Kind::Metadata)
            .map(|x| {
                let json: Value = serde_json::from_str(&x.content).unwrap();
                Author {
                    username: json["name"].as_str().map(|x| x.to_owned().into()),
                    display_name: json["display_name"].as_str().map(|x| x.to_owned().into()),
                }
            })
            .unwrap_or_default();
        self.filter_map(move |e| match e.kind {
            Kind::TextNote | Kind::LongFormTextNote => {
                let categories: Vec<Cow<'_, str>> = e
                    .tags
                    .iter()
                    .filter_map(|t| {
                        let [k, v] = t.as_slice() else {
                            return None;
                        };
                        if k == "t" {
                            return Some(v.into());
                        }
                        None
                    })
                    .collect();
                Some(Post {
                    author: author.clone(),
                    content: e.content.to_owned().into(),
                    created_at: e.created_at,
                    categories: vec![], // categories: e.tags.find(nostr_sdk::TagKind::Sub),
                })
            }
            _ => None,
        })
    }
}
