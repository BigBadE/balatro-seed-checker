use libm::{floor, nextafter, pow};

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
            d = d * core::f64::consts::PI + core::f64::consts::E;
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

pub fn pseudohash<const SIZE: usize>(s: [&str; SIZE]) -> f64 {
    let mut num = 1.0f64;
    for word in s.iter().rev() {
        for i in (1..=word.len()).rev() {
            num = (1.1239285023 / num * word.as_bytes()[i - 1] as f64 * core::f64::consts::PI + core::f64::consts::PI * i as f64) % 1.0;
        }
    }

    if num.is_nan() {
        // TODO quiet NaN
    }
    num
}

fn inv_prec() -> f64 { pow(10.0, 13.0) }

fn two_inv_prec() -> f64 { pow(2.0, 13.0) }

fn five_inv_prec() -> f64 { pow(5.0, 13.0) }

pub fn round13(x: f64) -> f64 {
    let tentative = floor(x * inv_prec()) / inv_prec();
    let truncated = x * two_inv_prec() % 1.0 * five_inv_prec();
    if tentative != x && tentative != nextafter(x, core::f64::INFINITY) && truncated % 1.0 >= 0.5 {
        return (floor(x * inv_prec()) + 1.0) / inv_prec();
    }
    tentative
}