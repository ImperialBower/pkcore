use serde::{Deserialize, Serialize};

pub mod rank_mask;
pub mod suit_mask;
pub mod suit_texture;

/// The rank and suit bit masks of a hand.
///
/// # Domain
///
/// - **Role:** value object
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mask {
    pub rank_mask: u32,
    pub suit_mask: u32,
}

/// Reduces a hand to its rank and suit bit masks.
///
/// # Domain
///
/// - **Role:** cohesive mechanism
pub trait Masked {
    fn rank_mask(&self) -> u32;
    fn suit_mask(&self) -> u32;
    fn get_make(&self) -> Mask {
        Mask {
            rank_mask: self.rank_mask(),
            suit_mask: self.suit_mask(),
        }
    }
}
