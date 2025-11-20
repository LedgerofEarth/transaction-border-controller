use criterion::{criterion_group, criterion_main, Criterion, black_box};

use tbc_core::tgp::messages::TGPMessage;
use tbc_core::tgp::validation::{
    validate_address,
    validate_non_empty,
    validate_positive_amount,
    validate_correlation_id,
};
use tbc_core::codec_tx::{encode_message, check_or_insert};

/// Load a sample fully-valid QUERY for benchmarking.
/// You will place this file at:
/// crates/tbc-core/benches/sample_query.json
const SAMPLE_QUERY: &str = include_str!("sample_query.json");

/// End-to-end TGP pipeline:
/// 1. Deserialize JSON → TGPMessage
/// 2. Validate fields
/// 3. Replay-protect
/// 4. Encode with codec_tx
fn process_tgp_message(json: &str) -> Result<(), String> {
    // Step 1: Parse into your real enum
    let msg: TGPMessage = serde_json::from_str(json)
        .map_err(|e| format!("deserialize error: {e}"))?;

    // Step 2: Validate fields per message type
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

    // Step 3: Replay protection (correlation ID)
    let cid = msg.correlation_id();
    check_or_insert(&cid)
        .map_err(|e| format!("replay error: {e:?}"))?;

    // Step 4: Encode
    encode_message(&msg)
        .map_err(|e| format!("encode error: {e}"))?;

    Ok(())
}

/// Criterion benchmark — full pipeline performance
fn bench_tgp_full_flow(c: &mut Criterion) {
    c.bench_function("tgp_full_message_flow", |b| {
        b.iter(|| {
            process_tgp_message(black_box(SAMPLE_QUERY))
                .expect("pipeline must succeed");
        })
    });
}

criterion_group!(benches, bench_tgp_full_flow);
criterion_main!(benches);