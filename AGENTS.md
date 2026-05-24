# Repository Guidelines

## Project Overview

`libnostrblog` is a Rust library for reading and publishing blog content over Nostr. It re-exports `nostr_sdk` from `src/lib.rs`, wraps a Nostr `Client` in `Blog`, and exposes async extension traits for common blog workflows.

- `Blog` owns the shared Nostr client, author cache, category set, post cache, last pull timestamp, and moderation settings.
- Blog posts are built from Nostr text note and long-form note events.
- Authors are fetched from Nostr metadata events and stored in a shared `Authors` map.
- Comments use NIP-22-style comment events plus optional moderation approval labels.
- The optional poster API builds and uploads posts when the `poster` feature is enabled.

## Project Structure

- `src/lib.rs` defines `Blog`, `BlogSettings`, public modules, and the `nostr_sdk` re-export.
- `src/types.rs` contains shared type aliases such as `Authors` and `CheapClonePubkey`.
- `src/objects/` contains serializable domain objects: posts, authors, and comments.
- `src/blog/fetch_authors.rs` fetches owner metadata into the author cache.
- `src/blog/fetch_posts.rs` fetches owner posts from relays and converts them into `Post` values.
- `src/blog/poll_posts.rs` contains post polling behavior.
- `src/blog/comments.rs` contains comment fetching, approval labels, and moderation helpers.
- `src/blog/extensions/` contains conversion and Nostr filter extension traits.
- `src/blog/poster/` contains the post upload API and typed post builder.
- `src/tests.rs` contains crate-level integration-style tests, including tests that contact live relays.

## Build, Test, and Development Commands

- `cargo fmt` formats Rust code.
- `cargo check` validates the crate quickly.
- `cargo test` runs the default test suite. Some tests connect to public Nostr relays, so failures can be caused by network or relay availability.
- `cargo test --features poster` includes poster tests. The upload test expects the `nak` CLI to be installed because it starts `nak serve`.
- `cargo clippy --all-targets --all-features` is useful before larger changes when available locally.

Use targeted tests while iterating, for example `cargo test test_comment_from_nip22_event` or `cargo test --features poster test_upload_post`.

## Coding Style

Use standard Rust style and `rustfmt` defaults. Keep modules, functions, variables, and files in `snake_case`; use `PascalCase` for types and traits. Prefer small extension traits that match the existing API shape, such as `FetchAuthorsExt`, `FetchPostExt`, and `ToPosts`.

Avoid comments wherever possible; instead write self-describing code. Comments should explain unusual protocol behavior, feature-gated constraints, or surprising external integration details.

Prefer this:

```rust
let metadata_events = client
    .fetch_events(Filter::new().metadata_by_owners(owners), timeout)
    .await?;
```

Over this:

```rust
// Fetch events for metadata.
let events = client.fetch_events(filter, timeout).await?;
```

Prefer this:

```rust
fn approval_label_filter(comment_id: EventId) -> Filter {
    Filter::new()
        .kind(Kind::Custom(1985))
        .event(comment_id)
}
```

Over this:

```rust
// Build a filter for approval labels.
fn filter(id: EventId) -> Filter {
    Filter::new().kind(Kind::Custom(1985)).event(id)
}
```

Keep fallible Nostr operations returning `anyhow::Result` where the surrounding code already does. Avoid adding new `unwrap` or `expect` calls in library code unless an invariant is already established nearby and the panic message documents the invariant precisely.

## Data And Lifetime Notes

Domain objects commonly use `Cow<'a, str>` so fetched data can remain flexible between borrowed and owned strings. Preserve those lifetimes unless a caller boundary truly needs owned `String` values.

Author state is shared through `Arc<RwLock<HashMap<Arc<PublicKey>, Author<'static>>>>`. Prefer using the existing `Authors` alias and `CheapClonePubkey` alias instead of spelling the full type in new code.

The crate uses `parking_lot::RwLock` for synchronous shared state around cached blog data. Keep lock scopes short: collect keys or clone required values, then release the lock before network calls or heavier processing.

## Nostr Behavior

- Owner public keys define which authors and posts are fetched.
- Metadata fetches use custom filter extensions in `src/blog/extensions/filters.rs`.
- Post conversion currently accepts `Kind::TextNote` and `Kind::LongFormTextNote`.
- Post tags are parsed through `get_tag_values`; keep tag parsing centralized when adding new post fields.
- Moderated comments are represented with NIP-32 label events using the moderation namespace and approved label constants in `src/blog/comments.rs`.

## Testing Guidelines

Add focused unit tests for pure parsing, conversion, and builder behavior. Prefer generated `Keys` and signed in-memory events for deterministic protocol tests.

Be careful with tests that require live relays. If new behavior can be validated locally, keep it local. For relay-dependent tests, make timeouts explicit and avoid assuming public relays will return stable data.

Before handing off a change, run at least `cargo fmt` and `cargo test` when the network-dependent tests are relevant. If you skip a command because of external dependencies such as `nak` or live relays, mention that clearly.

## Dependency And Feature Notes

Keep dependency additions conservative. This crate already depends on `nostr`, `nostr-sdk`, `tokio`, `async-trait`, `serde`, `typed-builder`, and `url`; prefer those existing tools before adding another crate.

The `poster` feature gates publishing-related usage expectations, even though the module is currently exported unconditionally. Check both default and `--features poster` builds when changing poster code.

## Git Hygiene

Generated files under `target/` are not source and should not be edited. Keep changes scoped to the requested behavior. The worktree may contain user edits; inspect before modifying files and do not revert unrelated changes.
