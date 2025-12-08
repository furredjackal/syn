//! Benchmarks for syn_core hot paths.
//!
//! Run with: `cargo bench -p syn_core`
//!
//! Results are written to `target/criterion/`.

#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use rustc_hash::FxHashMap;

use syn_core::{
    DeterministicRng, WorldSeed, NpcId, Stats, Relationship,
    WorldState, AbstractNpc, Traits, AttachmentStyle,
};

/// Benchmark HashMap vs FxHashMap for relationship lookups.
fn bench_hashmap_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_lookup");
    
    // Populate maps with typical relationship counts
    for count in [100, 1000, 10_000] {
        let mut std_map: HashMap<(u64, u64), Relationship> = HashMap::new();
        let mut fx_map: FxHashMap<(u64, u64), Relationship> = FxHashMap::default();
        
        for i in 0..count {
            let key = (i / 100, i % 100);
            let rel = Relationship::default();
            std_map.insert(key, rel);
            fx_map.insert(key, rel);
        }
        
        group.bench_with_input(
            BenchmarkId::new("std_hashmap", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let key = black_box((50, 50));
                    std_map.get(&key)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("fx_hashmap", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let key = black_box((50, 50));
                    fx_map.get(&key)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark RNG performance (critical for determinism).
fn bench_rng(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng");
    
    group.bench_function("gen_f32", |b| {
        let mut rng = DeterministicRng::new(12345);
        b.iter(|| {
            black_box(rng.gen_f32())
        })
    });
    
    group.bench_function("gen_range_i32", |b| {
        let mut rng = DeterministicRng::new(12345);
        b.iter(|| {
            black_box(rng.gen_range_i32(0, 100))
        })
    });
    
    group.bench_function("gen_bool", |b| {
        let mut rng = DeterministicRng::new(12345);
        b.iter(|| {
            black_box(rng.gen_bool(0.5))
        })
    });
    
    group.bench_function("with_domain", |b| {
        b.iter(|| {
            let rng = DeterministicRng::with_domain(
                black_box(12345),
                black_box(1000),
                black_box("test_domain"),
            );
            black_box(rng)
        })
    });
    
    group.finish();
}

/// Benchmark WorldState creation and basic operations.
fn bench_world_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("world_state");
    
    group.bench_function("new", |b| {
        b.iter(|| {
            let seed = WorldSeed(black_box(12345));
            let player_id = NpcId(black_box(1));
            black_box(WorldState::new(seed, player_id))
        })
    });
    
    group.bench_function("get_relationship", |b| {
        let mut world = WorldState::new(WorldSeed(12345), NpcId(1));
        // Pre-populate some relationships
        for i in 0..100 {
            world.relationships.insert(
                (NpcId(1), NpcId(i + 2)),
                Relationship::default(),
            );
        }
        
        b.iter(|| {
            black_box(world.get_relationship(NpcId(1), NpcId(50)))
        })
    });
    
    group.finish();
}

/// Benchmark Stats operations.
fn bench_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats");
    
    group.bench_function("clamp", |b| {
        b.iter(|| {
            let mut stats = Stats {
                health: 150.0,
                intelligence: -10.0,
                charisma: 200.0,
                wealth: 50.0,
                mood: 15.0,
                appearance: 80.0,
                reputation: 150.0,
                wisdom: 60.0,
                curiosity: Some(40.0),
                energy: Some(30.0),
                libido: Some(20.0),
            };
            stats.clamp();
            black_box(stats)
        })
    });
    
    group.bench_function("mood_band", |b| {
        let stats = Stats::default();
        b.iter(|| {
            black_box(stats.mood_band())
        })
    });
    
    group.finish();
}

/// Benchmark NPC lookups with varying population sizes.
fn bench_npc_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_lookup");
    
    for pop_size in [100, 1000, 10_000] {
        let mut npcs: FxHashMap<NpcId, AbstractNpc> = FxHashMap::default();
        
        for i in 0..pop_size {
            npcs.insert(NpcId(i), AbstractNpc {
                id: NpcId(i),
                age: 25,
                job: "Worker".to_string(),
                district: "Downtown".to_string(),
                household_id: i / 4,
                traits: Traits::default(),
                seed: i,
                attachment_style: AttachmentStyle::default(),
            });
        }
        
        group.bench_with_input(
            BenchmarkId::new("get", pop_size),
            &pop_size,
            |b, _| {
                b.iter(|| {
                    let id = NpcId(black_box(pop_size / 2));
                    npcs.get(&id)
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark string interning performance.
fn bench_intern(c: &mut Criterion) {
    use syn_core::intern::{intern, resolve, InternedStr};
    
    let mut group = c.benchmark_group("intern");
    
    // Pre-intern some strings
    for i in 0..1000 {
        intern(&format!("pre_interned_{}", i));
    }
    
    group.bench_function("intern_existing", |b| {
        b.iter(|| {
            black_box(intern("pre_interned_500"))
        })
    });
    
    group.bench_function("intern_new", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            black_box(intern(&format!("new_string_{}", counter)))
        })
    });
    
    group.bench_function("resolve", |b| {
        let id = intern("resolve_test");
        b.iter(|| {
            black_box(resolve(id))
        })
    });
    
    group.bench_function("compare_interned", |b| {
        let id1 = intern("compare_a");
        let id2 = intern("compare_a");
        b.iter(|| {
            black_box(id1 == id2)
        })
    });
    
    group.bench_function("compare_strings", |b| {
        let s1 = "compare_this_longer_string";
        let s2 = "compare_this_longer_string";
        b.iter(|| {
            black_box(s1 == s2)
        })
    });
    
    group.finish();
}

/// Benchmark WorldFlags bitfield vs dynamic lookup.
fn bench_world_flags(c: &mut Criterion) {
    use syn_core::world_flags::{WorldFlags, KnownFlag};
    
    let mut group = c.benchmark_group("world_flags");
    
    let mut flags = WorldFlags::new();
    flags.set(KnownFlag::Married);
    flags.set(KnownFlag::GotFirstJob);
    flags.set(KnownFlag::GraduatedCollege);
    flags.set_dynamic("custom_flag_1");
    flags.set_dynamic("custom_flag_2");
    
    group.bench_function("get_known", |b| {
        b.iter(|| {
            black_box(flags.has(KnownFlag::Married))
        })
    });
    
    group.bench_function("get_dynamic", |b| {
        b.iter(|| {
            black_box(flags.has_dynamic("custom_flag_1"))
        })
    });
    
    group.bench_function("has_any_known", |b| {
        b.iter(|| {
            black_box(flags.has_any("married"))
        })
    });
    
    group.bench_function("has_any_dynamic", |b| {
        b.iter(|| {
            black_box(flags.has_any("custom_flag_1"))
        })
    });
    
    group.bench_function("set_known", |b| {
        b.iter(|| {
            let mut flags = WorldFlags::new();
            flags.set(KnownFlag::FirstKiss);
            black_box(flags)
        })
    });
    
    group.finish();
}

/// Benchmark rkyv snapshot serialization.
fn bench_snapshot(c: &mut Criterion) {
    use syn_core::snapshot::{save_snapshot, load_snapshot, deserialize_snapshot};
    
    let mut group = c.benchmark_group("snapshot");
    
    // Create a world with some data
    let mut world = WorldState::new(WorldSeed(12345), NpcId(1));
    world.player_stats.health = 75.0;
    world.player_age_years = 25;
    world.world_flags.set_any("tutorial_complete");
    
    // Add some NPCs
    for i in 0..100 {
        world.npcs.insert(NpcId(i + 2), AbstractNpc {
            id: NpcId(i + 2),
            age: 20 + (i % 60) as u32,
            job: "Worker".to_string(),
            district: "Downtown".to_string(),
            household_id: i / 4,
            traits: Traits::default(),
            seed: i,
            attachment_style: AttachmentStyle::default(),
        });
    }
    
    // Add some relationships
    for i in 0..50 {
        world.relationships.insert(
            (NpcId(1), NpcId(i + 2)),
            Relationship::default(),
        );
    }
    
    let bytes = save_snapshot(&world).expect("serialize failed");
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            black_box(save_snapshot(&world).unwrap())
        })
    });
    
    group.bench_function("zero_copy_load", |b| {
        b.iter(|| {
            black_box(load_snapshot(&bytes).unwrap())
        })
    });
    
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            black_box(deserialize_snapshot(&bytes).unwrap())
        })
    });
    
    // Compare with serde_json
    let json_bytes = serde_json::to_vec(&world).expect("json failed");
    
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_vec(&world).unwrap())
        })
    });
    
    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            black_box(serde_json::from_slice::<WorldState>(&json_bytes).unwrap())
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_hashmap_lookup,
    bench_rng,
    bench_world_state,
    bench_stats,
    bench_npc_lookup,
    bench_intern,
    bench_world_flags,
    bench_snapshot,
);

criterion_main!(benches);
