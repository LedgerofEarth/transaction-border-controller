use criterion::{criterion_group, criterion_main, Criterion, black_box};

use tbc_core::tgp::validation::{
    validate_address,
    validate_correlation_id,
    validate_positive_amount,
    validate_non_empty,
};

fn bench_validate_address(c: &mut Criterion) {
    c.bench_function("tgp_validate_address", |b| {
        b.iter(|| {
            validate_address(
                black_box("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"),
                black_box("sender"),
            )
            .unwrap();
        })
    });
}

fn bench_validate_correlation(c: &mut Criterion) {
    c.bench_function("tgp_validate_correlation_id", |b| {
        b.iter(|| {
            validate_correlation_id(
                black_box("q-123abc"),
                black_box(Some("QUERY")),
            )
            .unwrap();
        })
    });
}

fn bench_validate_positive_amount(c: &mut Criterion) {
    c.bench_function("tgp_validate_positive_amount", |b| {
        b.iter(|| {
            validate_positive_amount(
                black_box(1000u64),
                black_box("amount"),
            )
            .unwrap();
        })
    });
}

fn bench_validate_non_empty(c: &mut Criterion) {
    c.bench_function("tgp_validate_non_empty", |b| {
        b.iter(|| {
            validate_non_empty(
                black_box("hello"),
                black_box("field"),
            )
            .unwrap();
        })
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("tgp_full_validation_pipeline", |b| {
        b.iter(|| {
            validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "sender").unwrap();
            validate_correlation_id("q-xyz777", Some("QUERY")).unwrap();
            validate_positive_amount(42, "amount").unwrap();
            validate_non_empty("payment-profile-abc", "profile").unwrap();
        })
    });
}

criterion_group!(
    tgp_validation_benches,
    bench_validate_address,
    bench_validate_correlation,
    bench_validate_positive_amount,
    bench_validate_non_empty,
    bench_full_pipeline,
);

criterion_main!(tgp_validation_benches);