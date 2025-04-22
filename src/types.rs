use crate::objects::post::Author;
use nostr_sdk::PublicKey;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

pub type CheapClonePubkey = Arc<PublicKey>;
pub type Authors = Arc<RwLock<HashMap<CheapClonePubkey, Author<'static>>>>;
