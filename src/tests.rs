use crate::{
    Blog, BlogSettings,
    blog::{
        comments::{APPROVED_LABEL, MODERATION_LABEL_NAMESPACE, approval_event_builder},
        fetch_authors::FetchAuthorsExt,
        fetch_posts::FetchPostExt,
        poll_posts::PollPostsExt,
    },
    objects::comment::Comment,
};
use any_spawner::Executor;
use anyhow::Result;
use nostr_sdk::*;
use once_cell::sync::Lazy;
use test_log::test;
use tokio::{sync::OnceCell, task::LocalSet};
use tracing::debug;

static TEST_OWNER_PUBKEY: Lazy<PublicKey> = Lazy::new(|| {
    PublicKey::from_bech32("npub17rfmhcfv2f078kg0jrx4efw7pcj5gxfc3j49syvtf4vn0m58ta5sg00n88")
        .unwrap()
});
static TEST_GLOBAL_CLIENT: OnceCell<Client> = OnceCell::const_new();

fn has_tag(event: &Event, expected: &[&str]) -> bool {
    event.tags.iter().any(|tag| {
        tag.as_slice()
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
    })
}

async fn get_test_client(keys: Option<Keys>) -> &'static Client {
    Executor::init_tokio().ok();
    TEST_GLOBAL_CLIENT
        .get_or_init(|| async {
            let keys = keys.unwrap_or_else(Keys::generate);

            let client = Client::builder()
                .opts(Options::new().gossip(true))
                .signer(keys)
                .build();

            client.add_relay("wss://nos.lol").await.unwrap();
            client.add_relay("wss://relay.nostr.band").await.unwrap();

            client.connect().await;

            client
        })
        .await
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
async fn test_poll_posts() -> Result<()> {
    let client = get_test_client(None).await;
    let blog = Blog::new(client.clone(), vec![*TEST_OWNER_PUBKEY]);
    blog.fetch_authors().await.unwrap();
    let local = LocalSet::new();
    local
        .run_until(async move {
            let mut rx = blog.poll_posts(None).await.unwrap();
            while let Some(post) = rx.recv().await {
                debug!("Post: {:?}", post);
            }
        })
        .await;
    Ok(())
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
async fn test_posts() -> Result<()> {
    let client = get_test_client(None).await;
    let blog = Blog::new(client.clone(), vec![*TEST_OWNER_PUBKEY]);
    blog.fetch_authors().await.unwrap();
    let posts = blog.fetch_posts().await.unwrap();
    debug!("posts: {:?}", posts);
    Ok(())
}

#[test]
fn test_comment_from_nip22_event() -> Result<()> {
    let owner = Keys::generate();
    let commenter = Keys::generate();

    let root = EventBuilder::text_note("Root blog post").sign_with_keys(&owner)?;
    let parent = EventBuilder::comment("Parent comment", &root, Some(&root), None)
        .sign_with_keys(&commenter)?;
    let event = EventBuilder::comment("A moderated comment", &parent, Some(&root), None)
        .sign_with_keys(&commenter)?;

    let comment = Comment::from_event(event, false);
    debug!("comment parsed from NIP-22 event: {:?}", comment);

    assert_eq!(comment.author, commenter.public_key());
    assert_eq!(comment.content, "A moderated comment");
    assert_eq!(comment.root, Some(root.id));
    assert_eq!(comment.parent, Some(parent.id));
    assert!(!comment.approved);

    Ok(())
}

#[test]
fn test_approval_event_builder_creates_signed_nip32_label_event() -> Result<()> {
    let owner = Keys::generate();
    let commenter = Keys::generate();
    let comment = EventBuilder::text_note("Needs approval").sign_with_keys(&commenter)?;

    let approval = approval_event_builder(comment.id).sign_with_keys(&owner)?;
    debug!("approval NIP-32 label event: {:?}", approval);
    debug!("approval event tags: {:?}", approval.tags);

    assert_eq!(approval.kind, Kind::Custom(1985));
    assert_eq!(approval.pubkey, owner.public_key());
    approval.verify()?;

    assert!(has_tag(&approval, &["e", comment.id.to_hex().as_str()]));
    assert!(has_tag(&approval, &["L", MODERATION_LABEL_NAMESPACE]));
    assert!(has_tag(
        &approval,
        &["l", APPROVED_LABEL, MODERATION_LABEL_NAMESPACE]
    ));

    Ok(())
}

#[test]
fn test_comment_preapproval_setting() {
    let owner = Keys::generate();
    let client = Client::builder().build();

    let default_blog = Blog::new(client.clone(), vec![owner.public_key()]);
    debug!(
        "default comment approval setting: {}",
        default_blog.require_comment_approval()
    );
    assert!(!default_blog.require_comment_approval());

    let moderated_blog = Blog::new_with_settings(
        client,
        vec![owner.public_key()],
        BlogSettings {
            require_comment_approval: true,
        },
    );
    debug!(
        "moderated comment approval setting: {}",
        moderated_blog.require_comment_approval()
    );
    assert!(moderated_blog.require_comment_approval());
}

#[cfg(feature = "poster")]
#[test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
async fn test_upload_post() -> Result<()> {
    use crate::blog::poster::{BlogPoster, post::Post};
    use port_selector::random_free_port;
    use std::{process::Command, time::Duration};
    use tokio::time::sleep;

    let server_port = random_free_port().unwrap();
    let mut nak_server = Command::new("nak")
        .arg("serve")
        .arg("--port")
        .arg(format!("{}", server_port))
        .spawn()
        .unwrap();
    sleep(Duration::from_secs(5)).await;

    let post = Post::builder()
        .title("Test".to_string())
        .excerpt("This is a test post".to_owned())
        .featured_image(Url::parse("https://preview.redd.it/5ceb8dpf7j6f1.png").unwrap())
        .contents("This is a test post! We are here to check if everything is fine.".to_owned())
        .pow_difficulty(15)
        .categories(vec!["cooking".to_owned()])
        .build();

    let keys = Keys::generate();
    let client = {
        let client = Client::builder()
            .opts(Options::new().gossip(true))
            .signer(keys.clone())
            .build();
        client
            .add_relay(format!("ws://localhost:{}", server_port))
            .await
            .unwrap();
        client.connect().await;

        client
    };

    let metadata = Metadata::new()
        .name("emerald")
        .display_name("Emerald")
        .about("Test Emerald");
    client.set_metadata(&metadata).await.unwrap();

    let result = BlogPoster::new(client.clone())
        .upload_blog_post(&post)
        .await
        .unwrap();
    println!("blog posted: {}", result);
    // Some time to fetch the post manually
    sleep(Duration::from_secs(1)).await;
    let events = client
        .fetch_events(
            Filter::new().author(keys.public_key()),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    // TODO: why is events empty?
    println!("Events: {:?}", events);

    let blog = Blog::new(client.clone(), vec![keys.public_key()]);
    blog.fetch_authors().await.unwrap();
    let posts = blog.fetch_posts().await.unwrap();
    println!("posts: {:?}", posts);
    nak_server.kill().unwrap();
    Ok(())
}
