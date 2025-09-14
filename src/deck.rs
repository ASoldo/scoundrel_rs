use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::cards::{Card, Rank, Suit};

#[derive(Debug, Default, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn scoundrel_deck() -> Self {
        // Build a deck per Scoundrel rules:
        // - Remove Jokers (not present here)
        // - Remove Red Face Cards (J,Q,K of Hearts/Diamonds) and Red Aces (A of Hearts/Diamonds)
        // - Monsters: all Clubs/Spades (2..=10, J,Q,K,A)
        // - Weapons: Diamonds 2..=10
        // - Potions: Hearts 2..=10
        let mut cards = Vec::with_capacity(44);
        // Clubs & Spades: full 13 ranks (monsters)
        for suit in [Suit::Clubs, Suit::Spades] {
            for v in 1..=13u8 {
                cards.push(Card::new(suit, Rank::new(v)));
            }
        }
        // Diamonds: only 2..=10 (weapons)
        for v in 2..=10u8 { cards.push(Card::new(Suit::Diamonds, Rank::new(v))); }
        // Hearts: only 2..=10 (potions)
        for v in 2..=10u8 { cards.push(Card::new(Suit::Hearts, Rank::new(v))); }
        Self { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn len(&self) -> usize { self.cards.len() }
    pub fn is_empty(&self) -> bool { self.cards.is_empty() }
    pub fn push_bottom(&mut self, card: Card) { self.cards.insert(0, card); }
}
