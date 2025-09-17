use crate::items::JokerTypes::*;
use crate::random::{ItemChoice, Random};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use strum::IntoEnumIterator;

pub mod items;
pub mod pools;
pub mod random;
pub mod util;
pub mod shop;
mod deck;

pub const MAX_SEED_LENGTH: usize = 8;
pub const SEED_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[inline]
fn index_to_seed_bytes(mut idx: u64, out: &mut [u8; MAX_SEED_LENGTH]) {
    // little-endian (lowest digit first) then reverse to produce human-readable
    for i in (0..MAX_SEED_LENGTH).rev() {
        let digit = (idx % 36) as usize;
        out[i] = SEED_CHARSET[digit];
        idx /= 36;
    }
}

fn test_passes_for_seed_bytes(seed_bytes: &[u8; MAX_SEED_LENGTH], items_len: i32) -> bool {
    let mut random = Random::new(&seed_bytes[..]);
    true
}

fn brute_force_range(start: u64, count: u64, items_len: i32, result_counter: &AtomicU64) {
    // We'll iterate indices in parallel using rayon's par_iter over a range
    (0..count).into_par_iter().for_each(|i| {
        let idx = start + i;
        let mut seed = [0u8; MAX_SEED_LENGTH];
        index_to_seed_bytes(idx, &mut seed);
        if test_passes_for_seed_bytes(&seed, items_len) {
            result_counter.fetch_add(1, Ordering::Relaxed);
            // optionally collect found seeds into a shared lock (Vec + Mutex) or write to disk
        }
    });
}

fn main() {
    let items_len = 100; // adjust
    let result_counter = AtomicU64::new(0);
    let start = Instant::now();
    // Example: test first 10 million seeds
    brute_force_range(0, 100_000_000, items_len, &result_counter);
    println!("found: {} in {:?}", result_counter.load(Ordering::Relaxed), start.elapsed());

    /*
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
    ]);*/
}

/*
fn test_seed_recursive(useless: &HashSet<JokerTypes>, parent: &str, index: usize) {

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
    if !shop.generate::<8>().refreshable.iter().all(|item| match item {
        ShopItem::Joker(joker) => useless.contains(&joker.joker),
        _ => true,
    }) {
        return;
    }

    let mut shop = Shop::new(2, ShopRates::default(), &mut random);
    if !shop.generate::<8>().refreshable.iter().all(|item| match item {
        ShopItem::Joker(joker) => useless.contains(&joker.joker),
        _ => true,
    }) {
        return;
    }


    println!("Hit! Seed: {}", random.seed);
}*/