use cuda_std::prelude::*;
use common::random::Random;
use core::str;

const CHARSET: [u8; 36] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[inline(always)]
fn encode_seed(mut n: u64, out: &mut [u8; 8]) -> usize {
    // Encode n in base-36 using our custom charset. Return length in bytes.
    // 0 maps to 'A'. Variable length without leading symbols.
    let mut i = 0usize;
    if n == 0 {
        out[0] = CHARSET[0];
        return 1;
    }
    while n > 0 && i < 8 {
        let rem = (n % 36) as usize;
        out[i] = CHARSET[rem];
        n /= 36;
        i += 1;
    }
    // reverse in-place for the produced length
    for j in 0..(i / 2) {
        out.swap(j, i - 1 - j);
    }
    i
}

#[kernel]
pub unsafe fn iterate_seeds(start: u64, total: u64, out_checksums: *mut f64) {
    let tid = thread::index_1d() as u64;
    let block_dim = thread::block_dim_x() as u64;
    let block_idx = thread::block_idx_x() as u64;
    let grid_dim = thread::grid_dim_x() as u64;

    let global_idx = block_idx * block_dim + tid; // thread id across grid
    let stride = grid_dim * block_dim; // grid-stride loop

    let mut sum = 0.0f64;
    let mut seed_buf = [0u8; 8];

    let mut i = global_idx;
    while i < total {
        let idx = start + i;
        let len = encode_seed(idx, &mut seed_buf);
        // Safe because we only use ASCII from fixed charset
        let seed_str = unsafe { str::from_utf8_unchecked(&seed_buf[..len]) };
        let rng = Random::new(seed_str);
        // Accumulate hashed_seed to avoid being optimized out
        sum += rng.hashed_seed;
        i += stride;
    }

    let out_ptr = unsafe { out_checksums.add(tid as usize) };
    unsafe { *out_ptr = sum; }
}