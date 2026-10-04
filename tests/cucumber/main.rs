//! Gherkin acceptance suite for pkcore.
//!
//! The `.feature` files under `tests/features/` describe pkcore's behaviour in
//! the language of the table — cards, hands, ranges, equity, betting and
//! showdowns — and the step modules here bind each sentence to the kernel's
//! public API. Nothing in this suite reaches past `pkcore::prelude` and the
//! `analysis::equity` / `games::omaha` / `casino` modules a consumer would use.
//!
//! Run it on its own with:
//!
//! ```text
//! cargo test --test cucumber
//! cargo test --test cucumber -- --name "side pot"     # filter scenarios
//! cargo test --test cucumber -- --tags @equity        # filter by tag
//! ```
//!
//! The suite runs on default features: it exercises the pure kernel, so it
//! needs no YAML, file I/O or thread pool.

// A failed `expect` *is* a failed step: cucumber-rs reports the panic message
// against the sentence that raised it. `allow-expect-in-tests` in clippy.toml
// only covers `#[test]` functions, and this harness has none. Step functions
// take their captures by value because cucumber-rs parses them into owned
// types.
#![allow(clippy::expect_used, clippy::needless_pass_by_value)]

mod steps;
mod world;

use cucumber::World as _;
use world::PokerWorld;

fn main() {
    let features = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features");
    futures::executor::block_on(
        PokerWorld::cucumber()
            // An unbound sentence is a bug in the suite, not a pending test.
            .fail_on_skipped()
            .run_and_exit(features),
    );
}
