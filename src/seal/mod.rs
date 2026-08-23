//! Modules that provides a sealed deck that pkcore cannot read, with the
//! keys, tokens, and scheme owned by the caller, so that shuffling, cutting,
//! burning and dealing all happen blind. Shuffling, cutting and dealing are
//! all permutations, and a permutation needs no knowledge.
//!
//! Wire secrecy is the scheme's job, not this module's. For `PlaintextSeal`, the payload
//! is literally a `Card` that serializes as "A♠".

pub mod card_seal;
// pub mod plaintext;
// pub mod sealed_card;
// pub mod sealed_deck;
pub mod slot;
