use crate::util::{pseudohash, round13, LuaRandom};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Default)]
pub struct Random {
    pub seed: String,
    pub hashed_seed: f64,
    pub lua_random: LuaRandom,
    pub nodes: HashMap<String, f64>,
}

pub trait ItemChoice {
    fn retry(&self) -> bool;

    fn locked(&self) -> bool;
}

impl Random {
    pub fn new(seed: String) -> Self {
        Self { hashed_seed: pseudohash(&seed), seed, ..Self::default() }
    }

    pub fn get_node(&mut self, id: &str) -> f64 {
        if self.nodes.get(id).is_none() {
            self.nodes.insert(id.to_string(), pseudohash(&format!("{id}{}", self.seed)));
        }
        self.nodes.insert(id.to_string(), round13((self.nodes[id] * 1.72431234 + 2.134453429141) % 1.0));
        (self.nodes[id] + self.hashed_seed) / 2.0
    }

    pub fn random(&mut self, id: &str) -> f64 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.random()
    }

    pub fn rand_int(&mut self, id: &str, min: i32, max: i32) -> i32 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.randint(min, max)
    }

    pub fn rand_choice<'a, T: ItemChoice + Debug>(&mut self, id: &str, items: &'a [T]) -> &'a T {
        self.lua_random = LuaRandom::new(self.get_node(id));
        let mut item = &items[self.lua_random.randint(0, items.len() as i32-1) as usize];
        // TODO showman check
        if item.locked() || item.retry() {
            for resample in 2..1000 {
                self.lua_random = LuaRandom::new(self.get_node(&format!("{id}_resample{resample}")));
                item = &items[self.lua_random.randint(0, items.len() as i32-1) as usize];
                if !item.retry() && !item.locked() {
                    return item;
                }
            }
        }
        item
    }

    pub fn rand_weighted_choice<'a, T: ItemChoice>(&mut self, id: &str, items: &'a Vec<(f64, T)>) -> &'a T {
        self.lua_random = LuaRandom::new(self.get_node(id));
        let poll = self.lua_random.random() * items[0].0;
        let mut idx = 1;
        let mut weight = 0.0f64;
        while weight < poll {
            weight += items[idx].0;
            idx += 1;
        }
        &items[idx - 1].1
    }
}