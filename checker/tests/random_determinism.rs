use common::random::Random;
use common::random::ItemChoice;
use common::random::RESAMPLE_IDS_LEN;
use std::ptr;

fn approx_eq(a: f64, b: f64, eps: f64) -> bool { (a - b).abs() <= eps }

fn index_of<T>(slice: &[T], target: &T) -> usize {
    slice
        .iter()
        .position(|p| ptr::eq(p, target))
        .expect("chosen reference must belong to items slice")
}

#[test]
fn rand_choice_resamples_when_first_pick_disallowed() {
    let len = 64;
    let seeds: [&[u8]; 2] = [b"ABCDEFG0", b"HELLO1"];
    let ids = [0usize, RESAMPLE_IDS_LEN / 2, RESAMPLE_IDS_LEN - 1];

    for seed in seeds {
        for &id in &ids {
            // Compute the index the initial draw would select
            let initial_idx = {
                let mut r = Random::new(seed);
                r.rand_int(id, 0, (len - 1) as i32) as usize
            };

            // Build items so that the initial index is disallowed, others allowed
            let mut items: Vec<DummyItem> = (0..len).map(|_| DummyItem { allow: true }).collect();
            items[initial_idx].allow = false;

            // rand_choice must not return the initially drawn disallowed item
            let mut rng = Random::new(seed);
            let chosen_ref = rng.rand_choice(id, &items);
            let chosen_idx = index_of(&items, chosen_ref);
            assert_ne!(
                chosen_idx, initial_idx,
                "rand_choice should resample when first pick is disallowed (seed {:?}, id {})",
                seed, id
            );

            // Determinism: repeat and expect the same chosen index
            let mut rng2 = Random::new(seed);
            let chosen_ref2 = rng2.rand_choice(id, &items);
            let chosen_idx2 = index_of(&items, chosen_ref2);
            assert_eq!(chosen_idx, chosen_idx2);
        }
    }
}

#[test]
#[should_panic]
fn rand_choice_panics_when_no_usable_item() {
    // All items disallowed -> should exhaust resamples and panic
    let items: Vec<DummyItem> = (0..32).map(|_| DummyItem { allow: false }).collect();
    let seed: [u8; 8] = *b"ABCDEFG0";
    let mut rng = Random::new(&seed);
    // Any id works; the function must panic after trying all resamples
    let _ = rng.rand_choice(0, &items);
}

#[derive(Clone, Copy, Debug)]
struct DummyItem {
    allow: bool,
}

impl ItemChoice for DummyItem {
    fn retry(&self) -> bool { !self.allow }
    fn locked(&self) -> bool { false }
}

#[test]
fn rand_choice_is_deterministic_for_fixed_seed() {
    // Build a list where all items are allowed; selection depends solely on RNG.
    let items: Vec<DummyItem> = (0..64).map(|_| DummyItem { allow: true }).collect();

    // Seed 1
    let seed1: [u8; 8] = *b"ABCDEFG0";
    let mut rng_a = Random::new(&seed1);
    let chosen_a = rng_a.rand_choice(0, &items) as *const DummyItem as usize;
    // Re-run with the same seed and id; must choose the same element deterministically.
    let mut rng_b = Random::new(&seed1);
    let chosen_b = rng_b.rand_choice(0, &items) as *const DummyItem as usize;
    assert_eq!(chosen_a, chosen_b, "rand_choice should be deterministic for a fixed seed and id");

    // Also verify determinism across a second id value
    let mut rng_c = Random::new(&seed1);
    let chosen_c = rng_c.rand_choice(RESAMPLE_IDS_LEN - 1, &items) as *const DummyItem as usize;
    let mut rng_d = Random::new(&seed1);
    let chosen_d = rng_d.rand_choice(RESAMPLE_IDS_LEN - 1, &items) as *const DummyItem as usize;
    assert_eq!(chosen_c, chosen_d);
}

#[test]
fn resample_ids_progress_prng_multiple_passes() {
    // For two seeds, iterate every resample id and ensure two successive calls progress output.
    let seeds: [&[u8]; 2] = [b"ABCDEFG0", b"HELLO1"]; // variable length seeds are supported
    for seed in seeds {
        // Do multiple passes to ensure stability
        for _pass in 0..3 {
            for id in 0..RESAMPLE_IDS_LEN {
                // First sample at this id
                let mut rng1 = Random::new(seed);
                let a1 = rng1.random(id);
                // Second sample should progress because get_node(id) advances and reseeds LuaRandom.
                let mut rng2 = Random::new(seed);
                let _ = rng2.random(id); // advance once
                let a2 = rng2.random(id); // second call
                assert!(
                    (a2 - a1).abs() > 1e-18,
                    "PRNG output did not progress for seed {:?} id {}: a1={} a2={}",
                    seed,
                    id,
                    a1,
                    a2
                );

                // Also verify node progression itself is not constant
                let mut rngn = Random::new(seed);
                let n1 = rngn.get_node(id);
                let n2 = rngn.get_node(id);
                assert!((n2 - n1).abs() > 0.0, "get_node did not progress for id {}", id);
            }
        }
    }
}

#[test]
fn random_determinism_fixed_seeds() {
    // Seed 1: "ABCDEFG0"
    let seed1: [u8; 8] = *b"ABCDEFG0";
    let mut rng1 = Random::new(&seed1);
    let n10 = rng1.get_node(0);
    let n11 = rng1.get_node(0);
    let r10 = {
        let mut rng = Random::new(&seed1);
        rng.random(0)
    };
    let r11 = {
        let mut rng = Random::new(&seed1);
        let _ = rng.random(0);
        rng.random(0)
    };
    let ri1 = {
        let mut rng = Random::new(&seed1);
        rng.rand_int(0, 1, 10)
    };

    assert!(n10.is_finite() && n11.is_finite() && r10.is_finite() && r11.is_finite());
    assert!(approx_eq(n10, 0.3974688476399526, 1e-15));
    assert!(approx_eq(n11, 0.1700901308279026, 1e-15));
    assert!(approx_eq(r10, 0.6802250579770714, 1e-15));
    assert!(approx_eq(r11, 0.7276895864589519, 1e-15));
    assert_eq!(ri1, 7);

    // Seed 2: "HELLO1"
    let seed2: [u8; 6] = *b"HELLO1";
    let mut rng2 = Random::new(&seed2);
    let n20 = rng2.get_node(0);
    let n21 = rng2.get_node(0);
    let r20 = {
        let mut rng = Random::new(&seed2);
        rng.random(0)
    };
    let r21 = {
        let mut rng = Random::new(&seed2);
        let _ = rng.random(0);
        rng.random(0)
    };
    let ri2 = {
        let mut rng = Random::new(&seed2);
        rng.rand_int(0, 1, 10)
    };

    assert!(n20.is_finite() && n21.is_finite() && r20.is_finite() && r21.is_finite());
    assert!(approx_eq(n20, 0.4166009355118860, 1e-15));
    assert!(approx_eq(n21, 0.1865191996113360, 1e-15));
    assert!(approx_eq(r20, 0.8118522286903114, 1e-15));
    assert!(approx_eq(r21, 0.8419855273013768, 1e-15));
    assert_eq!(ri2, 9);
}
