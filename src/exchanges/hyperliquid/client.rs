use super::auth::{generate_nonce, HyperliquidAuth};
#[allow(clippy::wildcard_imports)]
use super::types::*;
use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::traits::{AccountInfo, ExchangeConnector, MarketDataSource, OrderPlacer};
use crate::core::types::{
    Balance, Market, MarketDataType, OrderRequest, OrderResponse, OrderSide, Position,
    SubscriptionType, Symbol, WebSocketConfig,
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc;

const MAINNET_API_URL: &str = "https://api.hyperliquid.xyz";
const TESTNET_API_URL: &str = "https://api.hyperliquid-testnet.xyz";

pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    auth: HyperliquidAuth,
    vault_address: Option<String>,
    is_testnet: bool,
}

impl HyperliquidClient {
    /// Create a new client with configuration
    pub fn new(config: ExchangeConfig) -> Self {
        let is_testnet = config.testnet;
        let has_credentials = config.has_credentials();
        let api_key = if has_credentials {
            Some(config.api_key().to_string())
        } else {
            None
        };
        let base_url_option = config.base_url;

        let base_url = if is_testnet {
            TESTNET_API_URL.to_string()
        } else {
            base_url_option.unwrap_or_else(|| MAINNET_API_URL.to_string())
        };

        let auth = api_key.map_or_else(HyperliquidAuth::new, |key| {
            HyperliquidAuth::with_private_key(&key).unwrap_or_else(|_| HyperliquidAuth::new())
        });

        Self {
            client: Client::new(),
            base_url,
            auth,
            vault_address: None,
            is_testnet,
        }
    }

    /// Create a new client with private key for signing
    pub fn with_private_key(private_key: &str, testnet: bool) -> Result<Self, ExchangeError> {
        let base_url = if testnet {
            TESTNET_API_URL.to_string()
        } else {
            MAINNET_API_URL.to_string()
        };

        let auth = HyperliquidAuth::with_private_key(private_key)?;

        Ok(Self {
            client: Client::new(),
            base_url,
            auth,
            vault_address: None,
            is_testnet: testnet,
        })
    }

    /// Create a read-only client without signing capabilities
    pub fn read_only(testnet: bool) -> Self {
        let base_url = if testnet {
            TESTNET_API_URL.to_string()
        } else {
            MAINNET_API_URL.to_string()
        };

        Self {
            client: Client::new(),
            base_url,
            auth: HyperliquidAuth::new(),
            vault_address: None,
            is_testnet: testnet,
        }
    }

    /// Set vault address for trading
    pub fn with_vault_address(mut self, vault_address: String) -> Self {
        self.vault_address = Some(vault_address);
        self
    }

    /// Get wallet address
    pub fn wallet_address(&self) -> Option<&str> {
        self.auth.wallet_address()
    }

    /// Check if client can sign transactions
    pub fn can_sign(&self) -> bool {
        self.auth.can_sign()
    }

    /// Check if client is in testnet mode
    pub fn is_testnet(&self) -> bool {
        self.is_testnet
    }

    // Private helper methods
    async fn post_info_request<T>(&self, request: &InfoRequest) -> Result<T, ExchangeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/info", self.base_url);

        let response = self.client.post(&url).json(request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Info request failed: {}",
                error_text
            )));
        }

        let result: T = response.json().await?;
        Ok(result)
    }

    async fn post_exchange_request<T>(&self, request: &ExchangeRequest) -> Result<T, ExchangeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/exchange", self.base_url);

        let response = self.client.post(&url).json(request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Exchange request failed: {}",
                error_text
            )));
        }

        let result: T = response.json().await?;
        Ok(result)
    }

    // Helper method to convert Hyperliquid order to core OrderRequest
    fn convert_to_hyperliquid_order(&self, order: &OrderRequest) -> super::types::OrderRequest {
        let is_buy = matches!(order.side, OrderSide::Buy);
        let order_type = match order.order_type {
            crate::core::types::OrderType::Limit => super::types::OrderType::Limit {
                limit: LimitOrder {
                    tif: order.time_in_force.as_ref().map_or(
                        super::types::TimeInForce::Gtc,
                        |tif| match tif {
                            crate::core::types::TimeInForce::GTC => super::types::TimeInForce::Gtc,
                            crate::core::types::TimeInForce::IOC
                            | crate::core::types::TimeInForce::FOK => {
                                super::types::TimeInForce::Ioc
                            }
                        },
                    ),
                },
            },
            crate::core::types::OrderType::Market => super::types::OrderType::Limit {
                limit: LimitOrder {
                    tif: super::types::TimeInForce::Ioc,
                },
            },
            _ => super::types::OrderType::Limit {
                limit: LimitOrder {
                    tif: super::types::TimeInForce::Gtc,
                },
            },
        };

        let price = match order.order_type {
            crate::core::types::OrderType::Market => {
                if is_buy {
                    "999999999".to_string()
                } else {
                    "0.000001".to_string()
                }
            }
            _ => order.price.clone().unwrap_or_else(|| "0".to_string()),
        };

        super::types::OrderRequest {
            coin: order.symbol.clone(),
            is_buy,
            sz: order.quantity.clone(),
            limit_px: price,
            order_type,
            reduce_only: false,
        }
    }

    // Helper method to convert Hyperliquid order response to core OrderResponse
    fn convert_from_hyperliquid_response(
        &self,
        response: &super::types::OrderResponse,
        original_order: &OrderRequest,
    ) -> OrderResponse {
        OrderResponse {
            order_id: "0".to_string(), // Hyperliquid uses different ID system
            client_order_id: String::new(),
            symbol: original_order.symbol.clone(),
            side: original_order.side.clone(),
            order_type: original_order.order_type.clone(),
            quantity: original_order.quantity.clone(),
            price: original_order.price.clone(),
            status: if response.status == "ok" {
                "NEW".to_string()
            } else {
                "REJECTED".to_string()
            },
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

#[async_trait]
impl MarketDataSource for HyperliquidClient {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let request = InfoRequest::Meta;
        let response: Vec<AssetInfo> = self.post_info_request(&request).await?;

        let markets = response
            .into_iter()
            .map(|asset| {
                Market {
                    symbol: Symbol {
                        base: asset.name.clone(),
                        quote: "USD".to_string(), // Hyperliquid uses USD as quote currency
                        symbol: asset.name.clone(),
                    },
                    status: "TRADING".to_string(),
                    base_precision: 8, // Default precision
                    quote_precision: 2,
                    min_qty: Some(asset.sz_decimals.to_string()),
                    max_qty: None,
                    min_price: None,
                    max_price: None,
                }
            })
            .collect();

        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // For now, return an error as WebSocket implementation is complex
        Err(ExchangeError::Other(
            "WebSocket market data not yet implemented for Hyperliquid".to_string(),
        ))
    }

    fn get_websocket_url(&self) -> String {
        if self.is_testnet {
            "wss://api.hyperliquid-testnet.xyz/ws".to_string()
        } else {
            "wss://api.hyperliquid.xyz/ws".to_string()
        }
    }
}

#[async_trait]
impl OrderPlacer for HyperliquidClient {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Private key required for placing orders".to_string(),
            ));
        }

        let hyperliquid_order = self.convert_to_hyperliquid_order(&order);

        let action = json!({
            "type": "order",
            "orders": [hyperliquid_order]
        });

        let signed_request =
            self.auth
                .sign_l1_action(action, self.vault_address.clone(), Some(generate_nonce()))?;

        let response: super::types::OrderResponse =
            self.post_exchange_request(&signed_request).await?;
        Ok(self.convert_from_hyperliquid_response(&response, &order))
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Private key required for cancelling orders".to_string(),
            ));
        }

        let order_id_parsed = order_id
            .parse::<u64>()
            .map_err(|_| ExchangeError::InvalidParameters("Invalid order ID format".to_string()))?;

        let cancel_request = CancelRequest {
            coin: symbol,
            oid: order_id_parsed,
        };

        let action = json!({
            "type": "cancel",
            "cancels": [cancel_request]
        });

        let signed_request =
            self.auth
                .sign_l1_action(action, self.vault_address.clone(), Some(generate_nonce()))?;

        let _response: super::types::OrderResponse =
            self.post_exchange_request(&signed_request).await?;
        Ok(())
    }
}

#[async_trait]
impl AccountInfo for HyperliquidClient {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let balances = vec![
            Balance {
                asset: "USDC".to_string(),
                free: response.margin_summary.account_value,
                locked: response.margin_summary.total_margin_used,
            },
            Balance {
                asset: "USDC".to_string(),
                free: response.withdrawable,
                locked: "0".to_string(),
            },
        ];

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let positions = response
            .asset_positions
            .into_iter()
            .map(|pos| {
                let position_side = if pos.position.szi.parse::<f64>().unwrap_or(0.0) > 0.0 {
                    crate::core::types::PositionSide::Long
                } else {
                    crate::core::types::PositionSide::Short
                };

                Position {
                    symbol: pos.position.coin,
                    position_side,
                    entry_price: pos.position.entry_px.unwrap_or_else(|| "0".to_string()),
                    position_amount: pos.position.szi,
                    unrealized_pnl: pos.position.unrealized_pnl,
                    liquidation_price: None, // Not directly available in Hyperliquid response
                    leverage: pos.position.leverage.value.to_string(),
                }
            })
            .collect();

        Ok(positions)
    }
}

#[async_trait]
impl ExchangeConnector for HyperliquidClient {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HyperliquidClient::read_only(true);
        assert!(!client.can_sign());
        assert!(client.is_testnet);
    }
}
