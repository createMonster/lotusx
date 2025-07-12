use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{ReqwestRest, RestClientBuilder, RestClientConfig};
use crate::exchanges::bybit::connector::BybitConnector;
use crate::exchanges::bybit::signer::BybitSigner;
use std::sync::Arc;

/// Create a Bybit connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest, ()>, ExchangeError> {
    let base_url = config.base_url.clone().unwrap_or_else(|| {
        if config.testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            "https://api.bybit.com".to_string()
        }
    });

    let rest_config = RestClientConfig::new(base_url, "bybit".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    if config.has_credentials() {
        let signer = Arc::new(BybitSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;
    Ok(BybitConnector::new_without_ws(rest, config))
}

/// Build connector with WebSocket support (placeholder)
pub fn build_connector_with_websocket(
    _config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest, ()>, ExchangeError> {
    // For now, return the same as REST-only since WebSocket support is not implemented
    Err(ExchangeError::InvalidParameters(
        "WebSocket support not implemented yet".to_string(),
    ))
}

/// Build connector with reconnection support (placeholder)  
pub fn build_connector_with_reconnection(
    _config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest, ()>, ExchangeError> {
    // For now, return the same as REST-only since reconnection is not implemented
    Err(ExchangeError::InvalidParameters(
        "Reconnection support not implemented yet".to_string(),
    ))
}

/// Legacy compatibility functions
pub fn create_bybit_connector(
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

pub fn create_bybit_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest, ()>, ExchangeError> {
    build_connector_with_reconnection(config)
}

/// Modern kernel-based builder for Bybit spot trading
pub fn bybit_connector_with_rest(
    rest_client: ReqwestRest,
    config: ExchangeConfig,
) -> BybitConnector<ReqwestRest> {
    BybitConnector::new_with_rest(rest_client, config)
}

/// Traditional builder for Bybit spot trading (for backward compatibility)
pub fn build_bybit_spot_connector(
    api_key: String,
    api_secret: String,
    sandbox: bool,
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest>, ExchangeError> {
    let base_url = if sandbox {
        "https://api-testnet.bybit.com"
    } else {
        "https://api.bybit.com"
    };

    let signer = std::sync::Arc::new(BybitSigner::new(api_key, api_secret));
    let rest_config = RestClientConfig::new(base_url.to_string(), "bybit".to_string());

    let rest_client = RestClientBuilder::new(rest_config)
        .with_signer(signer)
        .build()?;

    Ok(BybitConnector::new_with_rest(rest_client, config))
}

/// Legacy builder for backward compatibility
pub fn build_bybit_connector(
    api_key: String,
    secret_key: String,
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest>, ExchangeError> {
    build_bybit_spot_connector(api_key, secret_key, false, config)
}

/// Legacy builder for testnet
pub fn build_bybit_testnet_connector(
    api_key: String,
    secret_key: String,
    config: ExchangeConfig,
) -> Result<BybitConnector<ReqwestRest>, ExchangeError> {
    build_bybit_spot_connector(api_key, secret_key, true, config)
}
