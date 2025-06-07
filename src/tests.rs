use any_spawner::Executor;
use anyhow::Result;
use nostr_sdk::{Client, FromBech32, Keys, Options, PublicKey};
use once_cell::sync::Lazy;
use test_log::test;
use tokio::{
    sync::OnceCell,
    task::{LocalSet, spawn_local},
};
use tracing::debug;

use crate::{
    Blog,
    blog::{fetch_authors::FetchAuthorsExt, fetch_posts::FetchPostExt, poll_posts::PollPostsExt},
};

static TEST_OWNER_PUBKEY: Lazy<PublicKey> = Lazy::new(|| {
    PublicKey::from_bech32("npub17rfmhcfv2f078kg0jrx4efw7pcj5gxfc3j49syvtf4vn0m58ta5sg00n88")
        .unwrap()
});
static TEST_GLOBAL_CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn get_test_client() -> &'static Client {
    Executor::init_tokio().ok();
    TEST_GLOBAL_CLIENT
        .get_or_init(|| async {
            let keys = Keys::generate();

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
    let client = get_test_client().await;
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
    let client = get_test_client().await;
    let blog = Blog::new(client.clone(), vec![*TEST_OWNER_PUBKEY]);
    blog.fetch_authors().await.unwrap();
    let posts = blog.fetch_posts().await.unwrap();
    debug!("posts: {:?}", posts);
    Ok(())
}
