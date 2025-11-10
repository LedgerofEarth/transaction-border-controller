use crate::config::ControllerConfig;

#[derive(Clone, Debug)]
pub struct Controller {
    pub config: ControllerConfig,
}

impl Controller {
    pub fn new(config: ControllerConfig) -> Self {
        Self { config }
    }

    pub fn handle_tgp_message(&self, raw: &[u8]) -> Result<(), ControllerError> {
        let _msg = crate::tgp::parse_message(raw)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error("TGP parse error: {0}")]
    TgpParse(#[from] crate::tgp::TgpError),
}