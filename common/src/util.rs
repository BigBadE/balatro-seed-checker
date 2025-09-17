use libm::{floor, nextafter, fma};
use core::f64::consts::{PI, E};

#[derive(Debug)]
pub struct LuaRandom {
    state: [u64; 4],
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
            let mut ulong_val = d.to_bits();
            if ulong_val < m {
                ulong_val += m;
            }
            returning.state[i] = ulong_val;
        }
        for _ in 0..10 {
            returning._randint();
        }

        returning
    }

    #[inline(always)]
    fn _randint(&mut self) -> u64 {
        let mut z;
        let mut r = 0u64;
        z = self.state[0];
        z = (((z << 31) ^ z) >> 45) ^ ((z & (u64::MAX << 1)) << 18);
        r ^= z;
        self.state[0] = z;
        z = self.state[1];
        z = (((z << 19) ^ z) >> 30) ^ ((z & (u64::MAX << 6)) << 28);
        r ^= z;
        self.state[1] = z;
        z = self.state[2];
        z = (((z << 24) ^ z) >> 48) ^ ((z & (u64::MAX << 9)) << 7);
        r ^= z;
        self.state[2] = z;
        z = self.state[3];
        z = (((z << 21) ^ z) >> 39) ^ ((z & (u64::MAX << 17)) << 8);
        r ^= z;
        self.state[3] = z;
        r
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
    if tentative != x && tentative != nextafter(x, f64::INFINITY) && truncated % 1.0 >= 0.5 {
        return (floored + 1.0) / INV_PREC;
    }
    tentative
}