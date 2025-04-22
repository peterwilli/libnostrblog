use nostr_sdk::{Filter, Kind, PublicKey};

pub trait FiltersExt {
    fn posts_by_owner(self, pubkey: PublicKey) -> Filter;
    fn metadata_by_owner(self, pubkey: PublicKey) -> Filter;
    fn posts_by_owners(self, pubkeys: Vec<PublicKey>) -> Filter;
    fn metadata_by_owners(self, pubkeys: Vec<PublicKey>) -> Filter;
}

impl FiltersExt for Filter {
    fn posts_by_owner(self, pubkey: PublicKey) -> Filter {
        self.author(pubkey)
            .kinds([Kind::TextNote, Kind::LongFormTextNote])
    }

    fn posts_by_owners(self, pubkeys: Vec<PublicKey>) -> Filter {
        self.authors(pubkeys)
            .kinds([Kind::TextNote, Kind::LongFormTextNote])
    }

    fn metadata_by_owner(self, pubkey: PublicKey) -> Filter {
        self.author(pubkey).kinds([Kind::Metadata])
    }

    fn metadata_by_owners(self, pubkeys: Vec<PublicKey>) -> Filter {
        self.authors(pubkeys).kinds([Kind::Metadata])
    }
}
