pub struct X402Adapter;

impl X402Adapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn initiate_payment(&self) -> anyhow::Result<()> {
        Ok(())
    }
}