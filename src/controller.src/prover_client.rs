use coreprover::Receipt;

pub struct ProverClient;

impl ProverClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn submit_receipt(&self, _r: Receipt) -> anyhow::Result<()> {
        Ok(())
    }
}