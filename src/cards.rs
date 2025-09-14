use ratatui::style::Color;
use std::fmt::{Display, Formatter, Result as FmtResult};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn symbol(self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }

    pub fn color(self) -> Color {
        match self {
            Suit::Diamonds | Suit::Hearts => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Gray,
        }
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(pub u8);

impl Rank {
    pub fn new(v: u8) -> Self { Self(v) }
    pub fn value(self) -> u8 { self.0 }
    pub fn label(self) -> &'static str {
        match self.0 {
            1 => "A",
            2..=10 => {
                // SAFETY: numbers 2..=10 have these literals
                match self.0 {
                    2 => "2",
                    3 => "3",
                    4 => "4",
                    5 => "5",
                    6 => "6",
                    7 => "7",
                    8 => "8",
                    9 => "9",
                    10 => "10",
                    _ => unreachable!(),
                }
            }
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => "?",
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self { Self { suit, rank } }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl Card {
    pub fn is_monster(&self) -> bool {
        matches!(self.suit, Suit::Clubs | Suit::Spades)
    }
    pub fn monster_value(&self) -> u8 {
        // Monsters are clubs/spades; Ace is 14, J=11, Q=12, K=13
        match self.rank.value() {
            1 => 14, // Ace
            n @ 2..=10 => n,
            11 => 11, // Jack
            12 => 12, // Queen
            13 => 13, // King
            _ => 0,
        }
    }
}
