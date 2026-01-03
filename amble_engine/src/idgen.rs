//! Deterministic ID helpers for world objects.
//!
//! IDs are now the string symbols from content files, so token -> ID is
//! a simple identity mapping. Namespaces remain for compatibility with
//! older loader call sites.

use crate::Id;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const NAMESPACE_ROOM: &str = "room";
pub const NAMESPACE_ITEM: &str = "item";
pub const NAMESPACE_CHARACTER: &str = "character";

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

/// Generate a unique, human-readable id for tests and tooling.
pub fn new_id() -> Id {
    let next = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("id_{next}")
}

/// Convert a token id from content files into a runtime ID.
pub fn uuid_from_token(namespace: impl AsRef<str>, token: &str) -> Id {
    let _ = namespace.as_ref();
    token.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_from_token_is_identity_mapping() {
        let token = "test_item";
        let id = uuid_from_token(NAMESPACE_ITEM, token);
        assert_eq!(id, token);
    }
}
