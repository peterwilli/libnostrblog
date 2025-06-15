use typed_builder::TypedBuilder;
use url::Url;

#[derive(Debug, TypedBuilder)]
pub struct Post {
    pub title: String,
    pub excerpt: String,
    #[builder(default, setter(strip_option))]
    pub featured_image: Option<Url>,
    pub contents: String,
    pub categories: Vec<String>,
    #[builder(default, setter(strip_option))]
    pub pow_difficulty: Option<u8>,
}
