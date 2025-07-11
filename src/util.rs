use crate::items::{Bosses, Vouchers};

pub struct GameState {
    ante: i32,
    seen_bosses: Vec<Bosses>,
    vouchers: Vec<Vouchers>,
}

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
    pub fn new(seed: f64) -> Self {
        let mut returning = Self {
            state: [0; 4],
        };
        let mut d = seed;
        let mut r = 0x11090601;
        for i in 0..4 {
            let m = 1 << (r & 255);
            r >>= 8;
            d = d * std::f64::consts::PI + std::f64::consts::E;
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

    fn _randint(&mut self) -> u64 {
        let mut z;
        let mut r = 0;
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

    pub fn randdblmem(&mut self) -> u64 {
        (self._randint() & 0xFFFFFFFFFFFFF) | 0x3FF0000000000000
    }

    pub fn random(&mut self) -> f64 {
        f64::from_bits(self.randdblmem()) - 1.0
    }

    pub fn randint(&mut self, min: i32, max: i32) -> i32 {
        let rand = self.random();
        (rand * (max - min + 1) as f64) as i32 + min
    }
}

pub fn pseudohash(s: &str) -> f64 {
    let mut num = 1.0f64;
    for i in (1..=s.len()).rev() {
        num = (1.1239285023 / num * s.as_bytes()[i - 1] as f64 * std::f64::consts::PI + std::f64::consts::PI * i as f64) % 1.0;
    }
    if num.is_nan() {
        // TODO quiet NaN
    }
    num
}

fn inv_prec() -> f64 {
    10.0f64.powi(13)
}

fn two_inv_prec() -> f64 {
    2.0f64.powi(13)
}

fn five_inv_prec() -> f64 {
    5.0f64.powi(13)
}

pub fn round13(x: f64) -> f64 {
    let tentative = (x * inv_prec()).floor() / inv_prec();
    let truncated = x * two_inv_prec() % 1.0 * five_inv_prec();
    if tentative != x && tentative != x.next_up() && truncated % 1.0 >= 0.5 {
        return ((x * inv_prec()).floor() + 1.0) / inv_prec();
    }
    tentative
}