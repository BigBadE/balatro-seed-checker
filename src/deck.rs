use std::fmt::Display;
use std::mem;
use strum::IntoEnumIterator;
use crate::items::{Card, CardSuits, CardTypes, Editions, EnhancementTypes, SealTypes};
use crate::random::Random;
use crate::util::pseudohash;

pub struct Deck {
    pub cards: Vec<Card>,
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cards.iter().map(|card| format!("{card}")).collect::<Vec<_>>().join(", "))
    }
}

pub enum ShuffleSeed {
    NewRound
}

impl Display for ShuffleSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShuffleSeed::NewRound => write!(f, "nr"),
        }
    }
}

impl Default for Deck {
    fn default() -> Self {
        let mut cards = Vec::new();
        let mut id = 0;
        for suit in CardSuits::iter() {
            for rank in CardTypes::iter() {
                cards.push(Card {
                    rank,
                    suit,
                    enhancement: EnhancementTypes::None,
                    edition: Editions::None,
                    seal: SealTypes::None,
                    sort_id: id,
                });
                id += 1;
            }
        }
        Self { cards }
    }
}

impl Deck {
    pub fn shuffle(&mut self, random: &mut Random, ante: i32) {
        self.pseudoshuffle(random, &format!("{}{ante}", ShuffleSeed::NewRound));
    }

    fn pseudoshuffle(&mut self, random: &mut Random, seed: &str) {
        self.cards.sort_by_key(|card| card.sort_id);
        for i in (1..self.cards.len()).rev() {
            let j = random.rand_int(seed, 1, i as i32);
            self.cards.swap(i, j as usize);
        }
    }
}