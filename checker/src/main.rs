use cust::prelude::*;
use std::error::Error;
use std::time::Instant;

/// How many numbers to generate and add together.
const NUMBERS_LEN: usize = 100_000;

static PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/gpu_driver.ptx"));

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
    let total: u64 = 1_000_000_000; // first 100 million seeds as requested

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