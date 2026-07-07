use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seatbit(pub u16);

impl Seatbit {
    /// Number of seat positions this bitmask can represent —
    /// one per bit of the `u16` backing field.
    #[allow(clippy::cast_possible_truncation)] // 16 fits in u8
    pub const CAPACITY: u8 = u16::BITS as u8;

    pub const NONE: Seatbit = Seatbit(0b0000_0000_0000_0000);
    pub const SEAT_0: Seatbit = Seatbit(0b0000_0000_0000_0001);
    pub const SEAT_1: Seatbit = Seatbit(0b0000_0000_0000_0010);
    pub const SEAT_2: Seatbit = Seatbit(0b0000_0000_0000_0100);
    pub const SEAT_3: Seatbit = Seatbit(0b0000_0000_0000_1000);
    pub const SEAT_4: Seatbit = Seatbit(0b0000_0000_0001_0000);
    pub const SEAT_5: Seatbit = Seatbit(0b0000_0000_0010_0000);
    pub const SEAT_6: Seatbit = Seatbit(0b0000_0000_0100_0000);
    pub const SEAT_7: Seatbit = Seatbit(0b0000_0000_1000_0000);
    pub const SEAT_8: Seatbit = Seatbit(0b0000_0001_0000_0000);
    pub const SEAT_9: Seatbit = Seatbit(0b0000_0010_0000_0000);
    pub const SEAT_10: Seatbit = Seatbit(0b0000_0100_0000_0000);
    pub const SEAT_11: Seatbit = Seatbit(0b0000_1000_0000_0000);
    pub const SEAT_12: Seatbit = Seatbit(0b0001_0000_0000_0000);
    pub const SEAT_13: Seatbit = Seatbit(0b0010_0000_0000_0000);
    pub const SEAT_14: Seatbit = Seatbit(0b0100_0000_0000_0000);
    pub const SEAT_15: Seatbit = Seatbit(0b1000_0000_0000_0000);

    /// Returns `true` if the bit for `seat_number` is set in this `Seatbit`.
    ///
    /// # Examples
    /// ```
    /// use pkcore::casino::table_celled::seats::seatbit::Seatbit;
    ///
    /// let seats = Seatbit::SEAT_0 | Seatbit::SEAT_3;
    /// assert!(seats.contains(0));
    /// assert!(seats.contains(3));
    /// assert!(!seats.contains(1));
    /// ```
    #[must_use]
    pub fn contains(self, seat_number: u8) -> bool {
        let bit = Seatbit::from(seat_number);
        (self.0 & bit.0) != 0
    }

    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.0.count_ones() as usize
    }
}

impl Display for Seatbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0b}", self.0)
    }
}

impl From<u8> for Seatbit {
    fn from(value: u8) -> Seatbit {
        match value {
            0 => Seatbit::SEAT_0,
            1 => Seatbit::SEAT_1,
            2 => Seatbit::SEAT_2,
            3 => Seatbit::SEAT_3,
            4 => Seatbit::SEAT_4,
            5 => Seatbit::SEAT_5,
            6 => Seatbit::SEAT_6,
            7 => Seatbit::SEAT_7,
            8 => Seatbit::SEAT_8,
            9 => Seatbit::SEAT_9,
            10 => Seatbit::SEAT_10,
            11 => Seatbit::SEAT_11,
            12 => Seatbit::SEAT_12,
            13 => Seatbit::SEAT_13,
            14 => Seatbit::SEAT_14,
            15 => Seatbit::SEAT_15,
            _ => Seatbit::default(),
        }
    }
}

impl From<usize> for Seatbit {
    fn from(value: usize) -> Seatbit {
        Seatbit::from(u8::try_from(value).unwrap_or(99))
    }
}

impl Add for Seatbit {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        // Combine seat bits as a set union.
        Seatbit(self.0 | rhs.0)
    }
}

impl Sub for Seatbit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // Remove rhs seat bits from self.
        Seatbit(self.0 & !rhs.0)
    }
}

impl AddAssign for Seatbit {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        // Combine seat bits in place as a set union.
        self.0 |= rhs.0;
    }
}

impl SubAssign for Seatbit {
    fn sub_assign(&mut self, rhs: Self) {
        // Remove rhs seat bits from self in place.
        self.0 &= !rhs.0;
    }
}

impl BitOr for Seatbit {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Seatbit(self.0 | rhs.0)
    }
}

impl BitOrAssign for Seatbit {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Seatbit {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Seatbit(self.0 & rhs.0)
    }
}

impl BitAndAssign for Seatbit {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXor for Seatbit {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Seatbit(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Seatbit {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Seatbit {
    type Output = Self;

    fn not(self) -> Self::Output {
        Seatbit(!self.0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seats_seatbits_tests {
    use super::*;

    #[test]
    fn display() {
        let combo = Seatbit::SEAT_0 + Seatbit::SEAT_1;

        assert_eq!(Seatbit::SEAT_0.to_string(), "1");
        assert_eq!(Seatbit::SEAT_1.to_string(), "10");
        assert_eq!(combo.to_string(), "11");
    }

    #[test]
    fn add_combines_bits() {
        let seats = Seatbit::SEAT_0 + Seatbit::SEAT_1;
        assert_eq!(seats, Seatbit(0b0000_0000_0000_0011));
    }

    #[test]
    fn subtract_removes_bits() {
        let seats = Seatbit::SEAT_0 + Seatbit::SEAT_1 + Seatbit::SEAT_2;
        let remaining = seats - Seatbit::SEAT_1;
        assert_eq!(remaining, Seatbit(0b0000_0000_0000_0101));
    }

    #[test]
    fn subtract_ignores_missing_bits() {
        let seats = Seatbit::SEAT_0;
        let remaining = seats - Seatbit::SEAT_4;
        assert_eq!(remaining, Seatbit::SEAT_0);
    }

    #[test]
    fn add_assign_combines_bits() {
        let mut seats = Seatbit::SEAT_0;
        seats += Seatbit::SEAT_2;
        assert_eq!(seats, Seatbit(0b0000_0000_0000_0101));
    }

    #[test]
    fn sub_assign_removes_bits() {
        let mut seats = Seatbit::SEAT_0 + Seatbit::SEAT_1 + Seatbit::SEAT_2;
        seats -= Seatbit::SEAT_1;
        assert_eq!(seats, Seatbit(0b0000_0000_0000_0101));
    }

    #[test]
    fn sub_assign_ignores_missing_bits() {
        let mut seats = Seatbit::SEAT_0;
        seats -= Seatbit::SEAT_4;
        assert_eq!(seats, Seatbit::SEAT_0);
    }

    #[test]
    fn bitor_combines_bits() {
        let seats = Seatbit::SEAT_0 | Seatbit::SEAT_1;
        assert_eq!(seats, Seatbit(0b0000_0000_0000_0011));
    }

    #[test]
    fn bitor_assign_combines_bits() {
        let mut seats = Seatbit::SEAT_0;
        seats |= Seatbit::SEAT_2;
        assert_eq!(seats, Seatbit(0b0000_0000_0000_0101));
    }

    #[test]
    fn bitand_intersects_bits() {
        let a = Seatbit::SEAT_0 | Seatbit::SEAT_1 | Seatbit::SEAT_2;
        let b = Seatbit::SEAT_1 | Seatbit::SEAT_2 | Seatbit::SEAT_3;
        assert_eq!(a & b, Seatbit::SEAT_1 | Seatbit::SEAT_2);
    }

    #[test]
    fn bitand_assign_intersects_bits() {
        let mut seats = Seatbit::SEAT_0 | Seatbit::SEAT_1 | Seatbit::SEAT_2;
        seats &= Seatbit::SEAT_1 | Seatbit::SEAT_2 | Seatbit::SEAT_3;
        assert_eq!(seats, Seatbit::SEAT_1 | Seatbit::SEAT_2);
    }

    #[test]
    fn bitxor_toggles_bits() {
        let a = Seatbit::SEAT_0 | Seatbit::SEAT_1;
        let b = Seatbit::SEAT_1 | Seatbit::SEAT_2;
        assert_eq!(a ^ b, Seatbit::SEAT_0 | Seatbit::SEAT_2);
    }

    #[test]
    fn bitxor_assign_toggles_bits() {
        let mut seats = Seatbit::SEAT_0 | Seatbit::SEAT_1;
        seats ^= Seatbit::SEAT_1 | Seatbit::SEAT_2;
        assert_eq!(seats, Seatbit::SEAT_0 | Seatbit::SEAT_2);
    }

    #[test]
    fn not_inverts_bits() {
        let seats = !Seatbit::SEAT_0;
        assert!(!seats.contains(0));
        assert!(seats.contains(1));
        assert!(seats.contains(15));
    }

    #[test]
    fn contains_returns_true_for_set_bits() {
        let seats = Seatbit::SEAT_0 | Seatbit::SEAT_3;
        assert!(seats.contains(0));
        assert!(seats.contains(3));
        assert!(!seats.contains(1));
        assert!(!seats.contains(2));
    }
}
