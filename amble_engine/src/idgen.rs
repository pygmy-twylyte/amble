//! ** idgen module **
//! Contains namespaces and helpers for generating stable v5 uuids for loaded `WorldObjects`.
//! Dynamically generated / named `WorldObjects` use v4 (random) UUIDs instead.
use std::sync::LazyLock;
use uuid::Uuid;

// Note: can't use "from_bytes" or "parse_str" in const expressions, so we use lazily
// evaluated static variables instead.
pub static NAMESPACE_ROOM: LazyLock<Uuid> =
    LazyLock::new(|| Uuid::parse_str("26dc1968-c645-4f5c-915a-400e06bd361c").unwrap());

pub static NAMESPACE_ITEM: LazyLock<Uuid> =
    LazyLock::new(|| Uuid::parse_str("47d2aad8-22cc-4dd4-bf9f-b9eddc4fe2cf").unwrap());

pub static NAMESPACE_CHARACTER: LazyLock<Uuid> =
    LazyLock::new(|| Uuid::parse_str("99897a5d-4297-4bdc-832e-29df86925063").unwrap());

pub fn uuid_from_token(namespace: &Uuid, token: &str) -> Uuid {
    Uuid::new_v5(namespace, token.as_bytes())
}
