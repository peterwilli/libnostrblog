use crate::Blog;
use nostr_sdk::{Filter, Kind};

pub trait PostsFiltersExt {
    fn owner_filter(&self) -> Filter;
}

impl PostsFiltersExt for Blog<'_> {
    fn owner_filter(&self) -> Filter {
        Filter::new().author(self.owner).kinds([
            Kind::Metadata,
            Kind::TextNote,
            Kind::LongFormTextNote,
        ])
    }
}
