use std::env;
use std::time::Instant;

use common::random::Random;

fn main() {
    // Parse iterations from first CLI arg, default to 1_000_000
    let iterations: u64 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);

    // Fixed 8-byte ASCII seed buffer; we'll tweak last byte each iteration to avoid being too cache-friendly
    let mut base_seed: [u8; 8] = *b"ABCDEFG0";

    let start = Instant::now();
    let mut acc = 0.0f64;

    for i in 0..iterations {
        // Vary the last character within [0-9A-Z]; keep within our charset
        let digit = (i % 36) as u8;
        base_seed[7] = match digit {
            0..=9 => b'0' + digit,
            _ => b'A' + (digit - 10),
        };

        let mut rng = Random::new(&base_seed);
        // Call get_node three times on small ids (valid for current IDS_LEN >= 3)
        acc += rng.get_node(0);
        acc += rng.get_node(1);
        acc += rng.get_node(2);
    }

    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let iters_per_sec = iterations as f64 / secs;
    println!(
        "bench_random: iterations={} time={:.3}s iters/s={:.0} sum={}",
        iterations, secs, iters_per_sec, acc
    );
}
