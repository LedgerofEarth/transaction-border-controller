#[derive(Debug)]
pub enum TgpMessage {
    Dummy,
}

#[derive(thiserror::Error, Debug)]
pub enum TgpError {
    #[error("unimplemented")]
    Unimplemented,
}

pub fn parse_message(_raw: &[u8]) -> Result<TgpMessage, TgpError> {
    Err(TgpError::Unimplemented)
}