use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rayon::prelude::*;

use tbc_core::tgp::messages::TGPMessage;
use tbc_core::tgp::validation::{
    validate_address,
    validate_non_empty,
    validate_positive_amount,
    validate_correlation_id,
};
use tbc_core::codec_tx::{encode_message, check_or_insert};

const SAMPLE_QUERY: &str = include_str!("sample_query.json");

/// Same processor as the single-threaded flow test
fn process_tgp_message(json: &str) -> Result<(), String> {
    let msg: TGPMessage = serde_json::from_str(json)
        .map_err(|e| format!("deserialize error: {e}"))?;

    match &msg {
        TGPMessage::Query(q) => {
            validate_correlation_id(&q.id, Some("QUERY"))?;
            validate_address(&q.from, "from")?;
            validate_address(&q.to, "to")?;
            validate_non_empty(&q.asset, "asset")?;
            validate_positive_amount(q.amount, "amount")?;
        }
        TGPMessage::Offer(o) => {
            validate_correlation_id(&o.id, Some("OFFER"))?;
            validate_address(&o.from, "from")?;
            validate_address(&o.to, "to")?;
        }
        TGPMessage::Settle(s) => {
            validate_correlation_id(&s.id, Some("SETTLE"))?;
            if let Some(tx) = &s.tx_hash {
                validate_transaction_hash(tx, "tx_hash")?;
            }
        }
        TGPMessage::Error(e) => {
            validate_correlation_id(&e.id, Some("ERROR"))?;
            validate_non_empty(&e.code, "code")?;
            validate_non_empty(&e.message, "message")?;
        }
    }

    let cid = msg.correlation_id();
    check_or_insert(&cid)
        .map_err(|e| format!("replay error: {e:?}"))?;

    encode_message(&msg)
        .map_err(|e| format!("encode error: {e}"))?;

    Ok(())
}

/// Parallel benchmark: runs many messages in parallel using Rayon.
/// This demonstrates horizontal scalability of the control plane.
fn bench_tgp_parallel_flow(c: &mut Criterion) {
    // Generate a batch of messages for parallel testing
    // (Replay cache sees different correlation IDs)
    let payloads: Vec<String> = (0..10_000)
        .map(|i| {
            SAMPLE_QUERY
                .replace("q-demo-001", &format!("q-par-{}", i))
        })
        .collect();

    c.bench_function("tgp_parallel_message_flow", |b| {
        b.iter(|| {
            payloads.par_iter().for_each(|json| {
                process_tgp_message(black_box(json))
                    .expect("parallel pipeline must succeed");
            });
        });
    });
}

criterion_group!(benches, bench_tgp_parallel_flow);
criterion_main!(benches);