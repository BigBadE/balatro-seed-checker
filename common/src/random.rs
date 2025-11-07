use crate::util::{pseudohash_bytes, round13, LuaRandom};
use libm::{floor, fma};
use core::mem::MaybeUninit;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::collections::btree_map::Entry;
use alloc::borrow::ToOwned;

pub struct Random {
    // Inline seed storage (up to 8 bytes) to avoid external lifetimes and minimize memory.
    seed_inline: [u8; 8],
    seed_inline_len: u8,
    // Full seed string for string-based IDs (Blueprint parity)
    seed_full: String,
    pub hashed_seed: f64,
    // JS-style hashed seed for string-id path (computed via pseudohash_bytes over seed_full)
    hashed_seed_js: f64,
    pub lua_random: LuaRandom,
    // Lazily initialized node values; avoid touching memory on construction by
    // storing the entire array in a single MaybeUninit and tracking per-index init.
    nodes: MaybeUninit<[f64; IDS_LEN]>,
    init_mask: [u64; (IDS_LEN + 63) / 64],
    // String-keyed nodes to mirror Blueprint's id+seed hashing (e.g., "boss1", "Voucher1", "Tarotsho1_resample2")
    str_nodes: BTreeMap<String, f64>,
}

impl Default for Random {
    fn default() -> Self {
        Self {
            seed_inline: [0; 8],
            seed_inline_len: 0,
            seed_full: String::new(),
            hashed_seed: 0.0,
            hashed_seed_js: 0.0,
            lua_random: LuaRandom::default(),
            nodes: MaybeUninit::uninit(),
            init_mask: [0u64; (IDS_LEN + 63) / 64],
            str_nodes: BTreeMap::new(),
        }
    }
}

pub const RESAMPLE_IDS_LEN: usize = 10;
// Support many independent RNG groups (streams x sources). Keep a safety margin.
pub const IDS_LEN: usize = 6000;

pub enum RandIds {
    SEED = 0
}

// node_mapping removed; we directly incorporate group/resample indices into the hash.

pub trait ItemChoice {
    fn retry(&self) -> bool;

    fn locked(&self) -> bool;
}

impl Random {
    #[inline(always)]
    pub fn new(seed: &[u8]) -> Self {
        // Avoid Default's eager array setup to keep construction cheap.
        let mut s = Self {
            seed_inline: [0; 8],
            seed_inline_len: 0,
            seed_full: String::new(),
            hashed_seed: 0.0,
            hashed_seed_js: 0.0,
            lua_random: LuaRandom::empty(),
            nodes: MaybeUninit::uninit(),
            init_mask: [0u64; (IDS_LEN + 63) / 64],
            str_nodes: BTreeMap::new(),
        };
        s.set_seed_bytes(seed);
        s.seed_full = core::str::from_utf8(seed).map(|s| s.to_owned()).unwrap_or_else(|_| {
            // fallback: lossy
            String::from_utf8_lossy(seed).into_owned()
        });
        s.hashed_seed = pseudohash_bytes([s.seed_bytes()]);
        s.hashed_seed_js = Self::pseudohash_js_str(&s.seed_full);
        s
    }

    #[inline(always)]
    pub fn reset_seed(&mut self, seed: &[u8]) {
        self.set_seed_bytes(seed);
        self.seed_full = core::str::from_utf8(seed).map(|s| s.to_owned()).unwrap_or_else(|_| {
            String::from_utf8_lossy(seed).into_owned()
        });
        self.hashed_seed = pseudohash_bytes([self.seed_bytes()]);
        self.hashed_seed_js = Self::pseudohash_js_str(&self.seed_full);
        // Clear initialization mask so nodes will be lazily recomputed for the new seed
        for m in &mut self.init_mask {
            *m = 0;
        }
        // Lua RNG will be reseeded on demand
        self.lua_random = LuaRandom::empty();
        // Clear string-node cache for new seed
        self.str_nodes.clear();
    }

    #[inline(always)]
    fn set_seed_bytes(&mut self, seed: &[u8]) {
        let len = if seed.len() > 8 { 8 } else { seed.len() };
        self.seed_inline[..len].copy_from_slice(&seed[..len]);
        self.seed_inline_len = len as u8;
    }

    #[inline(always)]
    fn seed_bytes(&self) -> &[u8] {
        &self.seed_inline[..self.seed_inline_len as usize]
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
            let val: f64 = if grp == 0 {
                // Backward-compatible hashing for base group 0 to preserve determinism of existing tests
                if res != 0 {
                    let mut buf_res = [0u8; 4];
                    pseudohash_bytes([
                        self.seed_bytes(),
                        b"_resample",
                        Self::itoa_usize_bytes(&mut buf_res, res),
                        self.seed_bytes(),
                    ])
                } else {
                    // id == 0
                    pseudohash_bytes([self.seed_bytes(), self.seed_bytes()])
                }
            } else {
                // New generalized hashing for other groups
                if res != 0 {
                    let mut buf_grp = [0u8; 4];
                    let mut buf_res = [0u8; 4];
                    pseudohash_bytes([
                        b"group",
                        Self::itoa_usize_bytes(&mut buf_grp, grp),
                        b"_resample",
                        Self::itoa_usize_bytes(&mut buf_res, res),
                        self.seed_bytes(),
                    ])
                } else {
                    let mut buf_grp = [0u8; 4];
                    pseudohash_bytes([
                        b"group",
                        Self::itoa_usize_bytes(&mut buf_grp, grp),
                        self.seed_bytes(),
                    ])
                }
            };
            // Write initialized value and set bit
            unsafe { (*self.nodes.as_mut_ptr())[id] = val };
            self.init_mask[word] |= bit;
        }
        // Safe: guarded by init_mask bit above
        let current = unsafe { (*self.nodes.as_ptr())[id] };
        // Use fused multiply-add and fast fractional part extraction to avoid costly `% 1.0`.
        let t = fma(current, 1.72431234, 2.134453429141);
        let advanced = round13(t - floor(t));
        // Update stored node value
        unsafe { (*self.nodes.as_mut_ptr())[id] = advanced };
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
    fn itoa_usize_bytes<'b>(buf: &'b mut [u8; 4], mut n: usize) -> &'b [u8] {
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

    // ---------------------- String-based helpers (QueueName parity) ----------------------
    #[inline(always)]
    // JS-style pseudohash over a single string (no fused ops)
    fn pseudohash_js_str(s: &str) -> f64 {
        let bytes = s.as_bytes();
        let mut num = 1.0f64;
        // iterate from end to start
        let mut i = bytes.len() as i32;
        while i > 0 {
            let b = bytes[(i - 1) as usize] as f64;
            // t = (1.1239285023/num) * b * PI + PI * i
            let t = (1.1239285023 / num) * b * core::f64::consts::PI
                + core::f64::consts::PI * (i as f64);
            num = t - floor(t);
            i -= 1;
        }
        num
    }

    // JS-style pseudohash over concatenation of two strings: id + seed
    fn pseudohash_js_concat(id: &str, seed: &str) -> f64 {
        // Build concatenated byte buffer
        let mut buf = alloc::vec::Vec::with_capacity(id.len() + seed.len());
        buf.extend_from_slice(id.as_bytes());
        buf.extend_from_slice(seed.as_bytes());
        let mut num = 1.0f64;
        let mut i = buf.len() as i32;
        while i > 0 {
            let b = buf[(i - 1) as usize] as f64;
            let t = (1.1239285023 / num) * b * core::f64::consts::PI
                + core::f64::consts::PI * (i as f64);
            num = t - floor(t);
            i -= 1;
        }
        num
    }
    #[inline(always)]
    pub(crate) fn get_node_str(&mut self, id: &str) -> f64 {
        // Lookup or initialize node value for this string id
        let entry = if let Entry::Occupied(e) = self.str_nodes.entry(String::from(id)) {
            e.into_mut()
        } else {
            // compute initial without borrowing self across map mutation
            // JS uses pseudohash(id + seed) on the concatenated string
            let init = Self::pseudohash_js_concat(id, &self.seed_full);
            self.str_nodes.insert(String::from(id), init);
            // safe to unwrap since just inserted
            self.str_nodes.get_mut(id).unwrap()
        };
        // Progression and final mix mirrors JS (avoid fused operations)
        let t = (*entry * 1.72431234) + 2.134453429141f64;
        let advanced = round13(t - floor(t));
        *entry = advanced;
        (advanced + self.hashed_seed_js) / 2.0
    }

    #[inline(always)]
    pub(crate) fn debug_node_str(&mut self, id: &str) -> (f64, f64, f64) {
        // initial c without mutating cache yet
        let c = if let Some(v) = self.str_nodes.get(id) {
            *v
        } else {
            Self::pseudohash_js_concat(id, &self.seed_full)
        };
        let t = (c * 1.72431234) + 2.134453429141f64;
        let value = round13(t - floor(t));
        let mixed = (value + self.hashed_seed_js) / 2.0;
        // advance cache like get_node_str
        self.str_nodes.insert(String::from(id), value);
        (c, value, mixed)
    }

    #[inline(always)]
    pub fn random_str(&mut self, id: &str) -> f64 {
        self.lua_random = LuaRandom::new(self.get_node_str(id));
        self.lua_random.random()
    }

    #[inline(always)]
    pub fn rand_int_str(&mut self, id: &str, min: i32, max: i32) -> i32 {
        self.lua_random = LuaRandom::new(self.get_node_str(id));
        self.lua_random.randint(min, max)
    }

    pub fn rand_choice_str<'b, T: ItemChoice>(&mut self, id: &str, items: &'b [T]) -> &'b T {
        // Initial draw
        self.lua_random = LuaRandom::new(self.get_node_str(id));
        let mut item = &items[self.lua_random.randint(0, items.len() as i32 - 1) as usize];
        if item.locked() || item.retry() {
            for resample in 2usize..RESAMPLE_IDS_LEN {
                let res_id = {
                    let mut s = String::from(id);
                    s.push_str("_resample");
                    // write resample number (<= 4 digits as in usize->itoa used elsewhere)
                    let mut buf = [0u8; 4];
                    let digits = Self::itoa_usize_bytes(&mut buf, resample);
                    s.push_str(core::str::from_utf8(digits).unwrap_or(""));
                    s
                };
                self.lua_random = LuaRandom::new(self.get_node_str(&res_id));
                item = &items[self.lua_random.randint(0, items.len() as i32 - 1) as usize];
                if !item.retry() && !item.locked() {
                    return item;
                }
            }
            Self::no_item_found();
        }
        item
    }
}