pub mod post;
pub use nostr_sdk::*;
use post::Post;

pub struct BlogPoster {
    client: Client,
}

impl BlogPoster {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn upload_blog_post(&self, post: &Post) -> Result<String> {
        let event_id = self
            .client
            .send_event_builder(blog_post_event_builder(post))
            .await?;
        Ok(event_id.to_bech32()?)
    }

    pub async fn upload_comment(
        &self,
        content: &str,
        comment_to: &Event,
        root: Option<&Event>,
        relay_url: Option<RelayUrl>,
    ) -> Result<String> {
        let event_id = self
            .client
            .send_event_builder(EventBuilder::comment(content, comment_to, root, relay_url))
            .await?;
        Ok(event_id.to_bech32()?)
    }
}

fn blog_post_event_builder(post: &Post) -> EventBuilder {
    let published_at = Timestamp::now().as_u64().to_string();
    let identifier = format!("{}-{published_at}", title_slug(&post.title));
    let mut builder = EventBuilder::long_form_text_note(post.contents.as_str());
    builder = builder.tag(Tag::title(post.title.as_str()));
    builder = builder.tag(Tag::identifier(identifier));
    builder = builder.tag(Tag::parse(["published_at", published_at.as_str()]).expect("valid tag"));
    if let Some(image_url) = post.featured_image.as_ref() {
        builder = builder.tag(Tag::image(image_url.clone(), None))
    }
    for category in post.categories.iter() {
        builder = builder.tag(Tag::hashtag(category));
    }
    builder = builder.tag(Tag::from_standardized_without_cell(TagStandard::Summary(
        post.excerpt.to_owned(),
    )));
    if let Some(difficulty) = post.pow_difficulty {
        builder = builder.pow(difficulty);
    }
    builder
}

fn title_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut needs_separator = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if needs_separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(character);
            needs_separator = false;
        } else if !slug.is_empty() {
            needs_separator = true;
        }
    }

    if slug.is_empty() {
        "post".to_owned()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use nostr_sdk::{Keys, Kind};

    use super::{blog_post_event_builder, post::Post};

    #[test]
    fn blog_post_event_builder_creates_long_form_text_note() -> Result<()> {
        let post = Post::builder()
            .title("Title".to_owned())
            .excerpt("Excerpt".to_owned())
            .contents("Content".to_owned())
            .categories(vec!["blog".to_owned()])
            .build();

        let event = blog_post_event_builder(&post).sign_with_keys(&Keys::generate())?;

        assert_eq!(event.kind, Kind::LongFormTextNote);
        assert!(event.tags.iter().any(|tag| {
            let [key, value, ..] = tag.as_slice() else {
                return false;
            };
            key == "d" && value.starts_with("title-")
        }));
        Ok(())
    }
}
