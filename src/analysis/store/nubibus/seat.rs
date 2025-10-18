use crate::analysis::store::nubibus::chips::Chips;
use crate::arrays::two::Two;
use crate::play::Position6Max;
use crate::{Betting, PKError, Pile};
use std::cell::Cell;
use std::fmt::{Display, Formatter};

/// One important distinction between chips represented with the Chips struct as compared
/// to a simple `usize` integer is that the Chips struct represents the control of the
/// value, were as an integer is only providing a reference without implying control.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Seat {
    pub position: Position6Max,
    pub name: String,
    pub is_active: Cell<bool>,
    pub stack: Cell<Chips>,
    pub chips_in_play: Cell<Chips>,
    pub chips_in_pot: Cell<usize>,
    pub hand: Cell<Two>,
}

impl Seat {
    #[must_use]
    pub fn chips_in_play_size(&self) -> usize {
        self.chips_in_play.get().size()
    }

    #[must_use]
    pub fn chips_in_pot_size(&self) -> usize {
        self.chips_in_pot.get()
    }

    #[must_use]
    pub fn new(name: String, seat: Position6Max, starting_chips: Chips) -> Seat {
        Seat {
            position: seat,
            name,
            is_active: Cell::new(true),
            stack: Cell::new(starting_chips),
            chips_in_play: Cell::new(Chips::default()),
            chips_in_pot: Cell::new(0),
            hand: Cell::new(Two::default()),
        }
    }

    /// Returns a count of the players remaining `Chips` if successful.
    ///
    /// Betting from a `Chips'` perspective is simple. You simply subtract the amount that
    /// you are betting. But for a `Seat`, it is a complex dance.
    ///
    /// * `Chips` are removed from the players stack
    /// * Those `Chips` are added to the chips in play.
    /// * When the betting round is over, those chips are moved to the `Table` and the amount is added to `chips_in_pot`.
    ///
    /// # Panics
    ///
    /// For Pluribus processing, it should never have a chips in play value that's greater than the amount bet.
    pub fn bet(&self, amount: usize) -> Option<usize> {
        // This logic would need to be different for a non pluribus module. I am debating if the code should
        // be generic.
        let Some(diff) = amount.checked_sub(self.chips_in_play.get().size()) else {
            panic!(
                "amount: {} is less than chips_in_play: {}",
                amount,
                self.chips_in_play.get().size()
            )
        };

        self.stack.get().size().checked_sub(diff).inspect(|&new_stack_size| {
            // Remove the `Chips` from the stack
            self.stack.set(Chips::new(new_stack_size));
            // Pluribus passes in the total chips in play for the round, so just set that.
            self.chips_in_play.set(Chips::new(amount));
            // Return the player's remaining Chips.
        })
    }

    /// # Errors
    ///
    /// Throws a `PKError::AlreadyDealt` if the player is already holding a hand.
    pub fn dealt(&self, hand: Two) -> Result<(), PKError> {
        if self.is_dealt() {
            Err(PKError::AlreadyDealt)
        } else {
            self.hand.set(hand);
            Ok(())
        }
    }

    pub fn desc(&self) -> String {
        format!("{}({})", self.name, self.position,)
    }

    pub fn end_round(&self) -> Chips {
        let into_pot = self.chips_in_play.get();
        self.chips_in_play.set(Chips::default());
        self.chips_in_pot.set(self.chips_in_pot.get() + into_pot.size());
        into_pot
    }

    pub fn fold(&self) -> (Two, usize) {
        // Throw hand into the muck
        let discarded = self.hand.get();
        let chips_lost_in_round = self.chips_in_play.get().size();

        self.hand.set(Two::default());
        self.end_round();
        self.is_active.set(false);

        (discarded, chips_lost_in_round)
    }

    pub fn holding(&self) -> Two {
        self.hand.get()
    }

    pub fn is_active(&self) -> bool {
        self.is_active.get()
    }

    pub fn is_dealt(&self) -> bool {
        self.hand.get().is_dealt()
    }

    pub fn stack_size(&self) -> usize {
        self.stack.get().size()
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SEAT #{} {:>4}\t{:>8}:\t{}\t{} IN PLAY: {}\tINTO POT: {}",
            self.position as u8,
            self.position,
            self.name,
            self.stack.get(),
            self.hand.get(),
            self.chips_in_play.get(),
            self.chips_in_pot.get()
        )
    }
}

impl From<SeatSnapshot> for Seat {
    fn from(ss: SeatSnapshot) -> Self {
        Seat {
            position: ss.position,
            name: ss.name.clone(),
            is_active: Cell::new(ss.is_active),
            stack: Cell::new(Chips::new(ss.stack)),
            chips_in_play: Cell::new(Chips::new(ss.chips_in_play)),
            chips_in_pot: Cell::new(ss.chips_in_pot),
            hand: Cell::new(ss.hand),
        }
    }
}

/// Utility struct to enable easy recording of games in progress.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SeatSnapshot {
    pub position: Position6Max,
    pub name: String,
    pub is_active: bool,
    pub stack: usize,
    pub chips_in_play: usize,
    pub chips_in_pot: usize,
    pub hand: Two,
}

impl Display for SeatSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SEAT #{} {:>4}\t{:>8}:\t{}\t{} IN PLAY: {}\tINTO POT: {}",
            self.position as u8, self.position, self.name, self.stack, self.hand, self.chips_in_play, self.chips_in_pot
        )
    }
}

impl From<&Seat> for SeatSnapshot {
    fn from(seat: &Seat) -> Self {
        SeatSnapshot {
            position: seat.position,
            name: seat.name.clone(),
            is_active: seat.is_active.get(),
            stack: seat.stack.get().size(),
            chips_in_play: seat.chips_in_play.get().size(),
            chips_in_pot: seat.chips_in_pot.get(),
            hand: seat.hand.get(),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod store_pluribus_seat_tests {
    use super::*;

    fn test_struct() -> Seat {
        Seat {
            position: Position6Max::UTG,
            name: "Flub".to_string(),
            is_active: Cell::new(true),
            stack: Cell::new(Chips::new(500)),
            chips_in_play: Cell::new(Chips::default()),
            chips_in_pot: Cell::new(0),
            hand: Cell::from(Two::default()),
        }
    }

    #[test]
    fn new() {
        let expected = test_struct();

        let actual = Seat::new("Flub".to_string(), Position6Max::UTG, Chips::new(500));

        assert_eq!(expected, actual);
    }

    #[test]
    fn bet() {
        let seat = test_struct();
        let amount = 40usize;
        let expected_stack_size = seat.stack_size() - amount;

        let the_bet = seat.bet(amount);

        assert_eq!(expected_stack_size, seat.stack_size());
        assert_eq!(expected_stack_size, the_bet.unwrap());
        assert_eq!(Chips::new(amount), seat.chips_in_play.get());
        assert!(seat.bet(600).is_none());
    }

    // Bill(SB) posts Small Blind 50
    // Pluribus(BB) posts Big Blind 100
    // Bill(SB) dealt 8♥ 3♣
    // Pluribus(BB) dealt 7♦ 6♦
    // MrWhite(UTG) dealt T♦ 5♦
    // Gogo(MP) dealt 7♠ 7♣
    // Budd(CO) dealt A♥ 4♣
    // Eddie(BTN) dealt T♠ 9♦
    // Preflop Phase over
    //
    // MrWhite(UTG) folds T♦ 5♦ leaving 0 in the pot
    // Gogo(MP) 7♠ 7♣ raises 200 FLOOR BEFORE: 100 FLOOR AFTER:200
    // Budd(CO) folds A♥ 4♣ leaving 0 in the pot
    // Eddie(BTN) folds T♠ 9♦ leaving 0 in the pot
    // Bill(SB) folds 8♥ 3♣ leaving 50 in the pot
    // Pluribus(BB) 7♦ 6♦ raises 1100 FLOOR BEFORE: 200 FLOOR AFTER:1100
    // SEAT #2 BB	Pluribus:	8800	7♦ 6♦ IN PLAY: 1200	INTO POT: 0
    #[test]
    fn bet_52() {
        let pluribus = Seat::new("Pluribus".to_string(), Position6Max::BB, Chips::new(10_000));
        pluribus.hand.set(Two::HAND_7D_6D);
        pluribus.bet(100);
        assert_eq!(9_900, pluribus.stack.get().size());
        assert_eq!(100, pluribus.chips_in_play.get().size());

        pluribus.bet(1_100);
        assert_eq!(8_900, pluribus.stack.get().size());
        assert_eq!(1_100, pluribus.chips_in_play.get().size());
    }

    #[test]
    fn desc() {
        let seat = test_struct();

        assert_eq!("Flub(UTG)", seat.desc());
    }

    #[test]
    fn end_round() {
        let seat = test_struct();
        seat.chips_in_pot.set(100);
        let first_bet_amount = 40usize;
        seat.bet(first_bet_amount);
        let second_bet_amount = 120usize;
        seat.bet(second_bet_amount);
        let expected_into = Chips::new(second_bet_amount);
        let expected_in_pot = second_bet_amount + seat.chips_in_pot.get();

        let in_to_pot = seat.end_round();

        assert_eq!(expected_into, in_to_pot);
        assert_eq!(expected_in_pot, seat.chips_in_pot.get());
        assert!(seat.chips_in_play.get().is_empty());
    }

    #[test]
    fn fold() {
        let seat = test_struct();
        seat.dealt(Two::HAND_AS_AH).expect("nada");
        seat.bet(200);
        let (discarded, chips_lost_in_round) = seat.fold();

        assert_eq!(Two::HAND_AS_AH, discarded);
        assert_eq!(200, chips_lost_in_round);
        assert!(!seat.is_dealt());
        assert_eq!(Two::default(), seat.hand.get());
        assert_eq!(Chips::new(0), seat.chips_in_play.get());
        assert_eq!(200, seat.chips_in_pot.get());
    }

    #[test]
    fn is_dealt() {
        let seat = test_struct();

        assert!(!test_struct().is_dealt());

        assert!(seat.dealt(Two::HAND_AS_AH).is_ok());
        assert_eq!(Two::HAND_AS_AH, seat.hand.get());
        assert!(seat.hand.get().is_dealt());
        assert!(seat.is_dealt());
    }

    #[test]
    fn stack_size() {
        assert_eq!(500, test_struct().stack_size());
    }

    #[test]
    fn display() {
        let seat = test_struct();
        seat.dealt(Two::HAND_AS_AH).expect("");
        seat.bet(40);

        assert_eq!(
            "SEAT #3 UTG\t    Flub:\t460\tA♠ A♥ IN PLAY: 40\tINTO POT: 0",
            seat.to_string()
        );
    }

    #[test]
    fn from_seat_snapshot() {
        let ss = SeatSnapshot {
            position: Position6Max::UTG,
            name: "Flub".to_string(),
            is_active: true,
            stack: 500,
            chips_in_play: 0,
            chips_in_pot: 0,
            hand: Two::default(),
        };

        assert_eq!(test_struct(), Seat::from(ss));
    }

    #[test]
    fn seat_snapshot__from_seat() {
        let expected = SeatSnapshot {
            position: Position6Max::UTG,
            name: "Flub".to_string(),
            is_active: true,
            stack: 500,
            chips_in_play: 0,
            chips_in_pot: 0,
            hand: Two::default(),
        };

        let snapshot = SeatSnapshot::from(&test_struct());

        assert_eq!(expected, snapshot);
    }
}
