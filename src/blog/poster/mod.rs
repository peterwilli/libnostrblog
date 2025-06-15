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
        let mut builder = EventBuilder::text_note(post.contents.as_str());
        builder = builder.tag(Tag::title(post.title.as_str()));
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
        let event_id = self.client.send_event_builder(builder).await?;
        Ok(event_id.to_bech32()?)
    }
}
