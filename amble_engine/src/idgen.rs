//! Deterministic UUID generation for world objects.
//!
//! Provides namespaced v5 UUID helpers so content authored in TOML maps to
//! stable identifiers at runtime. No transient/ephemeral object generation is yet supported,
//! but they would use v4 UUIDs if implemented. 
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_from_token_generates_consistent_uuids() {
        let token = "test_item";
        let uuid1 = uuid_from_token(&NAMESPACE_ITEM, token);
        let uuid2 = uuid_from_token(&NAMESPACE_ITEM, token);

        // Same token and namespace should generate identical UUIDs
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn uuid_from_token_different_namespaces_generate_different_uuids() {
        let token = "test";
        let room_uuid = uuid_from_token(&NAMESPACE_ROOM, token);
        let item_uuid = uuid_from_token(&NAMESPACE_ITEM, token);
        let character_uuid = uuid_from_token(&NAMESPACE_CHARACTER, token);

        // Same token with different namespaces should generate different UUIDs
        assert_ne!(room_uuid, item_uuid);
        assert_ne!(item_uuid, character_uuid);
        assert_ne!(room_uuid, character_uuid);
    }

    #[test]
    fn uuid_from_token_different_tokens_generate_different_uuids() {
        let uuid1 = uuid_from_token(&NAMESPACE_ITEM, "token1");
        let uuid2 = uuid_from_token(&NAMESPACE_ITEM, "token2");

        // Different tokens with same namespace should generate different UUIDs
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn uuid_from_token_empty_string() {
        let uuid = uuid_from_token(&NAMESPACE_ITEM, "");

        // Empty string should still generate a valid UUID
        assert!(!uuid.is_nil());
    }

    #[test]
    fn uuid_from_token_special_characters() {
        let token = "test-item_with.special@chars#123";
        let uuid = uuid_from_token(&NAMESPACE_ITEM, token);

        // Special characters should not cause issues
        assert!(!uuid.is_nil());

        // Should be consistent
        let uuid2 = uuid_from_token(&NAMESPACE_ITEM, token);
        assert_eq!(uuid, uuid2);
    }

    #[test]
    fn namespaces_are_valid_uuids() {
        // All namespaces should be valid UUIDs
        assert!(!NAMESPACE_ROOM.is_nil());
        assert!(!NAMESPACE_ITEM.is_nil());
        assert!(!NAMESPACE_CHARACTER.is_nil());
    }

    #[test]
    fn namespaces_are_different() {
        // All namespaces should be different from each other
        assert_ne!(NAMESPACE_ROOM, NAMESPACE_ITEM);
        assert_ne!(NAMESPACE_ITEM, NAMESPACE_CHARACTER);
        assert_ne!(NAMESPACE_ROOM, NAMESPACE_CHARACTER);
    }

    #[test]
    fn generated_uuids_are_version_5() {
        let uuid = uuid_from_token(&NAMESPACE_ITEM, "test");

        // Version 5 UUIDs should have version bits set to 5
        assert_eq!(uuid.get_version_num(), 5);
    }

    #[test]
    fn unicode_tokens_work() {
        let token = "тест_предмет";
        let uuid1 = uuid_from_token(&NAMESPACE_ITEM, token);
        let uuid2 = uuid_from_token(&NAMESPACE_ITEM, token);

        // Unicode tokens should work consistently
        assert_eq!(uuid1, uuid2);
        assert!(!uuid1.is_nil());
    }

    #[test]
    fn very_long_tokens_work() {
        let long_token = "a".repeat(1000);
        let uuid = uuid_from_token(&NAMESPACE_ITEM, &long_token);

        // Very long tokens should still work
        assert!(!uuid.is_nil());

        // Should be consistent
        let uuid2 = uuid_from_token(&NAMESPACE_ITEM, &long_token);
        assert_eq!(uuid, uuid2);
    }
}
