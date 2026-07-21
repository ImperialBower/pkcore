//! The identity of an actor, as far as the domain kernel is concerned.
//!
//! A [`Principal`] names *who* is acting; it says nothing about *how* they
//! proved it. Authentication: tokens, claims, signatures; happens at the
//! transport edge (the `pkgate` gateway, EPIC-50–53), which resolves a
//! credential to a `Principal` and passes only that inward. Constructing one
//! verifies nothing.
//!
//! The wrapped value is the same [`Uuid`] that already identifies a
//! [`Player`](crate::casino::player::Player) and keys `StatsRegistry`, so a
//! `Principal` drops into the existing seating, stats, and hand-history
//! machinery without a second identity space.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the identity of an actor within the system. Wraps `uuids` generated
/// by entities such as `Player::id` and `StatsRegistry`'s `HashMap<Uuid, PlayerStats>`
///
/// ```
/// use uuid::Uuid;
/// use pkcore::casino::principal::Principal;
///
/// let id = Uuid::new_v4();
/// let principal = Principal::new(id);
///
/// assert_eq!(principal.id(), id);
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub struct Principal(pub Uuid);

impl Principal {
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Principal(id)
    }

    #[must_use]
    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for Principal {
    fn from(value: Uuid) -> Self {
        Principal(value)
    }
}

impl From<Principal> for Uuid {
    fn from(value: Principal) -> Self {
        value.0
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::unwrap_used)]
mod casino__principal_tests {
    use super::*;

    #[test]
    fn principal_round_trips_uuid() {
        let id = Uuid::new_v4();

        let principal = Principal::from(id);
        let round_trip_id = Uuid::from(principal);

        assert_eq!(id, round_trip_id);
    }

    #[test]
    fn principal_serde_round_trip() {
        let id = Uuid::new_v4();
        let principal = Principal::new(id);

        let serialized = serde_json::to_string(&principal).unwrap();
        let deserialized: Principal = serde_json::from_str(&serialized).unwrap();

        assert_eq!(principal, deserialized);
        // A one-field tuple struct serializes through `serialize_newtype_struct`,
        // which serde_json emits as the bare inner value. The wire form of a
        // `Principal` is therefore indistinguishable from that of its `Uuid`.
        assert_eq!(serialized, serde_json::to_string(&id).unwrap());
    }

    #[test]
    fn principal_hashes_as_uuid() {
        use std::collections::HashMap;

        let id = Uuid::new_v4();
        let principal = Principal::new(id);

        // The shape of `StatsRegistry`'s player map.
        let mut registry: HashMap<Uuid, u32> = HashMap::new();
        registry.insert(id, 42);

        assert_eq!(registry.get(&principal.id()), Some(&42));
        assert_eq!(registry.get(&Uuid::from(principal)), Some(&42));

        // Hash and Eq agree with each other, not merely compile.
        let mut principals: HashMap<Principal, u32> = HashMap::new();
        principals.insert(principal, 1);
        principals.insert(Principal::from(id), 2);

        assert_eq!(principals.len(), 1);
        assert_eq!(principals.get(&principal), Some(&2));
    }
}
