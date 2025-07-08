use super::auth::BackpackAuth;
use crate::core::{config::ExchangeConfig, errors::ExchangeError, traits::ExchangeConnector};
use reqwest::Client;

pub struct BackpackConnector {
    pub(crate) client: Client,
    #[allow(dead_code)]
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
    pub(crate) auth: Option<BackpackAuth>,
}

impl BackpackConnector {
    pub fn new(config: ExchangeConfig) -> Result<Self, ExchangeError> {
        let base_url = if config.testnet {
            "https://api.backpack.exchange".to_string() // Backpack doesn't have a testnet
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.backpack.exchange".to_string())
        };

        let auth = if !config.api_key().is_empty() && !config.secret_key().is_empty() {
            Some(BackpackAuth::new(&config)?)
        } else {
            None
        };

        Ok(Self {
            client: Client::new(),
            config,
            base_url,
            auth,
        })
    }

    /// Create query string from parameters
    pub(crate) fn create_query_string(params: &[(String, String)]) -> String {
        let mut sorted_params = params.to_vec();
        sorted_params.sort_by(|a, b| a.0.cmp(&b.0));

        sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Check if authentication is available
    pub fn can_authenticate(&self) -> bool {
        self.auth
            .as_ref()
            .is_some_and(|auth| auth.can_authenticate())
    }

    /// Create signed headers for authenticated requests
    pub(crate) fn create_signed_headers(
        &self,
        instruction: &str,
        params: &str,
    ) -> Result<std::collections::HashMap<String, String>, ExchangeError> {
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| ExchangeError::AuthError("No authentication available".to_string()))?;
        auth.create_signed_headers(instruction, params)
    }

    /// Create WebSocket authentication message for use in examples and consumers
    pub fn create_websocket_auth_message(&self) -> Result<String, ExchangeError> {
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| ExchangeError::AuthError("No authentication available".to_string()))?;
        auth.create_websocket_auth_message()
    }
}

impl ExchangeConnector for BackpackConnector {}
