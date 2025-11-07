use glob::glob;
use serde::Deserialize;
use std::fs;
use common::game::GameState;
use common::items::RandomSource;
use common::names::{boss_name, voucher_name, tarot_name, planet_name, tag_name};

#[derive(Deserialize)]
struct AnalyzeState {
    seed: String,
}

#[derive(Deserialize)]
struct AnteInfo {
    ante: i32,
    boss: String,
    voucher: String,
}

#[derive(Deserialize)]
struct ImmolateResults {
    antes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct OptionsBlock {
    unlocks: Vec<String>,
}

#[derive(Deserialize)]
struct FixtureRoot {
    #[serde(rename = "analyzeState")]
    analyze_state: AnalyzeState,
    #[serde(rename = "immolateResults")]
    immolate_results: ImmolateResults,
    options: OptionsBlock,
}

#[test]
fn ante1_boss_and_voucher_match() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pattern = format!("{}/../Blueprint/___tests___/seedJson/*.json", manifest_dir).replace('\\', "/");
    let mut tested = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for entry in glob(&pattern).expect("glob pattern") {
        let path = entry.expect("path");
        let text = fs::read_to_string(&path).expect("read json");
        let parsed: FixtureRoot = serde_json::from_str(&text).expect("parse json");

        // Find ante "1" block
        let ante1 = parsed
            .immolate_results
            .antes
            .get("1")
            .expect("ante 1 present");
        let ante1: AnteInfo = serde_json::from_value(ante1.clone()).expect("ante record");

        let mut game = GameState::new(&parsed.analyze_state.seed, 1);
        // Mirror Blueprint: first lock level-two vouchers; then unlock selected options (lockOptions)
        game.lock_level_two_vouchers();
        game.apply_unlocks(parsed.options.unlocks.clone().into_iter());
        let voucher = game.next_voucher_from_at_ante(RandomSource::Shop, 1);
        let selected = &parsed.options.unlocks;
        if selected.iter().any(|s| s.as_str() == voucher_name(&voucher)) {
            game.activate_voucher(voucher);
        }
        let boss = game.next_boss_from_at_ante(RandomSource::Shop, 1);

        // Also compare first Tarot and first Planet in the queue if present
        let queue = parsed
            .immolate_results
            .antes
            .get("1")
            .unwrap()
            .get("queue")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let first_tarot_json = queue.iter().find(|e| e.get("type").and_then(|t| t.as_str()) == Some("Tarot"))
            .and_then(|e| e.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()));
        let first_planet_json = queue.iter().find(|e| e.get("type").and_then(|t| t.as_str()) == Some("Planet"))
            .and_then(|e| e.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()));
        let first_joker_json = queue.iter().find(|e| e.get("type").and_then(|t| t.as_str()) == Some("Joker"))
            .and_then(|e| e.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()));

        if let Some(expected_tarot) = first_tarot_json {
            // In Blueprint queues, the displayed Tarot typically comes from Arcana pack
            let ours_tarot = game.next_tarot_from_at_ante(RandomSource::Arcana, 1);
            let ours_tarot_name = tarot_name(&ours_tarot).to_string();
            if ours_tarot_name != expected_tarot {
                errors.push(format!(
                    "seed {}: tarot mismatch: ours='{}' expected='{}'",
                    parsed.analyze_state.seed, ours_tarot_name, expected_tarot
                ));
            }

        }

        if let Some(expected_planet) = first_planet_json {
            // In Blueprint queues, the displayed Planet typically comes from Celestial pack
            let ours_planet = game.next_planet_from_at_ante(RandomSource::Celestial, 1);
            let ours_planet_name = planet_name(&ours_planet).to_string();
            if ours_planet_name != expected_planet {
                errors.push(format!(
                    "seed {}: planet mismatch: ours='{}' expected='{}'",
                    parsed.analyze_state.seed, ours_planet_name, expected_planet
                ));
            }
        }

        // Joker diagnostic (not asserting yet as rarity path not mirrored)
        if let Some(expected_joker) = first_joker_json {
            let _ours_joker = game.next_joker_from_at_ante(RandomSource::Shop, 1);
            // We could log debug, but keep noise minimal for now.
            let _ = expected_joker;
        }

        // Validate tags independently of vouchers/boss: tags should not affect or be affected by other streams
        // Iterate antes in ascending order, using a fresh GameState per ante and drawing only tags
        let mut ante_keys: Vec<i32> = parsed
            .immolate_results
            .antes
            .keys()
            .filter_map(|k| k.parse::<i32>().ok())
            .collect();
        ante_keys.sort_unstable();
        for ante_num in ante_keys {
            let ante_val = parsed.immolate_results.antes.get(&ante_num.to_string()).unwrap();
            if let Some(tags_arr) = ante_val.get("tags").and_then(|t| t.as_array()) {
                // Fresh game state for tag-only generation
                let mut tag_game = GameState::new(&parsed.analyze_state.seed, 1);
                // Draw only tags for this ante; tags are independent (no locking/resample)
                let mut ours: Vec<String> = Vec::new();
                for _draw in 0..tags_arr.len() {
                    // debug line with node id, mixed, rand, idx, and choice name
                    let (id, mixed, rand, idx, name) = tag_game.debug_tag_once(ante_num);
                    println!(
                        "RS_TAG ante={} id={} mixed={:.13} rand={:.13} idx={} name={}",
                        ante_num, id, mixed, rand, idx, name
                    );
                    // Use this debug choice as authoritative to avoid a second advancement
                    ours.push(name.to_string());
                }
                for (i, v) in tags_arr.iter().enumerate() {
                    if let Some(exp) = v.as_str() {
                        if ours.get(i).map(|s| s.as_str()) != Some(exp) {
                            errors.push(format!(
                                "seed {} ante {}: tag #{} mismatch: ours='{}' expected='{}'",
                                parsed.analyze_state.seed, ante_num, i+1, ours.get(i).cloned().unwrap_or_default(), exp
                            ));
                        }
                    }
                }
            }
        }

        let boss_str = boss_name(&boss).to_string();
        let voucher_str = voucher_name(&voucher).to_string();

        if boss_str != ante1.boss {
            errors.push(format!("seed {}: boss mismatch: ours='{}' expected='{}'", parsed.analyze_state.seed, boss_str, ante1.boss));
        }
        if voucher_str != ante1.voucher {
            errors.push(format!("seed {}: voucher mismatch: ours='{}' expected='{}'", parsed.analyze_state.seed, voucher_str, ante1.voucher));
        }

        tested += 1;
        if tested >= 3 {
            // Limit runtime for CI; increase as needed
            break;
        }
    }

    assert!(tested > 0, "no fixtures tested");
    if !errors.is_empty() {
        panic!("\n{}", errors.join("\n"));
    }
}
