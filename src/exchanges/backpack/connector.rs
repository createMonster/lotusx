use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::{ExchangeConnector, OrderPlacer},
    types::{OrderRequest, OrderResponse},
};
use crate::exchanges::backpack::codec::{BackpackCodec, BackpackMessage};
use crate::exchanges::backpack::types::{
    BackpackBalanceMap, BackpackDepthResponse, BackpackFill, BackpackFundingRate,
    BackpackKlineResponse, BackpackMarketResponse, BackpackOrder, BackpackOrderResponse,
    BackpackPositionResponse, BackpackTickerResponse, BackpackTradeResponse,
};
use async_trait::async_trait;

/// Backpack connector using kernel architecture
pub struct BackpackConnector<R: RestClient, W: WsSession<BackpackCodec>> {
    rest: R,
    ws: Option<W>,
    base_url: String,
    config: ExchangeConfig,
}

impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    /// Create a new Backpack connector with dependency injection
    pub fn new(rest: R, ws: Option<W>, config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://api.backpack.exchange".to_string() // Backpack doesn't have a separate testnet
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.backpack.exchange".to_string())
        };

        Self {
            rest,
            ws,
            base_url,
            config,
        }
    }

    /// Get the base URL for API requests
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if authentication is available
    pub fn can_authenticate(&self) -> bool {
        !self.config.api_key().is_empty() && !self.config.secret_key().is_empty()
    }

    /// Get a mutable reference to the WebSocket session
    pub fn ws_mut(&mut self) -> Option<&mut W> {
        self.ws.as_mut()
    }

    /// Get the current configuration
    pub fn config(&self) -> &ExchangeConfig {
        &self.config
    }

    /// Get the REST client
    pub fn rest(&self) -> &R {
        &self.rest
    }
}

impl<R: RestClient, W: WsSession<BackpackCodec>> ExchangeConnector for BackpackConnector<R, W> {}

/// WebSocket functionality for Backpack
impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    /// Subscribe to WebSocket streams
    pub async fn subscribe_websocket(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.connect().await?;
            ws.subscribe(streams).await?;
        } else {
            return Err(ExchangeError::ConfigurationError(
                "WebSocket session not configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Unsubscribe from WebSocket streams
    pub async fn unsubscribe_websocket(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.unsubscribe(streams).await?;
        } else {
            return Err(ExchangeError::ConfigurationError(
                "WebSocket session not configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Get the next WebSocket message
    pub async fn next_websocket_message(
        &mut self,
    ) -> Option<Result<BackpackMessage, ExchangeError>> {
        if let Some(ws) = &mut self.ws {
            ws.next_message().await
        } else {
            None
        }
    }

    /// Close the WebSocket connection
    pub async fn close_websocket(&mut self) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.close().await?;
        }
        Ok(())
    }

    /// Check if WebSocket is connected
    pub fn is_websocket_connected(&self) -> bool {
        self.ws.as_ref().is_some_and(|ws| ws.is_connected())
    }
}

/// REST API functionality for Backpack
impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    /// Get markets from REST API
    pub async fn get_markets(&self) -> Result<Vec<BackpackMarketResponse>, ExchangeError> {
        let endpoint = "/api/v1/markets";
        self.rest.get_json(endpoint, &[], false).await
    }

    /// Get ticker for a specific symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<BackpackTickerResponse, ExchangeError> {
        let endpoint = "/api/v1/ticker";
        let params = [("symbol", symbol)];
        self.rest.get_json(endpoint, &params, false).await
    }

    /// Get order book for a specific symbol
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BackpackDepthResponse, ExchangeError> {
        let endpoint = "/api/v1/depth";
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, false).await
    }

    /// Get recent trades for a specific symbol
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackTradeResponse>, ExchangeError> {
        let endpoint = "/api/v1/trades";
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, false).await
    }

    /// Get klines for a specific symbol
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackKlineResponse>, ExchangeError> {
        let endpoint = "/api/v1/klines";
        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol), ("interval", interval)];

        if let Some(ref start) = start_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, false).await
    }

    /// Get funding rates
    pub async fn get_funding_rates(&self) -> Result<Vec<BackpackFundingRate>, ExchangeError> {
        let endpoint = "/api/v1/funding/rates";
        self.rest.get_json(endpoint, &[], false).await
    }

    /// Get funding rate history for a specific symbol
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackFundingRate>, ExchangeError> {
        let endpoint = "/api/v1/funding/rates/history";
        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref start) = start_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, false).await
    }
}

/// Implement OrderPlacer trait for Backpack
#[async_trait]
impl<R: RestClient, W: WsSession<BackpackCodec>> OrderPlacer for BackpackConnector<R, W> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert OrderRequest to Backpack API format
        let body = serde_json::json!({
            "symbol": order.symbol.as_str(),
            "side": order.side,
            "type": order.order_type,
            "quantity": order.quantity.value(),
            "price": order.price.map(|p| p.value()),
            "timeInForce": order.time_in_force,
        });

        let _response = self.place_order(&body).await?;

        // For now, return a basic OrderResponse
        // This would need proper parsing of Backpack response format
        Ok(OrderResponse {
            order_id: "0".to_string(),
            client_order_id: String::new(),
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            quantity: order.quantity,
            price: order.price,
            status: "NEW".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let order_id_num = order_id
            .parse::<i64>()
            .map_err(|e| ExchangeError::Other(format!("Invalid order ID format: {}", e)))?;

        self.cancel_order(&symbol, Some(order_id_num), None).await?;
        Ok(())
    }
}

/// Authenticated endpoints for Backpack
impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    /// Get account balances
    pub async fn get_balances(&self) -> Result<BackpackBalanceMap, ExchangeError> {
        let endpoint = "/api/v1/balances";
        self.rest.get_json(endpoint, &[], true).await
    }

    /// Get account positions
    pub async fn get_positions(&self) -> Result<Vec<BackpackPositionResponse>, ExchangeError> {
        let endpoint = "/api/v1/positions";
        self.rest.get_json(endpoint, &[], true).await
    }

    /// Get order history
    pub async fn get_order_history(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackOrder>, ExchangeError> {
        let endpoint = "/api/v1/orders";
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![];

        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, true).await
    }

    /// Place a new order
    pub async fn place_order(
        &self,
        body: &serde_json::Value,
    ) -> Result<BackpackOrderResponse, ExchangeError> {
        let endpoint = "/api/v1/order";
        self.rest.post_json(endpoint, body, true).await
    }

    /// Cancel an order
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BackpackOrderResponse, ExchangeError> {
        let endpoint = "/api/v1/order";
        let mut params = vec![("symbol", symbol)];

        let order_id_str = order_id.map(|id| id.to_string());
        if let Some(ref order_id) = order_id_str {
            params.push(("orderId", order_id.as_str()));
        }
        if let Some(client_order_id) = client_order_id {
            params.push(("clientOrderId", client_order_id));
        }

        self.rest.delete_json(endpoint, &params, true).await
    }

    /// Get fills
    pub async fn get_fills(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackFill>, ExchangeError> {
        let endpoint = "/api/v1/fills";
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![];

        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json(endpoint, &params, true).await
    }
}
