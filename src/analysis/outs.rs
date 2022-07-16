use crate::{Card, Cards};
use indexmap::IndexMap;

/// This is old `Fudd` code.
#[derive(Clone, Debug)]
pub struct Outs(IndexMap<usize, Cards>);

impl Outs {
    /// I'll confess that the `get_mut()` function threw me off.
    /// `let ref mut set = self.0.get_mut(&player).unwrap();` generates this error message:
    ///
    /// ```txt
    /// warning: `ref` on an entire `let` pattern is discouraged, take a reference with `&` instead
    ///   --> src/analysis/outs.rs:24:13
    ///    |
    /// 24 |         let ref mut set = self.0.get_mut(&player).unwrap();
    ///    |         ----^^^^^^^^^^^------------------------------------ help: try: `let set = &mut self.0.get_mut(&player).unwrap();`
    ///    |
    ///    = note: `#[warn(clippy::toplevel_ref_arg)]` on by default
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#toplevel_ref_arg
    /// ```
    ///
    /// But when I use the code suggested (`let set = &mut self.0.get_mut(&player).unwrap();`) I get
    /// this clippy warning:
    ///
    /// ```txt
    /// warning: this expression mutably borrows a mutable reference. Consider reborrowing
    ///   --> src/analysis/outs.rs:39:19
    ///    |
    /// 39 |         let set = &mut self.0.get_mut(&player).unwrap();
    ///    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///    |
    /// ```
    ///
    /// Then, on a lark I tried removing the `&mut`s all together, and what do you know, it worked.
    /// This is why we write unit tests. The rust compiler, no matter how good it is, can only show
    /// us so much. This gives us this:
    ///
    /// ```txt
    /// pub fn add(&mut self, player: usize, card: Card) {
    ///     self.touch(player);
    ///     let set = self.0.get_mut(&player).unwrap();
    ///     set.insert(card);
    /// }
    /// ```
    ///
    /// Let's try one last little change. Do we really need to set the set variable
    /// before we call insert? Turns out the answer is no.
    ///
    /// # Panics
    ///
    /// Shouldn't be possible 🤞
    pub fn add(&mut self, player: usize, card: Card) {
        self.touch(player);
        self.0.get_mut(&player).unwrap().insert(card);
    }

    /// *FRACK*
    ///
    /// Writing tests for this method has uncovered a defect with `Cards.sort()`.
    ///
    /// ```
    /// use pkcore::analysis::outs::Outs;
    /// use pkcore::card::Card;
    ///
    /// let mut outs = Outs::default();
    /// outs.add(1, Card::SIX_CLUBS);
    /// outs.add(1, Card::SEVEN_SPADES);
    /// outs.add(1, Card::SEVEN_DIAMONDS);
    /// outs.add(1, Card::EIGHT_DIAMONDS);
    ///
    /// assert_eq!("8♦ 7♠ 7♦ 6♣", outs.get(1).unwrap().sort().to_string());
    /// ```
    ///
    /// This sort result is `Rank` weighted. Ideally, we'd like this to be `Suit`
    /// weighted, followed by `Rank`. This would create a result from the test of `7♠ 8♦ 7♦ 6♣`.
    /// For now we're going to mark this as a `todo` in `Cards` and add a test that we ignore for
    /// now. This is one of those nice to haves right now.
    ///
    /// # Panics
    ///
    /// Shouldn't be possible 🤞
    pub fn append(&mut self, other: &Outs) {
        for (player, cards) in other.iter() {
            self.touch(*player);
            self.0.get_mut(player).unwrap().insert_all(cards);
        }
    }

    #[must_use]
    pub fn get(&self, player: usize) -> Option<&Cards> {
        self.0.get(&player)
    }

    #[must_use]
    pub fn iter(&self) -> indexmap::map::Iter<'_, usize, Cards> {
        self.0.iter()
    }

    pub fn touch(&mut self, player: usize) -> bool {
        if self.0.get(&player).is_none() {
            self.0.insert(player, Cards::default());
            true
        } else {
            false
        }
    }
}

impl Default for Outs {
    fn default() -> Outs {
        Outs(IndexMap::new())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__outs_tests {
    use super::*;

    #[test]
    fn add() {
        let mut outs = Outs::default();

        outs.add(1, Card::SIX_CLUBS);
        outs.add(1, Card::SEVEN_SPADES);

        assert_eq!("6♣ 7♠", outs.get(1).unwrap().to_string());
        assert_eq!("7♠ 6♣", outs.get(1).unwrap().sort().to_string());
    }

    #[test]
    fn append() {
        let mut outs1 = Outs::default();
        let mut outs2 = Outs::default();
        outs1.add(1, Card::SIX_CLUBS);
        outs1.add(1, Card::SEVEN_SPADES);
        outs2.add(1, Card::SEVEN_DIAMONDS);
        outs2.add(1, Card::EIGHT_DIAMONDS);

        outs1.append(&outs2);

        assert_eq!("6♣ 7♠ 7♦ 8♦", outs1.get(1).unwrap().to_string());
        assert_eq!("8♦ 7♠ 7♦ 6♣", outs1.get(1).unwrap().sort().to_string());
    }

    #[test]
    fn touch() {
        let mut outs = Outs::default();

        let touched = outs.touch(1);

        assert!(touched);
        assert_eq!(Cards::default(), *outs.get(1).unwrap());
        assert!(outs.get(2).is_none());
    }
}
