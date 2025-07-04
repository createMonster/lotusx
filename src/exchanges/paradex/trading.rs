use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use crate::exchanges::paradex::auth::ParadexAuth;
use crate::exchanges::paradex::ParadexConnector;
use async_trait::async_trait;
use secrecy::ExposeSecret;

#[async_trait]
impl OrderPlacer for ParadexConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())?;
        let token = auth.sign_jwt()?;

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.paradex.trade/v1/orders")
            .bearer_auth(token)
            .json(&order)
            .send()
            .await?;

        let order_response: OrderResponse = response.json().await?;
        Ok(order_response)
    }

    async fn cancel_order(&self, _symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())?;
        let token = auth.sign_jwt()?;

        let client = reqwest::Client::new();
        client
            .delete(&format!("https://api.paradex.trade/v1/orders/{}", order_id))
            .bearer_auth(token)
            .send()
            .await?;

        Ok(())
    }
}
