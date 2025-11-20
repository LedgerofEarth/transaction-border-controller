use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tbc_core::codec_tx::encode_message;
use tbc_core::protocol::{TGPMessage, QueryMessage};
use tbc_core::tgp::types::ZkProfile;

fn make_msg(i: usize) -> TGPMessage {
    let q = QueryMessage {
        id: format!("q-{i}"),
        from: "buyer://alice".into(),
        to: "seller://bob".into(),
        asset: "USDC".into(),
        amount: 1000,
        escrow_from_402: false,
        escrow_contract_from_402: None,
        zk_profile: ZkProfile::Optional,
    };
    TGPMessage::Query(q)
}

fn bench_codec_scaled(c: &mut Criterion) {
    let msg = make_msg(0);

    let mut group = c.benchmark_group("codec_scaled_iter");

    // multipliers applied to Criterion's iteration loop
    for &(name, mult) in &[
        ("1x", 1usize),
        ("5x", 5),
        ("25x", 25),
        ("125x", 125),
        ("625x", 625),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                for _ in 0..mult {
                    encode_message(black_box(&msg)).unwrap();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_codec_scaled);
criterion_main!(benches);