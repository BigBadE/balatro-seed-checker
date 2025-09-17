use cuda_std::prelude::*;
use common::random::Random;

const CHARSET: [u8; 36] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[inline(always)]
fn encode_seed(mut n: u64, out: &mut [u8; 8]) -> (usize, usize) {
    // Encode n in base-36 using our custom charset. Return (start, length) in bytes.
    // 0 maps to 'A'. Variable length without leading symbols.
    let mut end = out.len(); // exclusive end index
    if n == 0 {
        end -= 1;
        out[end] = CHARSET[0];
        return (end, 1);
    }
    while n > 0 && end > 0 {
        let rem = (n % 36) as usize;
        end -= 1;
        out[end] = CHARSET[rem];
        n /= 36;
    }
    let len = out.len() - end;
    (end, len)
}

#[kernel]
#[inline(never)]
pub unsafe fn iterate_seeds(start: u64, total: u64, out_checksums: *mut f64) {
    let tid = thread::index_1d() as u64;
    let tid_usize = tid as usize;
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
        let (start_off, len) = encode_seed(idx, &mut seed_buf);
        // Use raw ASCII bytes directly
        let rng = Random::new(&seed_buf[start_off..start_off + len]);
        // Accumulate hashed_seed to avoid being optimized out
       // sum += rng.get_node((i as usize) % IDS_LEN);
        sum += rng.hashed_seed;
        i += stride;
    }

    let out_ptr = unsafe { out_checksums.add(tid_usize) };
    unsafe { *out_ptr = sum; }
}