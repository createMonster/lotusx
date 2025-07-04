use crate::core::config::ExchangeConfig;

#[derive(Debug, Clone)]
pub struct ParadexConnector {
    pub config: ExchangeConfig,
}

impl ParadexConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        Self { config }
    }
}
