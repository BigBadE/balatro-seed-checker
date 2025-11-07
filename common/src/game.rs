use alloc::vec::Vec;
use crate::items::{Bosses, JokerTypes, Pack, Planets, Tarots, Vouchers, RandomSource, Tags};
use crate::random::Random;
use crate::util::LuaRandom;
use libm::floor;
use strum::IntoEnumIterator;
use alloc::string::String;
use crate::names::voucher_name;
use crate::lock::Lock;

// Streams used to derive deterministic RNG nodes for each generator.
// Keep values small and distinct; resampling is handled by Random internally.
#[derive(Copy, Clone)]
pub enum RngStream {
    Joker = 0,
    Tarot = 1,
    Planet = 2,
    Spectral = 3,
    Pack = 4,
    Voucher = 5,
    Tag = 6,
    Boss = 7,
    Standard = 8,
}

// Records all draws made so far to aid reproducibility and analysis.
#[derive(Default)]
pub struct SeenLog {
    pub jokers: Vec<JokerTypes>,
    pub tarots: Vec<Tarots>,
    pub planets: Vec<Planets>,
    pub packs: Vec<Pack>,
    pub bosses: Vec<Bosses>,
    pub vouchers: Vec<Vouchers>,
}

pub struct GameState {
    rng: Random,
    pub ante: i32,
    pub seen: SeenLog,
    lock: Lock,
}

impl GameState {
    #[inline]
    pub fn new(seed: &str, ante: i32) -> Self {
        let mut s = Self {
            rng: Random::new(seed.as_bytes()),
            ante,
            seen: SeenLog::default(),
            lock: Lock::new(),
        };
        // Initialize locks per Blueprint lifecycle
        // fresh_profile = true, fresh_run = false
        s.lock.init_locks(ante, true, false);
        // Also apply firstLock (level two vouchers and related items)
        s.lock.lock_level_two_vouchers();
        s
    }

    #[inline(always)]
    fn source_code(source: RandomSource) -> &'static str {
        match source {
            RandomSource::Shop => "sho",
            RandomSource::BuffonPack => "buf",
            RandomSource::Wraith => "wra",
            RandomSource::RareTag => "rta",
            RandomSource::UncommonTag => "uta",
            RandomSource::Soul => "sou",
            RandomSource::Arcana => "ar1",
            RandomSource::Celestial => "pl1",
        }
    }

    #[inline]
    pub fn reset_seed(&mut self, seed: &str) {
        self.rng.reset_seed(seed.as_bytes());
        self.clear_seen();
    }

    #[inline]
    pub fn clear_seen(&mut self) {
        self.seen = SeenLog::default();
    }

    #[inline]
    pub fn lock_level_two_vouchers(&mut self) {
        self.lock.lock_level_two_vouchers();
    }

    // Mirror Blueprint: mark voucher as owned/locked and unlock the next voucher in sequence
    #[inline]
    pub fn activate_voucher(&mut self, v: Vouchers) {
        let name = voucher_name(&v);
        self.lock.lock(name);
        // Find voucher index and unlock next if exists
        let all: Vec<Vouchers> = Vouchers::iter().collect();
        if let Some(idx) = all.iter().position(|x| voucher_name(x) == name) {
            if idx + 1 < all.len() {
                let next_name = voucher_name(&all[idx + 1]);
                self.lock.unlock(next_name);
            }
        }
    }

    

    #[inline]
    pub fn apply_unlocks<I: IntoIterator<Item=String>>(&mut self, names: I) {
        for n in names { self.lock.unlock(&n); }
    }

    // Generators -----------------------------------------------------------------

    // Compute a distinct RNG group base per (category, source).
    // Group size equals RESAMPLE_IDS_LEN.
    #[inline(always)]
    fn id_base_for(stream: RngStream, source: RandomSource) -> usize {
        const GROUP: usize = crate::random::RESAMPLE_IDS_LEN;
        // Table-driven minimal mapping aligned with reference expectations.
        // Reserve group 0 for Boss, group 1 for Voucher (shop),
        // group 2.. for primary shop draws (Joker/Tarot/Planet/Pack) as needed.
        match stream {
            RngStream::Boss => 0 * GROUP,
            RngStream::Voucher => 1 * GROUP,
            RngStream::Joker => match source {
                RandomSource::Shop => 2 * GROUP,
                RandomSource::BuffonPack => 3 * GROUP,
                RandomSource::Wraith => 4 * GROUP,
                RandomSource::RareTag => 5 * GROUP,
                RandomSource::UncommonTag => 6 * GROUP,
                RandomSource::Soul => 7 * GROUP,
                RandomSource::Arcana => 8 * GROUP,
                RandomSource::Celestial => 9 * GROUP,
            },
            RngStream::Tarot => match source {
                RandomSource::Shop => 8 * GROUP,
                RandomSource::Arcana => 9 * GROUP,
                _ => 9 * GROUP,
            },
            RngStream::Planet => match source {
                RandomSource::Shop => 10 * GROUP,
                RandomSource::Celestial => 11 * GROUP,
                _ => 11 * GROUP,
            },
            RngStream::Pack => match source {
                RandomSource::Shop => 12 * GROUP,
                _ => 13 * GROUP,
            },
            _ => 14 * GROUP,
        }
    }

    #[inline]
    pub fn next_joker(&mut self) -> JokerTypes {
        // Deterministic choice with retry/lock semantics handled by Random
        let all: Vec<JokerTypes> = JokerTypes::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Joker, RandomSource::Shop), &all);
        self.seen.jokers.push(choice);
        choice
    }

    #[inline]
    pub fn next_joker_from(&mut self, source: RandomSource) -> JokerTypes {
        let all: Vec<JokerTypes> = JokerTypes::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Joker, source), &all);
        self.seen.jokers.push(choice);
        choice
    }

    #[inline]
    pub fn next_tarot(&mut self) -> Tarots {
        let all: Vec<Tarots> = Tarots::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Tarot, RandomSource::Shop), &all);
        self.seen.tarots.push(choice);
        choice
    }

    #[inline]
    pub fn next_tarot_from(&mut self, source: RandomSource) -> Tarots {
        let all: Vec<Tarots> = Tarots::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Tarot, source), &all);
        self.seen.tarots.push(choice);
        choice
    }

    #[inline]
    pub fn next_tarot_from_at_ante(&mut self, source: RandomSource, ante: i32) -> Tarots {
        let all: Vec<Tarots> = Tarots::iter().collect();
        let id = alloc::format!("Tarot{}{}", Self::source_code(source), ante.max(1));
        let choice = *self.rng.rand_choice_str(&id, &all);
        self.seen.tarots.push(choice);
        choice
    }

    #[inline]
    pub fn next_planet(&mut self) -> Planets {
        let all: Vec<Planets> = Planets::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Planet, RandomSource::Shop), &all);
        self.seen.planets.push(choice);
        choice
    }

    #[inline]
    pub fn next_planet_from(&mut self, source: RandomSource) -> Planets {
        let all: Vec<Planets> = Planets::iter().collect();
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Planet, source), &all);
        self.seen.planets.push(choice);
        choice
    }

    #[inline]
    pub fn next_planet_from_at_ante(&mut self, source: RandomSource, ante: i32) -> Planets {
        let all: Vec<Planets> = Planets::iter().collect();
        let id = alloc::format!("Planet{}{}", Self::source_code(source), ante.max(1));
        let choice = *self.rng.rand_choice_str(&id, &all);
        self.seen.planets.push(choice);
        choice
    }

    #[inline]
    pub fn next_pack(&mut self) -> Pack {
        const ALL: &[Pack] = &[
            Pack::Buffoon,
            Pack::Arcana,
            Pack::Spectral,
            Pack::Planet,
        ];
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Pack, RandomSource::Shop), ALL);
        self.seen.packs.push(choice);
        choice
    }

    #[inline]
    pub fn next_pack_from(&mut self, source: RandomSource) -> Pack {
        const ALL: &[Pack] = &[
            Pack::Buffoon,
            Pack::Arcana,
            Pack::Spectral,
            Pack::Planet,
        ];
        let choice = *self.rng.rand_choice(Self::id_base_for(RngStream::Pack, source), ALL);
        self.seen.packs.push(choice);
        choice
    }

    #[inline]
    pub fn next_boss(&mut self) -> Bosses {
        // Apply per-ante unlocks as Blueprint does at start of generateAnte
        self.lock.init_unlocks(self.ante, false);
        let all: Vec<Bosses> = Bosses::iter().collect();
        let base_id = "boss";
        // resample on locked boss names
        let mut id = String::from(base_id);
        let mut resample = 2usize;
        loop {
            let mixed = self.rng.get_node_str(&id);
            self.rng.lua_random = LuaRandom::new(mixed);
            let idx = floor(self.rng.lua_random.random() * (all.len() as f64)) as usize;
            let choice = all[idx];
            if !self.lock.is_locked(crate::names::boss_name(&choice)) {
                self.seen.bosses.push(choice);
                return choice;
            }
            id.clear(); id.push_str(base_id); id.push_str("_resample"); id.push_str(&alloc::format!("{}", resample)); resample+=1;
        }
    }

    #[inline]
    pub fn next_boss_from(&mut self, source: RandomSource) -> Bosses {
        let _ = source; // source not used for boss in reference
        self.next_boss()
    }

    #[inline]
    pub fn next_boss_from_at_ante(&mut self, source: RandomSource, ante: i32) -> Bosses {
        let _ = source; // source not used for boss in reference
        let _ = ante;
        self.next_boss()
    }

    #[inline]
    pub fn next_voucher(&mut self) -> Vouchers {
        self.lock.init_unlocks(self.ante, false);
        let all: Vec<Vouchers> = Vouchers::iter().collect();
        let base_id = "Voucher1";
        let mut id = String::from(base_id);
        let mut resample = 2usize;
        loop {
            let mixed = self.rng.get_node_str(&id);
            self.rng.lua_random = LuaRandom::new(mixed);
            let idx = floor(self.rng.lua_random.random() * (all.len() as f64)) as usize;
            let choice = all[idx];
            let name = voucher_name(&choice);
            let locked = self.lock.is_locked(name);
            if !locked {
                self.seen.vouchers.push(choice);
                return choice;
            }
            id.clear(); id.push_str(base_id); id.push_str("_resample"); id.push_str(&alloc::format!("{}", resample)); resample += 1;
        }
    }

    #[inline]
    pub fn next_voucher_from(&mut self, source: RandomSource) -> Vouchers {
        let _ = source; // default to shop path unless using tag specific path
        self.next_voucher()
    }

    #[inline]
    pub fn next_voucher_from_at_ante(&mut self, source: RandomSource, ante: i32) -> Vouchers {
        let _ = source;
        // Use lock-aware version and id Voucher{ante}
        self.lock.init_unlocks(ante, false);
        let all: Vec<Vouchers> = Vouchers::iter().collect();
        let base_id = alloc::format!("Voucher{}", ante.max(1));
        let mut id = base_id.clone();
        let mut resample = 2usize;
        loop {
            let mixed = self.rng.get_node_str(&id);
            self.rng.lua_random = LuaRandom::new(mixed);
            let idx = floor(self.rng.lua_random.random() * (all.len() as f64)) as usize;
            let choice = all[idx];
            let name = voucher_name(&choice);
            if !self.lock.is_locked(name) { self.seen.vouchers.push(choice); return choice; }
            id.clear(); id.push_str(&base_id); id.push_str("_resample"); id.push_str(&alloc::format!("{}", resample)); resample += 1;
        }
    }

    #[inline]
    pub fn next_tag_from_at_ante(&mut self, ante: i32) -> Tags {
        let all: Vec<Tags> = Tags::iter().collect();
        let id = alloc::format!("Tag{}", ante.max(1));
        *self.rng.rand_choice_str(&id, &all)
    }

    #[inline]
    pub fn next_tag_k_from_at_ante(&mut self, ante: i32, k: usize) -> Tags {
        let mut last = self.next_tag_from_at_ante(ante);
        for _ in 1..k {
            last = self.next_tag_from_at_ante(ante);
        }
        last
    }

    #[inline]
    pub fn debug_tag_once(&mut self, ante: i32) -> (alloc::string::String, f64, f64, usize, &'static str) {
        let all: Vec<Tags> = Tags::iter().collect();
        let id = alloc::format!("Tag{}", ante.max(1));
        let mixed = self.rng.get_node_str(&id);
        self.rng.lua_random = LuaRandom::new(mixed);
        let rand = self.rng.lua_random.random();
        let idx = floor(rand * (all.len() as f64)) as usize;
        let name = crate::names::tag_name(&all[idx]);
        (id, mixed, rand, idx, name)
    }

    #[inline]
    pub fn next_joker_from_at_ante(&mut self, source: RandomSource, ante: i32) -> JokerTypes {
        let all: Vec<JokerTypes> = JokerTypes::iter().collect();
        let id = alloc::format!("Joker{}{}", Self::source_code(source), ante.max(1));
        let choice = *self.rng.rand_choice_str(&id, &all);
        self.seen.jokers.push(choice);
        choice
    }

}
