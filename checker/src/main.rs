use cust::prelude::*;
use rayon::prelude::*;
use common::random::Random;
use std::error::Error;
use std::time::Instant;

//

static PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/gpu_driver.ptx"));

const CHARSET: [u8; 36] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[inline(always)]
fn encode_seed_bytes(mut n: u64, out: &mut [u8; 8]) -> (usize, usize) {
    // Matches gpu_driver::encode_seed: base-36, no leading symbols
    let mut end = out.len();
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

/// CPU-parallel mirror of gpu_driver::iterate_seeds using rayon.
/// Returns the sum of per-item values (mirrors current GPU: rng.hashed_seed per seed).
pub fn iterate_seeds_cpu(start: u64, total: u64) -> f64 {
    (0..total)
        .into_par_iter()
        .map(|i| {
            let idx = start + i;
            let mut seed_buf = [0u8; 8];
            let (off, len) = encode_seed_bytes(idx, &mut seed_buf);
            let rng = Random::new(&seed_buf[off..off + len]);
            // Mirror current GPU behavior (summing hashed_seed). If the GPU switches back to get_node,
            // change this to: rng.get_node((i as usize) % IDS_LEN)
            rng.hashed_seed
        })
        .sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    // initialize CUDA, this will pick the first available device and will
    // make a CUDA context from it.
    // We don't need the context for anything but it must be kept alive.
    let _ctx = cust::quick_init()?;

    // Make the CUDA module, modules just house the GPU code for the kernels we created.
    // they can be made from PTX code, cubins, or fatbins.
    let module = Module::from_ptx(PTX, &[])?;

    // make a CUDA stream to issue calls to. You can think of this as an OS thread but for dispatching
    // GPU calls.
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

    // allocate the GPU memory needed to house our numbers and c
    // ---------------------------------------------------------------------
    // Launch the iterate_seeds kernel over the first 100,000,000 seeds
    // ---------------------------------------------------------------------
    let iterate = module.get_function("iterate_seeds")?;
    // Choose a reasonable block size suggested by CUDA; 0 dynamic shared mem
    let (_, iter_block_size) = iterate.suggested_launch_configuration(0, 0.into())?;
    let iter_grid_size: u32 = 4096; // fixed grid to bound output size
    let threads_per_grid = (iter_grid_size as usize) * (iter_block_size as usize);

    // Output buffer for per-thread checksums
    let mut checksums = vec![0.0f64; threads_per_grid];
    let checksums_buf = checksums.as_slice().as_dbuf()?;

    let start: u64 = 0;
    let total: u64 = 10_000_000_000;
    //let total: u64 = 36_u64.pow(8);

    unsafe {
        launch!(
            iterate<<<iter_grid_size, iter_block_size, 0, stream>>>(
                start,
                total,
                checksums_buf.as_device_ptr(),
            )
        )?;
    }

    stream.synchronize()?;
    checksums_buf.copy_to(&mut checksums)?;
    let checksum_sum: f64 = checksums.iter().copied().sum();
    println!(
        "iterate_seeds: grid={} block={} threads={} total={} checksum_sum={} time={:?}",
        iter_grid_size,
        iter_block_size,
        threads_per_grid,
        total,
        checksum_sum,
        start_time.elapsed()
    );

    Ok(())
}