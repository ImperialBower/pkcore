use serde::{Deserialize, Serialize};

pub mod rank_mask;
pub mod suit_mask;
pub mod suit_texture;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mask {
    pub rank_mask: u32,
    pub suit_mask: u32,
}

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
