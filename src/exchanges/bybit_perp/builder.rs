use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::bybit_perp::{
    codec::BybitPerpCodec, connector::BybitPerpConnector, signer::BybitPerpSigner,
};
use std::sync::Arc;

/// Create a Bybit Perpetual connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<BybitPerpConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    let base_url = if config.testnet {
        "https://api-testnet.bybit.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.bybit.com".to_string())
    };

    let rest_config = RestClientConfig::new(base_url, "bybit_perp".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    if config.has_credentials() {
        let signer = Arc::new(BybitPerpSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;
    Ok(BybitPerpConnector::new_without_ws(rest, config))
}

/// Create a Bybit Perpetual connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    BybitPerpConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BybitPerpCodec>>,
    ExchangeError,
> {
    let base_url = if config.testnet {
        "https://api-testnet.bybit.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.bybit.com".to_string())
    };

    let rest_config = RestClientConfig::new(base_url, "bybit_perp".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    if config.has_credentials() {
        let signer = Arc::new(BybitPerpSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    let ws_url = if config.testnet {
        "wss://stream-testnet.bybit.com/v5/public/linear".to_string()
    } else {
        "wss://stream.bybit.com/v5/public/linear".to_string()
    };

    let ws = TungsteniteWs::new(ws_url, "bybit_perp".to_string(), BybitPerpCodec::new());
    Ok(BybitPerpConnector::new(rest, ws, config))
}

/// Legacy function for backward compatibility
pub fn create_bybit_perp_connector(
    config: ExchangeConfig,
) -> Result<
    BybitPerpConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BybitPerpCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}
