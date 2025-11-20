use rayon::prelude::*;
use std::time::{Duration, Instant};

use tbc_core::tgp::messages::TGPMessage;
use tbc_core::tgp::validation::{
    validate_address,
    validate_correlation_id,
    validate_non_empty,
    validate_positive_amount,
};
use tbc_core::codec_tx::{check_or_insert, encode_message};

const SAMPLE_QUERY: &str = include_str!("sample_query.json");

/// Full TGP pipeline used in benchmarks
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
    check_or_insert(&cid).map_err(|e| format!("replay error: {e:?}"))?;

    encode_message(&msg).map_err(|e| format!("encode error: {e}"))?;

    Ok(())
}

/// Compute percentile of a dataset.
fn percentile(values: &mut [u128], pct: f64) -> u128 {
    values.sort_unstable();
    let idx = ((values.len() as f64) * pct).ceil().clamp(1.0, values.len() as f64) as usize - 1;
    values[idx]
}

fn main() {
    let batch_size = 50_000;
    let thread_count = rayon::current_num_threads();

    println!("===================================================");
    println!("🔬 TGP Throughput Summary Harness");
    println!("---------------------------------------------------");
    println!("Batch size:               {batch_size}");
    println!("Rayon threads:           {thread_count}");
    println!("===================================================");

    // Prepare dataset with unique correlation IDs
    let payloads: Vec<String> = (0..batch_size)
        .map(|i| SAMPLE_QUERY.replace("q-demo-001", &format!("q-load-{i}")))
        .collect();

    // Run load test
    let mut latencies: Vec<u128> = Vec::with_capacity(batch_size);
    let start = Instant::now();

    payloads.par_iter().for_each(|json| {
        let t0 = Instant::now();
        process_tgp_message(json).unwrap();
        let dt = t0.elapsed().as_nanos();
        latencies.push(dt);
    });

    let elapsed = start.elapsed().as_secs_f64();

    // Compute metrics
    let total_ops = batch_size as f64;
    let tps = total_ops / elapsed;
    let per_core = tps / thread_count as f64;

    let mut lat = latencies.clone();
    let p50 = percentile(&mut lat, 0.50) as f64 / 1_000_000.0;
    let p95 = percentile(&mut lat, 0.95) as f64 / 1_000_000.0;
    let p99 = percentile(&mut lat, 0.99) as f64 / 1_000_000.0;

    let mean = (latencies.iter().sum::<u128>() as f64 / latencies.len() as f64) / 1_000_000.0;

    println!("\n================= 📊 Throughput Report =================");
    println!("Total time:              {:.3} sec", elapsed);
    println!("Total ops:               {:.0}", total_ops);
    println!("--------------------------------------------------------");
    println!("🔥 Total TPS:            {:.0} tx/sec", tps);
    println!("🔥 Per-core TPS:         {:.0} tx/sec/core", per_core);
    println!("--------------------------------------------------------");
    println!("⏱️ Latency Mean:         {:.3} ms", mean);
    println!("⏱️ p50:                  {:.3} ms", p50);
    println!("⏱️ p95:                  {:.3} ms", p95);
    println!("⏱️ p99:                  {:.3} ms", p99);
    println!("========================================================");
}