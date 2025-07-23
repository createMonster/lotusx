use crate::core::errors::ExchangeError;
use crate::core::kernel::signer::Signer;
use async_trait::async_trait;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{instrument, trace};

/// REST client trait for making HTTP requests
///
/// This trait provides a unified interface for HTTP operations across different exchanges.
/// Implementations handle the specific authentication and request formatting requirements
/// for each exchange.
#[async_trait]
pub trait RestClient: Send + Sync {
    /// Make a GET request
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body as a JSON value
    async fn get(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError>;

    /// Make a GET request with strongly-typed response
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body deserialized to the specified type
    async fn get_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<T, ExchangeError>;

    /// Make a POST request
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - Request body as JSON value
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body as a JSON value
    async fn post(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError>;

    /// Make a POST request with strongly-typed response
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - Request body as JSON value
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body deserialized to the specified type
    async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<T, ExchangeError>;

    /// Make a PUT request
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - Request body as JSON value
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body as a JSON value
    async fn put(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError>;

    /// Make a PUT request with strongly-typed response
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `body` - Request body as JSON value
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body deserialized to the specified type
    async fn put_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<T, ExchangeError>;

    /// Make a DELETE request
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body as a JSON value
    async fn delete(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError>;

    /// Make a DELETE request with strongly-typed response
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `authenticated` - Whether to sign the request
    ///
    /// # Returns
    /// The response body deserialized to the specified type
    async fn delete_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<T, ExchangeError>;

    /// Make a signed request with custom method
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `body` - Request body as raw bytes
    ///
    /// # Returns
    /// The response body as a JSON value
    async fn signed_request(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<Value, ExchangeError>;

    /// Make a signed request with custom method and strongly-typed response
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `endpoint` - The API endpoint path
    /// * `query_params` - Query parameters as key-value pairs
    /// * `body` - Request body as raw bytes
    ///
    /// # Returns
    /// The response body deserialized to the specified type
    async fn signed_request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<T, ExchangeError>;
}

/// Configuration for the REST client
#[derive(Clone, Debug)]
pub struct RestClientConfig {
    /// Base URL for the API
    pub base_url: String,
    /// Exchange name for logging and tracing
    pub exchange_name: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// User agent string to include in requests
    pub user_agent: String,
}

impl RestClientConfig {
    /// Create a new configuration
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the API
    /// * `exchange_name` - Name of the exchange
    pub fn new(base_url: String, exchange_name: String) -> Self {
        Self {
            base_url,
            exchange_name,
            timeout_seconds: 30,
            max_retries: 3,
            user_agent: "LotusX/1.0".to_string(),
        }
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the user agent string
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
}

/// Builder for creating REST client instances
pub struct RestClientBuilder {
    config: RestClientConfig,
    signer: Option<Arc<dyn Signer>>,
}

impl RestClientBuilder {
    /// Create a new builder with the given configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the REST client
    pub fn new(config: RestClientConfig) -> Self {
        Self {
            config,
            signer: None,
        }
    }

    /// Set the signer for authenticated requests
    ///
    /// # Arguments
    /// * `signer` - The signer to use for authentication
    pub fn with_signer(mut self, signer: Arc<dyn Signer>) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Build the REST client
    ///
    /// # Returns
    /// A new `ReqwestRest` instance
    pub fn build(self) -> Result<ReqwestRest, ExchangeError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .user_agent(&self.config.user_agent)
            .build()
            .map_err(|e| {
                ExchangeError::ConfigurationError(format!("Failed to build HTTP client: {}", e))
            })?;

        Ok(ReqwestRest {
            client,
            config: self.config,
            signer: self.signer,
        })
    }
}

/// Implementation of `RestClient` using reqwest
#[derive(Clone)]
pub struct ReqwestRest {
    client: Client,
    config: RestClientConfig,
    signer: Option<Arc<dyn Signer>>,
}

impl std::fmt::Debug for ReqwestRest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReqwestRest")
            .field("config", &self.config)
            .field("has_signer", &self.signer.is_some())
            .finish_non_exhaustive()
    }
}

impl ReqwestRest {
    /// Create a new `ReqwestRest` instance
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the API
    /// * `exchange_name` - Name of the exchange for logging
    /// * `signer` - Optional signer for authenticated requests
    ///
    /// # Returns
    /// A new `ReqwestRest` instance
    pub fn new(
        base_url: String,
        exchange_name: String,
        signer: Option<Arc<dyn Signer>>,
    ) -> Result<Self, ExchangeError> {
        let config = RestClientConfig::new(base_url, exchange_name);
        RestClientBuilder::new(config)
            .with_signer(signer.unwrap_or_else(|| Arc::new(NoopSigner)))
            .build()
    }

    /// Get the current timestamp in milliseconds
    fn get_timestamp() -> Result<u64, ExchangeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .map_err(|e| ExchangeError::Other(format!("Failed to get timestamp: {}", e)))
    }

    /// Build the full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.config.base_url, endpoint)
    }

    /// Create query string from parameters
    fn create_query_string(params: &[(&str, &str)]) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Handle the response and extract JSON
    #[instrument(skip(self, response), fields(exchange = %self.config.exchange_name, status = %response.status()))]
    async fn handle_response(&self, response: Response) -> Result<Value, ExchangeError> {
        let status = response.status();
        let response_text = response.text().await.map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        trace!("Response body: {}", response_text);

        if status.is_success() {
            serde_json::from_str(&response_text).map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse JSON response: {}", e))
            })
        } else {
            Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: response_text,
            })
        }
    }

    /// Make a request with the given parameters
    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, method = %method, endpoint = %endpoint))]
    async fn make_request(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        let url = self.build_url(endpoint);
        let mut request = self.client.request(method.clone(), &url);

        let query_string = Self::create_query_string(query_params);

        // Handle authentication if required
        if authenticated {
            if let Some(signer) = &self.signer {
                let timestamp = Self::get_timestamp()?;
                let (headers, signed_params) = signer.sign_request(
                    method.as_str(),
                    endpoint,
                    &query_string,
                    body,
                    timestamp,
                )?;

                // Add headers
                for (key, value) in headers {
                    request = request.header(&key, &value);
                }

                // Add signed query parameters
                for (key, value) in signed_params {
                    request = request.query(&[(key, value)]);
                }
            } else {
                return Err(ExchangeError::AuthError(
                    "Authentication required but no signer provided".to_string(),
                ));
            }
        } else {
            // Add query parameters for non-authenticated requests
            for (key, value) in query_params {
                request = request.query(&[(key, value)]);
            }
        }

        // Add body if present and set Content-Type for JSON
        if !body.is_empty() {
            request = request
                .header("Content-Type", "application/json")
                .body(body.to_vec());
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Request failed: {}", e)))?;

        self.handle_response(response).await
    }
}

#[async_trait]
impl RestClient for ReqwestRest {
    #[instrument(skip(self, query_params), fields(exchange = %self.config.exchange_name, endpoint = %endpoint, param_count = query_params.len()))]
    async fn get(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.make_request(Method::GET, endpoint, query_params, &[], authenticated)
            .await
    }

    #[instrument(skip(self, query_params), fields(exchange = %self.config.exchange_name, endpoint = %endpoint, param_count = query_params.len()))]
    async fn get_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<T, ExchangeError> {
        self.make_request(Method::GET, endpoint, query_params, &[], authenticated)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| {
                    ExchangeError::DeserializationError(format!(
                        "Failed to deserialize JSON: {}",
                        e
                    ))
                })
            })
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, endpoint = %endpoint))]
    async fn post(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to serialize request body: {}", e))
        })?;

        self.make_request(Method::POST, endpoint, &[], &body_bytes, authenticated)
            .await
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, endpoint = %endpoint))]
    async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<T, ExchangeError> {
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to serialize request body: {}", e))
        })?;

        self.make_request(Method::POST, endpoint, &[], &body_bytes, authenticated)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| {
                    ExchangeError::DeserializationError(format!(
                        "Failed to deserialize JSON: {}",
                        e
                    ))
                })
            })
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, endpoint = %endpoint))]
    async fn put(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to serialize request body: {}", e))
        })?;

        self.make_request(Method::PUT, endpoint, &[], &body_bytes, authenticated)
            .await
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, endpoint = %endpoint))]
    async fn put_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<T, ExchangeError> {
        let body_bytes = serde_json::to_vec(body).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to serialize request body: {}", e))
        })?;

        self.make_request(Method::PUT, endpoint, &[], &body_bytes, authenticated)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| {
                    ExchangeError::DeserializationError(format!(
                        "Failed to deserialize JSON: {}",
                        e
                    ))
                })
            })
    }

    #[instrument(skip(self, query_params), fields(exchange = %self.config.exchange_name, endpoint = %endpoint, param_count = query_params.len()))]
    async fn delete(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.make_request(Method::DELETE, endpoint, query_params, &[], authenticated)
            .await
    }

    #[instrument(skip(self, query_params), fields(exchange = %self.config.exchange_name, endpoint = %endpoint, param_count = query_params.len()))]
    async fn delete_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<T, ExchangeError> {
        self.make_request(Method::DELETE, endpoint, query_params, &[], authenticated)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| {
                    ExchangeError::DeserializationError(format!(
                        "Failed to deserialize JSON: {}",
                        e
                    ))
                })
            })
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, method = %method, endpoint = %endpoint))]
    async fn signed_request(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<Value, ExchangeError> {
        self.make_request(method, endpoint, query_params, body, true)
            .await
    }

    #[instrument(skip(self, body), fields(exchange = %self.config.exchange_name, method = %method, endpoint = %endpoint))]
    async fn signed_request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<T, ExchangeError> {
        self.make_request(method, endpoint, query_params, body, true)
            .await
            .and_then(|value| {
                serde_json::from_value(value).map_err(|e| {
                    ExchangeError::DeserializationError(format!(
                        "Failed to deserialize JSON: {}",
                        e
                    ))
                })
            })
    }
}

/// No-op signer for testing or non-authenticated requests
struct NoopSigner;

#[async_trait]
impl Signer for NoopSigner {
    fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: &str,
        _body: &[u8],
        _timestamp: u64,
    ) -> Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError> {
        let headers = HashMap::new();
        let signed_params = if query_string.is_empty() {
            Vec::new()
        } else {
            query_string
                .split('&')
                .filter_map(|param| {
                    param
                        .split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect()
        };

        Ok((headers, signed_params))
    }
}
