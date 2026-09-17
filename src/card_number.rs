use crate::PKError;
use strum::EnumIter;

pub type CKCNumber = u32;

//region BasicCards
const CKC_AS: CKCNumber = 0b01_0000_0000_0000_1000_1100_0010_1001;
const CKC_KS: CKCNumber = 0b00_1000_0000_0000_1000_1011_0010_0101;
const CKC_QS: CKCNumber = 0b00_0100_0000_0000_1000_1010_0001_1111;
const CKC_JS: CKCNumber = 0b00_0010_0000_0000_1000_1001_0001_1101;
const CKC_TS: CKCNumber = 0b00_0001_0000_0000_1000_1000_0001_0111;
const CKC_9S: CKCNumber = 0b00_0000_1000_0000_1000_0111_0001_0011;
const CKC_8S: CKCNumber = 0b00_0000_0100_0000_1000_0110_0001_0001;
const CKC_7S: CKCNumber = 0b00_0000_0010_0000_1000_0101_0000_1101;
const CKC_6S: CKCNumber = 0b00_0000_0001_0000_1000_0100_0000_1011;
const CKC_5S: CKCNumber = 0b00_0000_0000_1000_1000_0011_0000_0111;
const CKC_4S: CKCNumber = 0b00_0000_0000_0100_1000_0010_0000_0101;
const CKC_3S: CKCNumber = 0b00_0000_0000_0010_1000_0001_0000_0011;
const CKC_2S: CKCNumber = 0b00_0000_0000_0001_1000_0000_0000_0010;

const CKC_AH: CKCNumber = 0b01_0000_0000_0000_0100_1100_0010_1001;
const CKC_KH: CKCNumber = 0b00_1000_0000_0000_0100_1011_0010_0101;
const CKC_QH: CKCNumber = 0b00_0100_0000_0000_0100_1010_0001_1111;
const CKC_JH: CKCNumber = 0b00_0010_0000_0000_0100_1001_0001_1101;
const CKC_TH: CKCNumber = 0b00_0001_0000_0000_0100_1000_0001_0111;
const CKC_9H: CKCNumber = 0b00_0000_1000_0000_0100_0111_0001_0011;
const CKC_8H: CKCNumber = 0b00_0000_0100_0000_0100_0110_0001_0001;
const CKC_7H: CKCNumber = 0b00_0000_0010_0000_0100_0101_0000_1101;
const CKC_6H: CKCNumber = 0b00_0000_0001_0000_0100_0100_0000_1011;
const CKC_5H: CKCNumber = 0b00_0000_0000_1000_0100_0011_0000_0111;
const CKC_4H: CKCNumber = 0b00_0000_0000_0100_0100_0010_0000_0101;
const CKC_3H: CKCNumber = 0b00_0000_0000_0010_0100_0001_0000_0011;
const CKC_2H: CKCNumber = 0b00_0000_0000_0001_0100_0000_0000_0010;

const CKC_AD: CKCNumber = 0b01_0000_0000_0000_0010_1100_0010_1001;
const CKC_KD: CKCNumber = 0b00_1000_0000_0000_0010_1011_0010_0101;
const CKC_QD: CKCNumber = 0b00_0100_0000_0000_0010_1010_0001_1111;
const CKC_JD: CKCNumber = 0b00_0010_0000_0000_0010_1001_0001_1101;
const CKC_TD: CKCNumber = 0b00_0001_0000_0000_0010_1000_0001_0111;
const CKC_9D: CKCNumber = 0b00_0000_1000_0000_0010_0111_0001_0011;
const CKC_8D: CKCNumber = 0b00_0000_0100_0000_0010_0110_0001_0001;
const CKC_7D: CKCNumber = 0b00_0000_0010_0000_0010_0101_0000_1101;
const CKC_6D: CKCNumber = 0b00_0000_0001_0000_0010_0100_0000_1011;
const CKC_5D: CKCNumber = 0b00_0000_0000_1000_0010_0011_0000_0111;
const CKC_4D: CKCNumber = 0b00_0000_0000_0100_0010_0010_0000_0101;
const CKC_3D: CKCNumber = 0b00_0000_0000_0010_0010_0001_0000_0011;
const CKC_2D: CKCNumber = 0b00_0000_0000_0001_0010_0000_0000_0010;

const CKC_AC: CKCNumber = 0b01_0000_0000_0000_0001_1100_0010_1001;
const CKC_KC: CKCNumber = 0b00_1000_0000_0000_0001_1011_0010_0101;
const CKC_QC: CKCNumber = 0b00_0100_0000_0000_0001_1010_0001_1111;
const CKC_JC: CKCNumber = 0b00_0010_0000_0000_0001_1001_0001_1101;
const CKC_TC: CKCNumber = 0b00_0001_0000_0000_0001_1000_0001_0111;
const CKC_9C: CKCNumber = 0b00_0000_1000_0000_0001_0111_0001_0011;
const CKC_8C: CKCNumber = 0b00_0000_0100_0000_0001_0110_0001_0001;
const CKC_7C: CKCNumber = 0b00_0000_0010_0000_0001_0101_0000_1101;
const CKC_6C: CKCNumber = 0b00_0000_0001_0000_0001_0100_0000_1011;
const CKC_5C: CKCNumber = 0b00_0000_0000_1000_0001_0011_0000_0111;
const CKC_4C: CKCNumber = 0b00_0000_0000_0100_0001_0010_0000_0101;
const CKC_3C: CKCNumber = 0b00_0000_0000_0010_0001_0001_0000_0011;
const CKC_2C: CKCNumber = 0b00_0000_0000_0001_0001_0000_0000_0010;
//endregion

/// The Cactus Kev number of each of the 52 cards.
///
/// # Domain
///
/// - **Role:** value object
/// - **Invariants:**
///   - only the 52 valid CKC numbers
#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum CardNumber {
    AceSpades = CKC_AS,
    KingSpades = CKC_KS,
    QueenSpades = CKC_QS,
    JackSpades = CKC_JS,
    TenSpades = CKC_TS,
    NineSpades = CKC_9S,
    EightSpades = CKC_8S,
    SevenSpades = CKC_7S,
    SixSpades = CKC_6S,
    FiveSpades = CKC_5S,
    FourSpades = CKC_4S,
    TreySpades = CKC_3S,
    DeuceSpades = CKC_2S,
    AceHearts = CKC_AH,
    KingHearts = CKC_KH,
    QueenHearts = CKC_QH,
    JackHearts = CKC_JH,
    TenHearts = CKC_TH,
    NineHearts = CKC_9H,
    EightHearts = CKC_8H,
    SevenHearts = CKC_7H,
    SixHearts = CKC_6H,
    FiveHearts = CKC_5H,
    FourHearts = CKC_4H,
    TreyHearts = CKC_3H,
    DeuceHearts = CKC_2H,
    AceDiamonds = CKC_AD,
    KingDiamonds = CKC_KD,
    QueenDiamonds = CKC_QD,
    JackDiamonds = CKC_JD,
    TenDiamonds = CKC_TD,
    NineDiamonds = CKC_9D,
    EightDiamonds = CKC_8D,
    SevenDiamonds = CKC_7D,
    SixDiamonds = CKC_6D,
    FiveDiamonds = CKC_5D,
    FourDiamonds = CKC_4D,
    TreyDiamonds = CKC_3D,
    DeuceDiamonds = CKC_2D,
    AceClubs = CKC_AC,
    KingClubs = CKC_KC,
    QueenClubs = CKC_QC,
    JackClubs = CKC_JC,
    TenClubs = CKC_TC,
    NineClubs = CKC_9C,
    EightClubs = CKC_8C,
    SevenClubs = CKC_7C,
    SixClubs = CKC_6C,
    FiveClubs = CKC_5C,
    FourClubs = CKC_4C,
    TreyClubs = CKC_3C,
    DeuceClubs = CKC_2C,
}

impl TryFrom<u32> for CardNumber {
    type Error = PKError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            CKC_AS => Ok(CardNumber::AceSpades),
            CKC_KS => Ok(CardNumber::KingSpades),
            CKC_QS => Ok(CardNumber::QueenSpades),
            CKC_JS => Ok(CardNumber::JackSpades),
            CKC_TS => Ok(CardNumber::TenSpades),
            CKC_9S => Ok(CardNumber::NineSpades),
            CKC_8S => Ok(CardNumber::EightSpades),
            CKC_7S => Ok(CardNumber::SevenSpades),
            CKC_6S => Ok(CardNumber::SixSpades),
            CKC_5S => Ok(CardNumber::FiveSpades),
            CKC_4S => Ok(CardNumber::FourSpades),
            CKC_3S => Ok(CardNumber::TreySpades),
            CKC_2S => Ok(CardNumber::DeuceSpades),

            CKC_AH => Ok(CardNumber::AceHearts),
            CKC_KH => Ok(CardNumber::KingHearts),
            CKC_QH => Ok(CardNumber::QueenHearts),
            CKC_JH => Ok(CardNumber::JackHearts),
            CKC_TH => Ok(CardNumber::TenHearts),
            CKC_9H => Ok(CardNumber::NineHearts),
            CKC_8H => Ok(CardNumber::EightHearts),
            CKC_7H => Ok(CardNumber::SevenHearts),
            CKC_6H => Ok(CardNumber::SixHearts),
            CKC_5H => Ok(CardNumber::FiveHearts),
            CKC_4H => Ok(CardNumber::FourHearts),
            CKC_3H => Ok(CardNumber::TreyHearts),
            CKC_2H => Ok(CardNumber::DeuceHearts),

            CKC_AD => Ok(CardNumber::AceDiamonds),
            CKC_KD => Ok(CardNumber::KingDiamonds),
            CKC_QD => Ok(CardNumber::QueenDiamonds),
            CKC_JD => Ok(CardNumber::JackDiamonds),
            CKC_TD => Ok(CardNumber::TenDiamonds),
            CKC_9D => Ok(CardNumber::NineDiamonds),
            CKC_8D => Ok(CardNumber::EightDiamonds),
            CKC_7D => Ok(CardNumber::SevenDiamonds),
            CKC_6D => Ok(CardNumber::SixDiamonds),
            CKC_5D => Ok(CardNumber::FiveDiamonds),
            CKC_4D => Ok(CardNumber::FourDiamonds),
            CKC_3D => Ok(CardNumber::TreyDiamonds),
            CKC_2D => Ok(CardNumber::DeuceDiamonds),

            CKC_AC => Ok(CardNumber::AceClubs),
            CKC_KC => Ok(CardNumber::KingClubs),
            CKC_QC => Ok(CardNumber::QueenClubs),
            CKC_JC => Ok(CardNumber::JackClubs),
            CKC_TC => Ok(CardNumber::TenClubs),
            CKC_9C => Ok(CardNumber::NineClubs),
            CKC_8C => Ok(CardNumber::EightClubs),
            CKC_7C => Ok(CardNumber::SevenClubs),
            CKC_6C => Ok(CardNumber::SixClubs),
            CKC_5C => Ok(CardNumber::FiveClubs),
            CKC_4C => Ok(CardNumber::FourClubs),
            CKC_3C => Ok(CardNumber::TreyClubs),
            CKC_2C => Ok(CardNumber::DeuceClubs),

            _ => Err(PKError::InvalidCardNumber),
        }
    }
}
