use criterion::{criterion_group, criterion_main, Criterion, Throughput, BatchSize};
use common::random::Random;

fn bench_random_new_and_get_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_new_get_node");
    group.throughput(Throughput::Elements(1));

    let mut seed: [u8; 8] = *b"ABCDEFG0";

    group.bench_function("new+get_node[x3]", |b| {
        b.iter_batched(
            || {
                // Update last byte to avoid being overly cache-friendly
                seed[7] = if seed[7] == b'Z' { b'0' } else { seed[7] + 1 };
                seed
            },
            |seed_bytes| {
                let mut rng = Random::new(&seed_bytes);
                let _ = rng.get_node(0);
                let _ = rng.get_node(1);
                let _ = rng.get_node(2);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_random_new_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_new_only");
    group.throughput(Throughput::Elements(1));

    let mut seed: [u8; 8] = *b"ABCDEFG0";

    group.bench_function("new_only", |b| {
        b.iter(|| {
            seed[7] = if seed[7] == b'Z' { b'0' } else { seed[7] + 1 };
            let _rng = Random::new(&seed);
        });
    });

    group.finish();
}

fn bench_get_node_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_get_node_only");
    group.throughput(Throughput::Elements(1));

    let seed: [u8; 8] = *b"ABCDEFG0";
    // Pre-construct and reuse Random across iterations to measure just get_node
    let mut rng = Random::new(&seed);

    group.bench_function("get_node[x3]_reuse", |b| {
        b.iter(|| {
            let _ = rng.get_node(0);
            let _ = rng.get_node(1);
            let _ = rng.get_node(2);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_random_new_and_get_nodes, bench_random_new_only, bench_get_node_only);
criterion_main!(benches);
