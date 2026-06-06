use std::{borrow::Cow, str::FromStr, sync::Arc};

use nostr_sdk::{Event, Kind};
use url::Url;

use crate::{
    blog::utils::get_tag_values,
    objects::post::{Author, Post},
    types::Authors,
};

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
            Kind::LongFormTextNote => {
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
                    .and_then(|x| Url::from_str(x).ok());
                let author = authors
                    .read()
                    .get(&e.pubkey)
                    .cloned()
                    .unwrap_or_else(|| Author::from_pubkey(Arc::new(e.pubkey)));

                Some(Post {
                    id: e.id,
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use anyhow::Result;
    use nostr_sdk::{EventBuilder, Keys, Tag};
    use parking_lot::RwLock;

    use super::ToPosts;
    use crate::{objects::post::Author, types::Authors};

    fn authors_for(keys: &Keys) -> Authors {
        let pubkey = Arc::new(keys.public_key());
        Arc::new(RwLock::new(HashMap::from([(
            pubkey.clone(),
            Author::from_pubkey(pubkey),
        )])))
    }

    #[test]
    fn long_form_text_note_becomes_post() -> Result<()> {
        let keys = Keys::generate();
        let event = EventBuilder::long_form_text_note("content")
            .tag(Tag::title("Title"))
            .sign_with_keys(&keys)?;

        let post = [event].into_iter().to_posts(authors_for(&keys)).next();

        assert!(post.is_some());
        assert_eq!(post.unwrap().title, "Title");
        Ok(())
    }

    #[test]
    fn long_form_text_note_with_invalid_image_url_becomes_post_without_featured_image() -> Result<()>
    {
        let keys = Keys::generate();
        let event = EventBuilder::long_form_text_note("content")
            .tag(Tag::title("Title"))
            .tag(Tag::parse(["image", "not a url"])?)
            .sign_with_keys(&keys)?;

        let post = [event].into_iter().to_posts(authors_for(&keys)).next();

        assert!(post.is_some());
        assert_eq!(post.unwrap().featured_image, None);
        Ok(())
    }

    #[test]
    fn long_form_text_note_without_prefetched_author_uses_pubkey_fallback() -> Result<()> {
        let keys = Keys::generate();
        let event = EventBuilder::long_form_text_note("content").sign_with_keys(&keys)?;
        let authors = Arc::new(RwLock::new(HashMap::new()));

        let post = [event].into_iter().to_posts(authors).next().unwrap();

        assert_eq!(*post.author.pubkey, keys.public_key());
        assert_eq!(post.author.username, None);
        assert_eq!(post.author.display_name, None);
        assert_eq!(post.author.picture, None);
        Ok(())
    }
}
