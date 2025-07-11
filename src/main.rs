use std::collections::HashSet;
use strum::IntoEnumIterator;
use crate::deck::Deck;
use crate::items::JokerTypes::*;
use crate::items::{Bosses, JokerTypes, Vouchers};
use crate::random::{ItemChoice, Random};
use crate::shop::{Shop, ShopItem, ShopRates};

pub mod items;
pub mod pools;
pub mod random;
pub mod util;
pub mod shop;
mod deck;

pub const MAX_SEED_LENGTH: usize = 8;
pub const SEED_CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn main() {
    let mut deck = Deck::default();
    deck.shuffle(&mut Random::new("TEST".to_string()), 1);
    println!("{}", deck);

    let mut boss_pool = Bosses::iter().filter(|boss| !boss.locked()).collect::<Vec<_>>();
    let mut random = Random::new("TESTING".to_string());
    for _ in 0..8 {
        let boss = random.rand_choice("boss", &boss_pool);
        println!("{:?}", boss);
        boss_pool.remove(boss_pool.iter().position(|b| b == boss).unwrap());
    }

    return;
    let useless_jokers = HashSet::from_iter([
        CreditCard,
        EightBall,
        ChaostheClown,
        DelayedGratification,
        BusinessCard,
        Egg,
        Splash,
        FacelessJoker,
        Superposition,
        ToDoList,
        ReservedParking,
        MailInRebate,
        Hallucination,
        Juggler,
        Drunkard,
        GoldenJoker,
        GoldenTicket,
        FourFingers,
        MarbleJoker,
        Pareidolia,
        Shortcut,
        Cloud9,
        Rocket,
        MidasMask,
        Luchador,
        GiftCard,
        TurtleBean,
        ToTheMoon,
        StoneJoker,
        DietCola,
        TradingCard,
        Troubadour,
        Certificate,
        SmearedJoker,
        RoughGem,
        Showman,
        MerryAndy,
        OopsAllSixes,
        Matador,
        Satellite,
        // Rare jokers
        DNA,
        Blueprint,
        InvisibleJoker,
        Brainstorm,
        DriversLicense,
    ]);

    test_seed_recursive(&useless_jokers, "", 0);
}

fn test_seed_recursive(useless: &HashSet<JokerTypes>, parent: &str, index: usize) {
    let current = format!("{parent}{}", SEED_CHARSET.chars().skip(index).next().unwrap());
    if parent.len() < MAX_SEED_LENGTH - 1 {
        test_seed_recursive(useless, &current, 0);
    }

    if index < SEED_CHARSET.len() - 1 {
        test_seed_recursive(useless, parent, index + 1);
    }

    let mut random = Random::new(current);

    let boss_pool = Bosses::iter().filter(|boss| !boss.locked()).collect::<Vec<_>>();
    if random.rand_choice("boss", &boss_pool) != &Bosses::TheNeedle {
        return;
    }

    // Rerolls to try:
    // Ante 1:
    // Small blind: 4 in, +3 base, +3 hands, 10 out
    // Big blind: 10 in, +2 interest, +4 base, +3 hands, 19 out
    // Max of 19 dollars, or 3 rerolls (+2 viewed shops)
    // Boss: 19 in, +3 interest, +5 base, +3 hands, 30 out
    // Ante 2:
    let mut shop = Shop::new(1, ShopRates::default(), &mut random);
    if shop.generate::<10>().refreshable.iter().all(|item| match item {
        ShopItem::Joker(joker) => useless.contains(&joker.joker),
        _ => true,
    }) {
        println!("Hit! Seed: {}", random.seed);
    }
}