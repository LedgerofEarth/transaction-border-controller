use criterion::{criterion_group, criterion_main, Criterion, black_box};
use tbc_core::codec_tx::{InMemoryReplayCache, ReplayProtector};

fn bench_replay_insert(c: &mut Criterion) {
    let cache = InMemoryReplayCache::new(8192);

    c.bench_function("replay_insert_10k", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                let id = format!("msg-{}", i);
                black_box(cache.check_or_insert(&id));
            }
        })
    });
}

fn bench_replay_replay_hits(c: &mut Criterion) {
    let cache = InMemoryReplayCache::new(8192);

    // Pre-fill cache
    for i in 0..8192 {
        cache.check_or_insert(&format!("msg-{}", i));
    }

    c.bench_function("replay_hit_100k", |b| {
        b.iter(|| {
            for i in 0..100_000 {
                black_box(cache.check_or_insert("msg-100")); // always replay
            }
        })
    });
}

fn bench_replay_eviction(c: &mut Criterion) {
    let cache = InMemoryReplayCache::new(1024);

    c.bench_function("replay_eviction_10k", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                black_box(
                    cache.check_or_insert(&format!("evict-{}", i))
                );
            }
        })
    });
}

criterion_group!(benches, bench_replay_insert, bench_replay_replay_hits, bench_replay_eviction);
criterion_main!(benches);