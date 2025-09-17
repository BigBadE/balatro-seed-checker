use crate::util::{pseudohash, round13, LuaRandom};

pub struct Random<'a> {
    pub seed: &'a str,
    pub hashed_seed: f64,
    pub lua_random: LuaRandom,
    pub nodes: [f64; 53],
}

impl Default for Random<'_> {
    fn default() -> Self {
        Self {
            seed: "",
            hashed_seed: 0.0,
            lua_random: LuaRandom::default(),
            nodes: [f64::NAN; 53],
        }
    }
}

pub const SEED: usize = 51;

fn node_mapping<'a>(_seed: &'a str, node: usize) -> &'a str {
    match node {

        _ => panic!("Invalid node"),
    }
}

pub trait ItemChoice {
    fn retry(&self) -> bool;

    fn locked(&self) -> bool;
}

impl<'a> Random<'a> {
    pub fn new(seed: &'a str) -> Self {
        Self { hashed_seed: pseudohash([seed]), seed, ..Self::default() }
    }

    pub fn get_node(&mut self, id: usize) -> f64 {
        if self.nodes[id].is_nan() {
            self.nodes[id] = pseudohash([node_mapping(self.seed, id), self.seed]);
        }
        self.nodes[id] = round13((self.nodes[id] * 1.72431234 + 2.134453429141) % 1.0);
        (self.nodes[id] + self.hashed_seed) / 2.0
    }

    pub fn random(&mut self, id: usize) -> f64 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.random()
    }

    pub fn rand_int(&mut self, id: usize, min: i32, max: i32) -> i32 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.randint(min, max)
    }

    pub fn rand_choice<'b, T: ItemChoice + Debug>(&mut self, id: &str, items: &'b [T]) -> &'b T {
        self.lua_random = LuaRandom::new(self.get_node(id));
        let mut item = &items[self.lua_random.randint(0, items.len() as i32-1) as usize];
        // TODO showman check
        if item.locked() || item.retry() {
            for resample in 2..1000 {
                self.lua_random = LuaRandom::new(self.get_node("{id}_resample{resample}")));
                item = &items[self.lua_random.randint(0, items.len() as i32-1) as usize];
                if !item.retry() && !item.locked() {
                    return item;
                }
            }
        }
        item
    }

    #[cfg(feature = "std")]
    pub fn rand_weighted_choice<'b, T: ItemChoice>(&mut self, id: &str, items: &'b Vec<(f64, T)>) -> &'b T {
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