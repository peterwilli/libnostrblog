# libnostrblog

Rust library for reading, streaming, moderating, and publishing blog content over Nostr.

`libnostrblog` wraps a `nostr_sdk::Client` in a small blog-oriented API. You give it one or more owner public keys, and it fetches long-form posts, author metadata, NIP-22 comments, deletion events, and optional moderation labels from the configured relays. It is the Nostr backend used by [Emerald Blog](https://github.com/peterwilli/emerald-blog), but it is written as a reusable crate for other Nostr-backed blogs.

## Features

- **Nostr-backed posts**: fetches owner-authored long-form text notes and converts them into serializable `Post` values with title, excerpt, content, categories, author data, timestamps, and featured images.
- **Author metadata**: resolves configured blog owners from Nostr metadata events, including names, display names, avatars, and bios.
- **Live post streaming**: subscribes to relay events and yields new posts through a Tokio channel for hydrated apps or background sync tasks.
- **Deletion-aware fetching**: fetches Nostr deletion events alongside posts and filters posts deleted by their original author.
- **NIP-22 comments**: fetches comments for a post, parses root and parent relationships, and exposes serializable `Comment` values.
- **Moderation support**: can require approval before comments appear, using NIP-32 label events in the `libnostrblog/moderation` namespace.
- **Publishing helpers**: the poster API builds and uploads long-form blog posts and comments from structured Rust values.
- **`nostr_sdk` re-export**: consumers can use Nostr SDK types directly from `libnostrblog` without adding a second import path.

## Prerequisites

Install a recent Rust toolchain:

```bash
rustup toolchain install stable
```

The crate uses Rust edition 2024. If your stable toolchain is too old for edition 2024, update it:

```bash
rustup update stable
```

The poster test starts a local relay with `nak`, so install `nak` before running poster-feature tests that upload posts.

## Installation

Add the crate to a Rust project from a local path:

```toml
[dependencies]
libnostrblog = { path = "../libnostrblog" }
```

Enable publishing helpers when you want the poster API:

```toml
[dependencies]
libnostrblog = { path = "../libnostrblog", features = ["poster"] }
```

## Basic Usage

Create a Nostr client, connect it to relays, then build a `Blog` with the public keys that own the blog.

```rust
use anyhow::Result;
use libnostrblog::{
    Blog, Client, Options, PublicKey, ToBech32,
    blog::{fetch_authors::FetchAuthorsExt, fetch_posts::FetchPostExt},
};

#[tokio::main]
async fn main() -> Result<()> {
    let owner = PublicKey::parse(
        "npub17rfmhcfv2f078kg0jrx4efw7pcj5gxfc3j49syvtf4vn0m58ta5sg00n88",
    )?;

    let client = Client::builder()
        .opts(Options::new().gossip(true))
        .build();

    client.add_relay("wss://nos.lol").await?;
    client.add_relay("wss://relay.nostr.band").await?;
    client.connect().await;

    let blog = Blog::new(client, vec![owner]);

    blog.fetch_authors().await?;
    let posts = blog.fetch_posts(None).await?;

    for post in posts {
        println!("{} {}", post.id.to_bech32()?, post.title);
    }

    Ok(())
}
```

## Comments And Moderation

By default, `fetch_comments` returns all comments for a post. Set `require_comment_approval` to only return comments approved by a configured owner key.

```rust
use anyhow::Result;
use libnostrblog::{
    Blog, BlogSettings, Client, EventId, PublicKey,
    blog::comments::CommentsExt,
};

async fn approved_comments(
    client: Client,
    owner: PublicKey,
    post_id: EventId,
) -> Result<()> {
    let blog = Blog::new_with_settings(
        client,
        vec![owner],
        BlogSettings {
            require_comment_approval: true,
        },
    );

    let comments = blog.fetch_comments(post_id).await?;

    for comment in comments {
        println!("{}", comment.content);
    }

    Ok(())
}
```

Approval events must be signed by one of the blog owner keys:

```rust
use anyhow::Result;
use libnostrblog::{Blog, EventId, blog::comments::CommentsExt};

async fn approve(blog: Blog<'_>, comment_id: EventId, post_id: EventId) -> Result<String> {
    blog.approve_comment_for_post(comment_id, post_id).await
}
```

## Publishing

Use `BlogPoster` to publish a long-form Nostr post from structured data:

```rust
use anyhow::Result;
use libnostrblog::blog::poster::{BlogPoster, Client, post::Post};

async fn publish(client: Client) -> Result<String> {
    let post = Post::builder()
        .title("Hello from Nostr".to_owned())
        .excerpt("A short summary for previews.".to_owned())
        .contents("The full Markdown body of the post.".to_owned())
        .categories(vec!["blog".to_owned(), "nostr".to_owned()])
        .build();

    BlogPoster::new(client).upload_blog_post(&post).await
}
```

Posts are published as Nostr long-form text notes with title, identifier, summary, published timestamp, optional image, optional proof-of-work difficulty, and hashtag category tags.

## Development

Format and check the crate:

```bash
cargo fmt
cargo check
```

Run the default tests:

```bash
cargo test
```

Some tests connect to public Nostr relays, so relay availability can affect results.

Run poster-feature tests:

```bash
cargo test --features poster
```

The upload test expects the `nak` CLI because it starts `nak serve` as a local relay.

