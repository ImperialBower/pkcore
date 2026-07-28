//! Extension traits for the kernel's fixed-size hand types (EPIC-80 Phase 3).
//!
//! Rust permits inherent impls only on types the crate defines, so pkcore's
//! domain constructors for `Five`/`Six`/`Seven` live here as extension traits.

use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::board::Board;
use crate::prelude::{Card, Five, PKError, Seven, Six};

pub trait FiveExt {
    #[must_use]
    fn from_2and3(hole_cards: Two, flop: Three) -> Five;
}

impl FiveExt for Five {
    fn from_2and3(hole_cards: Two, flop: Three) -> Five {
        Five::from([
            hole_cards.first(),
            hole_cards.second(),
            flop.first(),
            flop.second(),
            flop.third(),
        ])
    }
}

pub trait SixExt {
    #[must_use]
    fn from_2and3and1(hole_cards: Two, flop: Three, turn: Card) -> Six;
}

impl SixExt for Six {
    fn from_2and3and1(hole_cards: Two, flop: Three, turn: Card) -> Six {
        Six::from([
            hole_cards.first(),
            hole_cards.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            turn,
        ])
    }
}

pub trait SevenExt {
    /// # Errors
    ///
    /// `PKError::InvalidCard` if the case slice holds fewer than two cards.
    fn from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError>;

    /// # Errors
    ///
    /// Infallible in practice; `Result` kept for call-site signature stability.
    fn from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError>;

    /// # Errors
    ///
    /// Infallible in practice; `Result` kept for call-site signature stability.
    fn from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError>;

    #[must_use]
    fn from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven;

    #[must_use]
    fn from_case_and_board(player: &Two, board: &Board) -> Seven;
}

impl SevenExt for Seven {
    fn from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            *case.first().ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
    }

    fn from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            case.first(),
            case.second(),
            case.third(),
            case.forth(),
            case.fifth(),
        ]))
    }

    fn from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            case.first(),
            case.second(),
        ]))
    }

    fn from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven {
        Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            turn,
            case,
        ])
    }

    fn from_case_and_board(player: &Two, board: &Board) -> Seven {
        Seven::from_case_at_turn(*player, board.flop, board.turn, board.river)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ext_tests {
    use super::*;
    use crate::util::data::TestData;
    use std::str::FromStr;

    /// The inherent `Five::from_2and3`/`Seven::from_case_and_board` constructors this test
    /// used to check the extension trait against are gone now that `Five`/`Seven` are kernel
    /// re-exports (EPIC-80 Task 5) — inherent impls on a foreign type are not legal. The
    /// fixed literal below is the exact string these constructors produced before the swap
    /// (`TestData::the_flop()` = `9♣ 6♦ 5♥`, `Two::HAND_6S_6H` = `6♠ 6♥`, and
    /// `Five::from_2and3` lays them out as `[hole.first(), hole.second(), flop.first(),
    /// flop.second(), flop.third()]`), so this still proves the extension trait builds the
    /// same hand pkcore's own domain constructor always did.
    #[test]
    fn five_ext_matches_inherent() {
        assert_eq!(
            <Five as FiveExt>::from_2and3(Two::HAND_6S_6H, TestData::the_flop()),
            Five::from_str("6♠ 6♥ 9♣ 6♦ 5♥").unwrap()
        );
    }

    /// Same shape as `five_ext_matches_inherent`, one card longer: `SixExt::from_2and3and1`
    /// lays hole/flop/turn out as `[hole.first(), hole.second(), flop.first(), flop.second(),
    /// flop.third(), turn]`, so appending `Card::FIVE_SPADES` to `five_ext_matches_inherent`'s
    /// literal (`TestData::the_flop()` = `9♣ 6♦ 5♥`, `Two::HAND_6S_6H` = `6♠ 6♥`) gives the
    /// expected `Six`.
    #[test]
    fn six_ext_from_2and3and1() {
        assert_eq!(
            <Six as SixExt>::from_2and3and1(Two::HAND_6S_6H, TestData::the_flop(), Card::FIVE_SPADES),
            Six::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠").unwrap()
        );
    }

    /// See `five_ext_matches_inherent`'s doc comment: `Seven::from_case_and_board` was
    /// deleted along with the rest of `Seven`'s inherent impl. `seven.rs`'s own
    /// `from_case_and_board` test asserted this exact literal
    /// (`"6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠"`) against the pre-swap inherent method's output, so it's the
    /// source of truth here too.
    #[test]
    fn seven_ext_matches_inherent() {
        let board = TestData::the_hand().board;
        assert_eq!(
            <Seven as SevenExt>::from_case_and_board(&Two::HAND_6S_6H, &board),
            Seven::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap()
        );
    }
}
