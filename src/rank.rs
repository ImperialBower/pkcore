//! pkcore's `Rank` moved to the ckc-rs kernel (EPIC-80). This shim keeps the
//! `crate::rank::Rank` path alive.
pub use ckc_rs::standard52::Rank;
