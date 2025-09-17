use crate::util::{pseudohash_bytes, round13, LuaRandom};

pub struct Random<'a> {
    pub seed: &'a [u8],
    pub hashed_seed: f64,
    pub lua_random: LuaRandom,
    pub nodes: [f64; IDS_LEN],
}

impl Default for Random<'_> {
    fn default() -> Self {
        Self {
            seed: &[],
            hashed_seed: 0.0,
            lua_random: LuaRandom::default(),
            nodes: [f64::NAN; IDS_LEN],
        }
    }
}

pub const RESAMPLE_IDS_LEN: usize = 5;
pub const IDS_LEN: usize = 1 * RESAMPLE_IDS_LEN;

pub enum RandIds {
    SEED = 0
}

#[inline(always)]
fn node_mapping<'a>(seed: &'a [u8], node: usize) -> &'a [u8] {
    match node {
        0 => seed,
        _ => panic!("Invalid node"),
    }
}

pub trait ItemChoice {
    fn retry(&self) -> bool;

    fn locked(&self) -> bool;
}

impl<'a> Random<'a> {
    #[inline(always)]
    pub fn new(seed: &'a [u8]) -> Self {
        Self { hashed_seed: pseudohash_bytes([seed]), seed, ..Self::default() }
    }

    #[inline(always)]
    pub fn get_node(&mut self, id: usize) -> f64 {
        if self.nodes[id].is_nan() {
            let res = id % RESAMPLE_IDS_LEN;
            let grp = id / RESAMPLE_IDS_LEN;
            if res != 0 {
                let mut buf = [0u8; 20];
                self.nodes[id] = pseudohash_bytes([
                    node_mapping(self.seed, grp),
                    b"_resample",
                    Self::itoa_usize_bytes(&mut buf, res),
                    self.seed,
                ]);
            } else {
                self.nodes[id] = pseudohash_bytes([node_mapping(self.seed, id), self.seed]);
            }
        }
        self.nodes[id] = round13((self.nodes[id] * 1.72431234 + 2.134453429141) % 1.0);
        (self.nodes[id] + self.hashed_seed) / 2.0
    }

    #[inline(always)]
    pub fn random(&mut self, id: usize) -> f64 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.random()
    }

    #[inline(always)]
    pub fn rand_int(&mut self, id: usize, min: i32, max: i32) -> i32 {
        self.lua_random = LuaRandom::new(self.get_node(id));
        self.lua_random.randint(min, max)
    }

    pub fn rand_choice<'b, T: ItemChoice>(&mut self, id: usize, items: &'b [T]) -> &'b T {
        // Initial draw
        self.lua_random = LuaRandom::new(self.get_node(id));
        let mut item = &items[self.lua_random.randint(0, items.len() as i32 - 1) as usize];

        // If the item is not usable, deterministically resample with new per-attempt IDs,
        // using a stack buffer to avoid allocations.
        if item.locked() || item.retry() {
            for resample in 2usize..RESAMPLE_IDS_LEN {
                // Target string should be: "{id}_resample{resample}"
                self.lua_random = LuaRandom::new(self.get_node(id + resample));
                item = &items[self.lua_random.randint(0, items.len() as i32 - 1) as usize];
                if !item.retry() && !item.locked() {
                    return item;
                }
            }
            Self::no_item_found();
        }
        item
    }

    #[cold]
    fn no_item_found() -> ! {
        panic!("Failed to find a usable item!");
    }

    // Convert usize to decimal string in a stack buffer; returns the string slice within `buf`.
    #[inline(always)]
    fn itoa_usize_bytes<'b>(buf: &'b mut [u8; 20], mut n: usize) -> &'b [u8] {
        let mut i = buf.len();
        if n == 0 {
            i -= 1;
            buf[i] = b'0';
        } else {
            while n > 0 {
                i -= 1;
                buf[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }
        }
        &buf[i..]
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