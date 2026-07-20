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
mod casino__principal_tests {
    use super::*;
}
