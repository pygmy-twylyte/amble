//! ** idgen module **
//! Contains namespaces and helpers for generating stable v5 uuids for loaded `WorldObjects`.
//! Dynamically generated / named `WorldObjects` use v4 (random) UUIDs instead.
use uuid::Uuid;

pub const NAMESPACE_ROOM: Uuid = uuid::uuid!("26dc1968-c645-4f5c-915a-400e06bd361c");

pub const NAMESPACE_ITEM: Uuid = uuid::uuid!("47d2aad8-22cc-4dd4-bf9f-b9eddc4fe2cf");

pub const NAMESPACE_CHARACTER: Uuid = uuid::uuid!("99897a5d-4297-4bdc-832e-29df86925063");

/// Generate a v5 UUID for a given token id from the TOML data files.
///
/// Uses the namespaces above to separate rooms / items / characters.
pub fn uuid_from_token(namespace: &Uuid, token: &str) -> Uuid {
    Uuid::new_v5(namespace, token.as_bytes())
}
