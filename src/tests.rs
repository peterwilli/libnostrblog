use crate::{
    Blog,
    blog::{fetch_authors::FetchAuthorsExt, fetch_posts::FetchPostExt, poll_posts::PollPostsExt},
};
use any_spawner::Executor;
use anyhow::Result;
use nostr_sdk::*;
use once_cell::sync::Lazy;
use test_log::test;
use tokio::{
    sync::OnceCell,
    task::{LocalSet, spawn_local},
};
use tracing::debug;

static TEST_OWNER_PUBKEY: Lazy<PublicKey> = Lazy::new(|| {
    PublicKey::from_bech32("npub17rfmhcfv2f078kg0jrx4efw7pcj5gxfc3j49syvtf4vn0m58ta5sg00n88")
        .unwrap()
});
static TEST_GLOBAL_CLIENT: OnceCell<Client> = OnceCell::const_new();

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
