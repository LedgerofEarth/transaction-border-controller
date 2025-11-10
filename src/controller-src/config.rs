#[derive(Clone, Debug)]
pub struct ControllerConfig {
    pub listen_addr: String,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".to_string(),
        }
    }
}