use crate::util::{pseudohash_bytes, round13, LuaRandom};
use libm::{floor, fma};
use core::mem::MaybeUninit;

pub struct Random<'a> {
    pub seed: &'a [u8],
    pub hashed_seed: f64,
    pub lua_random: LuaRandom,
    // Lazily initialized node values to avoid eager memset/fill on construction.
    // Use a compact bitset to track which indices are initialized.
    nodes: [MaybeUninit<f64>; IDS_LEN],
    init_mask: [u64; (IDS_LEN + 63) / 64],
}

impl Default for Random<'_> {
    fn default() -> Self {
        Self {
            seed: &[],
            hashed_seed: 0.0,
            lua_random: LuaRandom::default(),
            nodes: [MaybeUninit::<f64>::uninit(); IDS_LEN],
            init_mask: [0u64; (IDS_LEN + 63) / 64],
        }
    }
}

pub const RESAMPLE_IDS_LEN: usize = 1000;
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
        // Avoid Default's eager array setup to keep construction cheap.
        Self {
            seed,
            hashed_seed: pseudohash_bytes([seed]),
            lua_random: LuaRandom::empty(),
            nodes: [MaybeUninit::<f64>::uninit(); IDS_LEN],
            init_mask: [0u64; (IDS_LEN + 63) / 64],
        }
    }

    #[inline(always)]
    pub fn get_node(&mut self, id: usize) -> f64 {
        debug_assert!(id < IDS_LEN);
        let word = id >> 6;
        let bit = 1u64 << (id & 63);
        if (self.init_mask[word] & bit) == 0 {
            // Initialize lazily
            let res = id % RESAMPLE_IDS_LEN;
            let grp = id / RESAMPLE_IDS_LEN;
            let val: f64 = if res != 0 {
                let mut buf = [0u8; 20];
                pseudohash_bytes([
                    node_mapping(self.seed, grp),
                    b"_resample",
                    Self::itoa_usize_bytes(&mut buf, res),
                    self.seed,
                ])
            } else {
                pseudohash_bytes([node_mapping(self.seed, id), self.seed])
            };
            // Write initialized value and set bit
            unsafe { self.nodes.get_unchecked_mut(id).write(val) };
            self.init_mask[word] |= bit;
        }
        // Safe: guarded by init_mask bit above
        let current = unsafe { *self.nodes.get_unchecked(id).assume_init_ref() };
        // Use fused multiply-add and fast fractional part extraction to avoid costly `% 1.0`.
        let t = fma(current, 1.72431234, 2.134453429141);
        let advanced = round13(t - floor(t));
        // Update stored node value
        unsafe { self.nodes.get_unchecked_mut(id).write(advanced) };
        (advanced + self.hashed_seed) / 2.0
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
                // Stay within the same group as `id` to avoid out-of-bounds indexing.
                // Group base is the start index of the group containing `id`.
                let group_base = id - (id % RESAMPLE_IDS_LEN);
                self.lua_random = LuaRandom::new(self.get_node(group_base + resample));
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