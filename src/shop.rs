use std::fmt::Display;
use crate::items::{Card, Joker, Pack, Planets, RandomSource, Spectral, Tarots, Vouchers};
use crate::pools::{next_joker, next_planet, next_spectral, next_tarot};
use crate::random::Random;

pub struct Shop<'a> {
    pub ante: i32,
    pub random: &'a mut Random,
    pub rates: ShopRates,
}

pub struct ShopRates {
    pub joker_rate: f64,
    pub tarot_rate: f64,
    pub planet_rate: f64,
    pub playing_card_rate: f64,
    pub spectral_rate: f64,
}

impl Default for ShopRates {
    fn default() -> Self {
        ShopRates {
            joker_rate: 20.0,
            tarot_rate: 4.0,
            planet_rate: 4.0,
            playing_card_rate: 0.0,
            spectral_rate: 0.0,
        }
    }
}

impl ShopRates {
    pub fn total_rate(&self) -> f64 {
        self.joker_rate + self.tarot_rate + self.planet_rate + self.playing_card_rate + self.spectral_rate
    }
}

#[derive(Debug)]
pub enum ShopItem {
    Joker(Joker),
    Tarot(Tarots),
    Planet(Planets),
    Spectral(Spectral),
    Card(Card)
}

impl Display for ShopItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShopItem::Joker(joker) => write!(f, "{}", joker),
            ShopItem::Tarot(tarot) => write!(f, "{:?}", tarot),
            ShopItem::Planet(planet) => write!(f, "{:?}", planet),
            ShopItem::Spectral(spectral) => write!(f, "{:?}", spectral),
            ShopItem::Card(card) => write!(f, "{:?}", card),
        }
    }
}

pub struct ShopState<const N: usize> {
    pub refreshable: [ShopItem; N],
    pub vouchers: Vec<Vouchers>,
    pub packs: Vec<Pack>,
}

impl Shop<'_> {
    pub fn new(ante: i32, rates: ShopRates, random: &mut Random) -> Shop {
        Shop { ante, rates, random }
    }

    pub fn generate<const N: usize>(&mut self) -> ShopState<N> {
        ShopState {
            refreshable: (0..N).map(|_| self.next_shop_item()).collect::<Vec<ShopItem>>().try_into().unwrap(),
            vouchers: vec![],
            packs: vec![],
        }
    }

    fn next_shop_item(&mut self) -> ShopItem {
        let mut cdt_poll = self.random.random(&format!("cdt{}", self.ante)) * self.rates.total_rate();

        if cdt_poll < self.rates.joker_rate  {
            return ShopItem::Joker(next_joker(&mut self.random, RandomSource::Shop, self.ante, true))
        } else {
            cdt_poll -= self.rates.joker_rate;
        }

        if cdt_poll < self.rates.tarot_rate  {
            return ShopItem::Tarot(next_tarot(&mut self.random, RandomSource::Shop, self.ante, false))
        } else {
            cdt_poll -= self.rates.tarot_rate;
        }

        if cdt_poll < self.rates.planet_rate {
            return ShopItem::Planet(next_planet(&mut self.random, RandomSource::Shop, self.ante, false))
        } else {
            cdt_poll -= self.rates.planet_rate;
        }

        if cdt_poll < self.rates.playing_card_rate {
            todo!()
        }

        ShopItem::Spectral(next_spectral(&mut self.random, RandomSource::Shop, self.ante, false))
    }
}