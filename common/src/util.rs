use libm::{floor, fma};
use core::f64::consts::{PI, E};

#[derive(Debug)]
pub struct LuaRandom {
    state: [i128; 4],
}

impl Default for LuaRandom {
    fn default() -> Self {
        LuaRandom::new(0.0)
    }
}

impl LuaRandom {
    #[inline(always)]
    pub fn empty() -> Self {
        // Construct without seeding/warm-up; suitable as a placeholder
        // when the RNG will be reseeded immediately before use.
        Self { state: [0; 4] }
    }

    #[inline(always)]
    pub fn new(seed: f64) -> Self {
        let mut returning = Self {
            state: [0; 4],
        };
        let mut d = seed;
        let mut r = 0x11090601;
        for i in 0..4 {
            let m = 1u64 << (r & 255);
            r >>= 8;
            d = d * PI + E;
            // Interpret the f64's little-endian byte layout as u64, matching JS Uint8Array assembly.
            let mut ulong_val = u64::from_le_bytes(d.to_le_bytes());
            if ulong_val < m {
                ulong_val += m;
            }
            returning.state[i] = ulong_val as i128;
        }
        for _ in 0..10 {
            returning._randint();
        }

        returning
    }

    #[inline(always)]
    fn _randint(&mut self) -> u64 {
        // Emulate JS BigInt behavior using 128-bit intermediates.
        let mask64: u128 = (1u128 << 64) - 1;
        let mut r_acc: u128 = 0;
        // Use arithmetic right shifts by interpreting as i64, matching JS BigInt >> behavior.
        // State 0
        let z0 = self.state[0] as u128;
        let a0 = (((z0 << 31) ^ z0) as i128) >> 45; // arithmetic right shift
        let b0 = a0 as u128;
        let c0 = (z0 & ((mask64 << 1))) << 18;
        let z = b0 ^ c0;
        r_acc ^= z;
        self.state[0] = z as i128;
        // State 1
        let z1 = self.state[1] as u128;
        let a1 = (((z1 << 19) ^ z1) as i128) >> 30;
        let b1 = a1 as u128;
        let c1 = (z1 & ((mask64 << 6))) << 28;
        let z = b1 ^ c1;
        r_acc ^= z;
        self.state[1] = z as i128;
        // State 2
        let z2 = self.state[2] as u128;
        let a2 = (((z2 << 24) ^ z2) as i128) >> 48;
        let b2 = a2 as u128;
        let c2 = (z2 & ((mask64 << 9))) << 7;
        let z = b2 ^ c2;
        r_acc ^= z;
        self.state[2] = z as i128;
        // State 3
        let z3 = self.state[3] as u128;
        let a3 = (((z3 << 21) ^ z3) as i128) >> 39;
        let b3 = a3 as u128;
        let c3 = (z3 & ((mask64 << 17))) << 8;
        let z = b3 ^ c3;
        r_acc ^= z;
        self.state[3] = z as i128;
        // Return lower 64 bits like JS when extracting for double bits
        (r_acc & mask64) as u64
    }

    #[inline(always)]
    pub fn randdblmem(&mut self) -> u64 {
        (self._randint() & 0xFFFFFFFFFFFFF) | 0x3FF0000000000000
    }

    #[inline(always)]
    pub fn random(&mut self) -> f64 {
        f64::from_bits(self.randdblmem()) - 1.0
    }

    #[inline(always)]
    pub fn randint(&mut self, min: i32, max: i32) -> i32 {
        let rand = self.random();
        (rand * (max - min + 1) as f64) as i32 + min
    }

    #[inline(always)]
    pub fn debug_first_rand_bits(seed: f64) -> (u64, f64) {
        let mut lr = LuaRandom::new(seed);
        let bits = lr.randdblmem();
        let val = f64::from_bits(bits) - 1.0;
        (bits, val)
    }

    #[inline(always)]
    pub fn debug_seed_states(seed: f64) -> [u64; 4] {
        let lr = LuaRandom::new(seed);
        [
            (lr.state[0] as u128 & ((1u128<<64)-1)) as u64,
            (lr.state[1] as u128 & ((1u128<<64)-1)) as u64,
            (lr.state[2] as u128 & ((1u128<<64)-1)) as u64,
            (lr.state[3] as u128 & ((1u128<<64)-1)) as u64,
        ]
    }

    #[inline(always)]
    pub fn debug_first_rand_steps(seed: f64) -> ([u128; 4], u128) {
        // Recompute the exact steps of the first _randint pass for debugging
        let mut lr = LuaRandom::new(seed);
        let mask64: u128 = (1u128 << 64) - 1;
        let mut r_acc: u128 = 0;
        let mut zs: [u128; 4] = [0; 4];

        // State 0
        let z0 = lr.state[0] as u128;
        let a0 = (((z0 << 31) ^ z0) as i128) >> 45;
        let b0 = a0 as u128;
        let c0 = (z0 & (mask64 << 1)) << 18;
        let z = b0 ^ c0;
        r_acc ^= z;
        lr.state[0] = z as i128;
        zs[0] = z;

        // State 1
        let z1 = lr.state[1] as u128;
        let a1 = (((z1 << 19) ^ z1) as i128) >> 30;
        let b1 = a1 as u128;
        let c1 = (z1 & (mask64 << 6)) << 28;
        let z = b1 ^ c1;
        r_acc ^= z;
        lr.state[1] = z as i128;
        zs[1] = z;

        // State 2
        let z2 = lr.state[2] as u128;
        let a2 = (((z2 << 24) ^ z2) as i128) >> 48;
        let b2 = a2 as u128;
        let c2 = (z2 & (mask64 << 9)) << 7;
        let z = b2 ^ c2;
        r_acc ^= z;
        lr.state[2] = z as i128;
        zs[2] = z;

        // State 3
        let z3 = lr.state[3] as u128;
        let a3 = (((z3 << 21) ^ z3) as i128) >> 39;
        let b3 = a3 as u128;
        let c3 = (z3 & (mask64 << 17)) << 8;
        let z = b3 ^ c3;
        r_acc ^= z;
        lr.state[3] = z as i128;
        zs[3] = z;

        (zs, r_acc)
    }
}

#[inline(always)]
pub fn pseudohash_bytes<const SIZE: usize>(s: [&[u8]; SIZE]) -> f64 {
    let mut num = 1.0f64;
    // Fold constant: C = 1.1239285023 * PI
    const C: f64 = 1.1239285023 * PI;
    for bytes in s.iter().rev() {
        let mut i = bytes.len() as i32;
        // Precompute PI * i and decrement by PI in the loop to avoid a mul per step
        let mut pi_term = PI * i as f64;
        while i > 0 {
            // SAFETY: i ranges from bytes.len() down to 1, so (i-1) is always a valid index
            let b = unsafe { *bytes.get_unchecked((i - 1) as usize) } as f64;
            let inv = 1.0 / num;
            // t = C*b*inv + pi_term, use libm::fma to fuse when available
            let t = fma(C * b, inv, pi_term);
            // fast fract for non-negative t: t - floor(t)
            num = t - floor(t);
            i -= 1;
            pi_term -= PI;
        }
    }
    num
}

#[inline(always)]
pub fn pseudohash<const SIZE: usize>(s: [&str; SIZE]) -> f64 {
    let mut arr: [&[u8]; SIZE] = [&[]; SIZE];
    for (i, w) in s.iter().enumerate() {
        arr[i] = w.as_bytes();
    }
    pseudohash_bytes(arr)
}

const INV_PREC: f64 = 1e13; // 10^13
const TWO_INV_PREC: f64 = 8192.0; // 2^13
const FIVE_INV_PREC: f64 = 1220703125.0; // 5^13

pub fn round13(x: f64) -> f64 {
    let scaled = x * INV_PREC;
    let floored = floor(scaled);
    let tentative = floored / INV_PREC;
    let truncated = (x * TWO_INV_PREC) % 1.0 * FIVE_INV_PREC;
    // JS uses a simplified nextAfter: x +/- Number.EPSILON. In Game.ts it's nextAfter(x, 1)
    // which for 0<=x<=1 is always x + EPSILON.
    let x_next = x + f64::EPSILON;
    if tentative != x && tentative != x_next && truncated % 1.0 >= 0.5 {
        return (floored + 1.0) / INV_PREC;
    }

    tentative
}