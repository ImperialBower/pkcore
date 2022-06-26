use crate::util::wincounter::PlayerFlag;

#[derive(Debug)]
pub struct Win;

impl Win {
    pub const FIRST: PlayerFlag = 0b0000_0001;
    pub const SECOND: PlayerFlag = 0b0000_0010;
    pub const THIRD: PlayerFlag = 0b0000_0100;
    pub const FORTH: PlayerFlag = 0b0000_1000;
    pub const FIFTH: PlayerFlag = 0b0001_0000;
    pub const SIXTH: PlayerFlag = 0b0010_0000;
    pub const SEVENTH: PlayerFlag = 0b0100_0000;
    pub const EIGHT: PlayerFlag = 0b1000_0000;
    pub const NINTH: PlayerFlag = 0b1_0000_0000;
    pub const TENTH: PlayerFlag = 0b10_0000_0000;
    pub const ELEVENTH: PlayerFlag = 0b100_0000_0000;
    pub const TWELFTH: PlayerFlag = 0b1000_0000_0000;
    pub const THIRTEENTH: PlayerFlag = 0b1_0000_0000_0000;
    pub const FOURTEENTH: PlayerFlag = 0b10_0000_0000_0000;
    pub const FIFTEENTH: PlayerFlag = 0b100_0000_0000_0000;
    pub const SIXTEENTH: PlayerFlag = 0b1000_0000_0000_0000;

    /// `CaseEval` win count Test #2: TAKE TWO detour.
    ///
    /// Our heroic system has been sidelined. Our heroes need a way to translate a zero based
    /// index position of a vector into a bit flag representation of that index. So, in other words,
    /// the index for the first position in a vector is 0:
    ///
    /// ```
    /// let v: Vec<usize> = vec![10, 9, 8, 7, 6];
    ///
    /// assert_eq!(*v.get(0).unwrap(), 10);
    /// ```
    /// [rust playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6cf71e83aa0d16dba23fd310d07efc3c)
    ///
    /// So we need a function that will return the bit flag for the specific position in the vector:
    /// `0` returns `0b0000_0001`... `1` returns `0b0000_0010`, etc...
    ///
    /// Now I suppose the smart way to write this function would be to create some logic that will
    /// convert the index into a bit flag. The problem is, that I don't want to think about that
    /// right now, and, as the founder of the dumb coder movement, I am going to write this function
    /// in the stupidest way I can think of, proud of the fact that later on, someone smarter than
    /// me will offer a cooler, more awesomer way of coding this, and will send me a pull request
    /// with their solution just to prove how much smarter they are than me.
    ///
    /// ASIDE: _When you think about it, how much of the shit we do boils down to basic primate
    /// behavior? Lord knows that's true with software developers. If only there was a way to
    /// harness this dick measuring energy into getting them to write this book for me 🤔_
    ///
    /// Me, I'm going to code a good ol' fashioned match statement:
    ///
    /// ```
    /// use pkcore::util::wincounter::PlayerFlag;
    /// use pkcore::util::wincounter::win::Win;
    /// fn from_index(i: usize) -> PlayerFlag {
    ///     match i {
    ///         0 => Win::FIRST,
    ///         1 => Win::SECOND,
    ///         2 => Win::THIRD,
    ///         _ => PlayerFlag::default()
    ///     }
    /// }
    /// assert_eq!(Win::FIRST, from_index(0));
    /// assert_eq!(Win::SECOND, from_index(1));
    /// ```
    ///
    /// For now, the contract of `wincounter` is to support up to 16 players. The idea behind
    /// this library was to have an easy way to deal winning percentages for situations where
    /// more than only person in a game could win. Granted, the maximum number of players in a
    /// single deck poker game is generally less than that. I've heard numbers of
    /// [9, 10, 11,](https://poker.stackexchange.com/questions/4413/what-is-the-maximum-number-of-players-in-texas-holdem)
    /// and even [22](https://www.betfirm.com/max-number-of-players-in-texas-hold-em/). Me, I'm
    /// doing 16 so I don't have to think about it for a while.
    ///
    /// You will have to forgive me if I don't test drive through this function too much. I've got
    /// one failing `todo!()` test, and I'm going to just implement the match as seems write,
    /// write tests to verify all of the boundary conditions, and call it a day.
    ///
    /// So what are the boundary conditions?
    ///
    /// * Positive: unsigned integer between 0 and 15.
    /// * Negative: unsigned integer greater than 15.
    ///
    /// I'll be honest with you. This wincounter library isn't wowing me. My desire to avoid
    /// the hassle of wrapping an u16 `Count` in a struct is making me bend over backwards to
    /// deal with the fact that I can't write methods against `Count` because it isn't a struct
    /// or an enum. I feel a major refactoring coming on for this code. I'll hold off for now,
    /// but it's in my backlog.
    ///
    /// Adding a technical debt not to my code as a reminder.
    ///
    /// *NOTE:* This isn't the first time I've had to do this sort of refactoring. The initial
    /// version of the `Card` struct was a
    /// [simple type alias](https://github.com/ContractBridge/ckc-rs/blob/5f301f182eb579c9f8df4e243b6ebecd310b1b24/src/lib.rs#L33).
    /// For the instance in this book, I decided to write it as a struct to make the code cleaner
    /// and easier to manage. It's only a matter of time before I do the same thing to
    /// `wincounter::Count`. Not doing things right in order to save you some time will always
    /// end up taking more time in the long run. Count on it.
    ///
    #[must_use]
    pub fn from_index(i: usize) -> PlayerFlag {
        match i {
            0 => Win::FIRST,
            1 => Win::SECOND,
            2 => Win::THIRD,
            3 => Win::FORTH,
            4 => Win::FIFTH,
            5 => Win::SIXTH,
            6 => Win::SEVENTH,
            7 => Win::EIGHT,
            8 => Win::NINTH,
            9 => Win::TENTH,
            10 => Win::ELEVENTH,
            11 => Win::TWELFTH,
            12 => Win::THIRTEENTH,
            13 => Win::FOURTEENTH,
            14 => Win::FIFTEENTH,
            15 => Win::SIXTEENTH,
            _ => PlayerFlag::default(),
        }
    }

    /// This is probably a needless function, and a pair partner stronger than the voice in my head
    /// would probably just tell me to do a simple OR operation. The truth is, that I have to
    /// duckduckgo these bitwise operators every time I start to use them, so this is a way for me
    /// to separate out the logic, write some tests to make sure I'm getting it right, and move on
    /// comfortable with the knowledge that I am doing what I set out to do. Don't be too hard on
    /// yourself for writing stupid code. Be hard on yourself for writing untested code.
    #[must_use]
    pub fn or(a: PlayerFlag, b: PlayerFlag) -> PlayerFlag {
        a | b
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod util__wincounter__win__tests {
    use super::*;

    #[test]
    fn from_index() {
        assert_eq!(Win::FIRST, Win::from_index(0));
        assert_eq!(Win::SECOND, Win::from_index(1));
        assert_eq!(Win::THIRD, Win::from_index(2));
        assert_eq!(Win::FORTH, Win::from_index(3));
        assert_eq!(Win::FIFTH, Win::from_index(4));
        assert_eq!(Win::SIXTH, Win::from_index(5));
        assert_eq!(Win::SEVENTH, Win::from_index(6));
        assert_eq!(Win::EIGHT, Win::from_index(7));
        assert_eq!(Win::NINTH, Win::from_index(8));
        assert_eq!(Win::TENTH, Win::from_index(9));
        assert_eq!(Win::ELEVENTH, Win::from_index(10));
        assert_eq!(Win::TWELFTH, Win::from_index(11));
        assert_eq!(Win::THIRTEENTH, Win::from_index(12));
        assert_eq!(Win::FOURTEENTH, Win::from_index(13));
        assert_eq!(Win::FIFTEENTH, Win::from_index(14));
        assert_eq!(Win::SIXTEENTH, Win::from_index(15));
        assert_eq!(PlayerFlag::default(), Win::from_index(16));
    }

    #[test]
    fn or() {
        assert_eq!(0b0000_0110, Win::or(Win::SECOND, Win::THIRD));
    }
}
