use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seatbit(pub u16);

impl Seatbit {
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
}

impl Display for Seatbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0b}", self.0)
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
}
