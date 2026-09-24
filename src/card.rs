use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    #[inline]
    pub fn symbol(&self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }

    #[inline]
    pub fn char(&self) -> char {
        match self {
            Suit::Clubs => 'c',
            Suit::Diamonds => 'd',
            Suit::Hearts => 'h',
            Suit::Spades => 's',
        }
    }

    #[inline]
    pub fn mask(&self) -> u32 {
        1 << (*self as u32)
    }

    pub fn from_char(c: char) -> Result<Self, String> {
        match c.to_ascii_lowercase() {
            'c' | '♣' => Ok(Suit::Clubs),
            'd' | '♦' => Ok(Suit::Diamonds),
            'h' | '♥' => Ok(Suit::Hearts),
            's' | '♠' => Ok(Suit::Spades),
            _ => Err(format!("Invalid suit character: '{c}'")),
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rank {
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}

const PRIMES: [u32; 13] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    #[inline]
    pub fn prime(&self) -> u32 {
        PRIMES[*self as usize]
    }

    #[inline]
    pub fn char(&self) -> char {
        match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Rank::Two => "Deuce",
            Rank::Three => "Three",
            Rank::Four => "Four",
            Rank::Five => "Five",
            Rank::Six => "Six",
            Rank::Seven => "Seven",
            Rank::Eight => "Eight",
            Rank::Nine => "Nine",
            Rank::Ten => "Ten",
            Rank::Jack => "Jack",
            Rank::Queen => "Queen",
            Rank::King => "King",
            Rank::Ace => "Ace",
        }
    }

    pub fn plural_name(&self) -> &'static str {
        match self {
            Rank::Two => "Deuces",
            Rank::Three => "Threes",
            Rank::Four => "Fours",
            Rank::Five => "Fives",
            Rank::Six => "Sixes",
            Rank::Seven => "Sevens",
            Rank::Eight => "Eights",
            Rank::Nine => "Nines",
            Rank::Ten => "Tens",
            Rank::Jack => "Jacks",
            Rank::Queen => "Queens",
            Rank::King => "Kings",
            Rank::Ace => "Aces",
        }
    }

    pub fn from_char(c: char) -> Result<Self, String> {
        match c.to_ascii_uppercase() {
            '2' => Ok(Rank::Two),
            '3' => Ok(Rank::Three),
            '4' => Ok(Rank::Four),
            '5' => Ok(Rank::Five),
            '6' => Ok(Rank::Six),
            '7' => Ok(Rank::Seven),
            '8' => Ok(Rank::Eight),
            '9' => Ok(Rank::Nine),
            'T' => Ok(Rank::Ten),
            'J' => Ok(Rank::Jack),
            'Q' => Ok(Rank::Queen),
            'K' => Ok(Rank::King),
            'A' => Ok(Rank::Ace),
            _ => Err(format!("Invalid rank character: '{c}'")),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.char())
    }
}

/// A standard playing card represented internally using the Cactus Kev 32-bit integer encoding:
/// - Bits 0-7:   Prime number for rank (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41)
/// - Bits 8-11:  Rank value (0 to 12)
/// - Bits 12-15: Suit bitmask (1 = Club, 2 = Diamond, 4 = Heart, 8 = Spade)
/// - Bits 16-31: Rank bitmask (1 << rank)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    code: u32,
}

impl Card {
    pub const fn from_raw(code: u32) -> Self {
        Self { code }
    }

    pub fn new(rank: Rank, suit: Suit) -> Self {
        let prime = rank.prime();
        let rank_val = (rank as u32) << 8;
        let suit_mask = (suit.mask()) << 12;
        let rank_bit = (1 << (rank as u32)) << 16;
        let code = prime | rank_val | suit_mask | rank_bit;
        Self { code }
    }

    #[inline]
    pub fn raw(&self) -> u32 {
        self.code
    }

    #[inline]
    pub fn prime(&self) -> u32 {
        self.code & 0xFF
    }

    #[inline]
    pub fn rank(&self) -> Rank {
        let r = ((self.code >> 8) & 0x0F) as u8;
        Rank::ALL[r as usize]
    }

    #[inline]
    pub fn suit(&self) -> Suit {
        let s_mask = (self.code >> 12) & 0x0F;
        match s_mask {
            1 => Suit::Clubs,
            2 => Suit::Diamonds,
            4 => Suit::Hearts,
            8 => Suit::Spades,
            _ => unreachable!("Invalid suit mask"),
        }
    }

    #[inline]
    pub fn rank_bitmask(&self) -> u32 {
        self.code >> 16
    }

    pub fn to_ascii(&self) -> String {
        format!("{}{}", self.rank().char(), self.suit().char())
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

impl FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return Err(format!(
                "Card string must be exactly 2 characters (e.g. 'As', 'Th'), got '{s}'"
            ));
        }
        let rank = Rank::from_char(chars[0])?;
        let suit = Suit::from_char(chars[1])?;
        Ok(Card::new(rank, suit))
    }
}

impl TryFrom<&str> for Card {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Card::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_parsing_and_properties() {
        let ace_spades = Card::from_str("As").unwrap();
        assert_eq!(ace_spades.rank(), Rank::Ace);
        assert_eq!(ace_spades.suit(), Suit::Spades);
        assert_eq!(ace_spades.prime(), 41);
        assert_eq!(ace_spades.to_string(), "A♠");

        let two_clubs = Card::from_str("2c").unwrap();
        assert_eq!(two_clubs.rank(), Rank::Two);
        assert_eq!(two_clubs.suit(), Suit::Clubs);
        assert_eq!(two_clubs.prime(), 2);
        assert_eq!(two_clubs.to_string(), "2♣");

        let ten_diamonds = Card::from_str("Td").unwrap();
        assert_eq!(ten_diamonds.rank(), Rank::Ten);
        assert_eq!(ten_diamonds.suit(), Suit::Diamonds);
        assert_eq!(ten_diamonds.to_string(), "T♦");
    }

    #[test]
    fn test_all_52_cards_unique() {
        use std::collections::HashSet;
        let mut codes = HashSet::new();
        for &rank in &Rank::ALL {
            for &suit in &Suit::ALL {
                let card = Card::new(rank, suit);
                assert!(codes.insert(card.raw()));
            }
        }
        assert_eq!(codes.len(), 52);
    }
}
