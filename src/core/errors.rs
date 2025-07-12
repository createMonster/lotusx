use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("API error: {code} - {message}")]
    ApiError { code: i32, message: String },

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] crate::core::config::ConfigError),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Other error: {0}")]
    Other(String),

    #[error("Context error: {0}")]
    ContextError(#[from] anyhow::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Authentication required: API credentials not provided")]
    AuthenticationRequired,

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("WebSocket connection closed: {0}")]
    WebSocketClosed(String),
}

// Add conversions for new typed errors
impl From<crate::exchanges::bybit::BybitError> for ExchangeError {
    fn from(err: crate::exchanges::bybit::BybitError) -> Self {
        match err {
            crate::exchanges::bybit::BybitError::ApiError { code, message } => {
                Self::ApiError { code, message }
            }
            crate::exchanges::bybit::BybitError::AuthError { reason } => Self::AuthError(reason),
            crate::exchanges::bybit::BybitError::InvalidOrder { details } => {
                Self::InvalidParameters(details)
            }
            crate::exchanges::bybit::BybitError::NetworkError(req_err) => Self::HttpError(req_err),
            crate::exchanges::bybit::BybitError::JsonError(json_err) => Self::JsonError(json_err),
            crate::exchanges::bybit::BybitError::RateLimit { endpoint } => {
                Self::NetworkError(format!("Rate limit exceeded for endpoint: {}", endpoint))
            }
            crate::exchanges::bybit::BybitError::SymbolNotFound { symbol } => {
                Self::InvalidParameters(format!("Symbol not found: {}", symbol))
            }
            crate::exchanges::bybit::BybitError::InsufficientBalance => {
                Self::InvalidParameters("Insufficient balance for operation".to_string())
            }
        }
    }
}

impl From<crate::exchanges::bybit_perp::BybitPerpError> for ExchangeError {
    fn from(err: crate::exchanges::bybit_perp::BybitPerpError) -> Self {
        match err {
            crate::exchanges::bybit_perp::BybitPerpError::ApiError { code, message } => {
                Self::ApiError { code, message }
            }
            crate::exchanges::bybit_perp::BybitPerpError::AuthError { reason } => {
                Self::AuthError(reason)
            }
            crate::exchanges::bybit_perp::BybitPerpError::InvalidOrder { details } => {
                Self::InvalidParameters(details)
            }
            crate::exchanges::bybit_perp::BybitPerpError::NetworkError(req_err) => {
                Self::HttpError(req_err)
            }
            crate::exchanges::bybit_perp::BybitPerpError::JsonError(json_err) => {
                Self::JsonError(json_err)
            }
            crate::exchanges::bybit_perp::BybitPerpError::RateLimit { endpoint } => {
                Self::NetworkError(format!("Rate limit exceeded for endpoint: {}", endpoint))
            }
            crate::exchanges::bybit_perp::BybitPerpError::ContractNotFound { symbol } => {
                Self::InvalidParameters(format!("Contract not found: {}", symbol))
            }
            crate::exchanges::bybit_perp::BybitPerpError::InsufficientMargin => {
                Self::InvalidParameters("Insufficient margin for position".to_string())
            }
            crate::exchanges::bybit_perp::BybitPerpError::PositionSizeExceeded {
                max,
                requested,
            } => Self::InvalidParameters(format!(
                "Position size exceeds limit: max={}, requested={}",
                max, requested
            )),
            crate::exchanges::bybit_perp::BybitPerpError::InvalidLeverage {
                min,
                max,
                requested,
            } => Self::InvalidParameters(format!(
                "Leverage out of range: min={}, max={}, requested={}",
                min, max, requested
            )),
        }
    }
}

impl From<crate::exchanges::hyperliquid::HyperliquidError> for ExchangeError {
    fn from(err: crate::exchanges::hyperliquid::HyperliquidError) -> Self {
        match err {
            crate::exchanges::hyperliquid::HyperliquidError::ApiError { message } => {
                Self::Other(format!("Hyperliquid API error: {}", message))
            }
            crate::exchanges::hyperliquid::HyperliquidError::AuthError { reason } => {
                Self::AuthError(reason)
            }
            crate::exchanges::hyperliquid::HyperliquidError::InvalidOrder { details } => {
                Self::InvalidParameters(details)
            }
            crate::exchanges::hyperliquid::HyperliquidError::NetworkError(req_err) => {
                Self::HttpError(req_err)
            }
            crate::exchanges::hyperliquid::HyperliquidError::JsonError(json_err) => {
                Self::JsonError(json_err)
            }
            crate::exchanges::hyperliquid::HyperliquidError::RateLimit { operation } => {
                Self::NetworkError(format!("Rate limit exceeded for operation: {}", operation))
            }
            crate::exchanges::hyperliquid::HyperliquidError::AssetNotFound { symbol } => {
                Self::InvalidParameters(format!("Asset not found: {}", symbol))
            }
            crate::exchanges::hyperliquid::HyperliquidError::InsufficientMargin => {
                Self::InvalidParameters("Insufficient margin for position".to_string())
            }
            crate::exchanges::hyperliquid::HyperliquidError::PositionSizeExceeded {
                max,
                requested,
            } => Self::InvalidParameters(format!(
                "Position size exceeds limit: max={}, requested={}",
                max, requested
            )),
            crate::exchanges::hyperliquid::HyperliquidError::SignatureError => {
                Self::AuthError("Invalid signature or nonce".to_string())
            }
            crate::exchanges::hyperliquid::HyperliquidError::VaultError { operation } => {
                Self::InvalidParameters(format!("Vault operation not supported: {}", operation))
            }
            crate::exchanges::hyperliquid::HyperliquidError::WebSocketError { reason } => {
                Self::NetworkError(format!("WebSocket connection failed: {}", reason))
            }
        }
    }
}

// Add reverse conversions for the helper traits
impl From<ExchangeError> for crate::exchanges::bybit::BybitError {
    fn from(err: ExchangeError) -> Self {
        match err {
            ExchangeError::HttpError(req_err) => Self::NetworkError(req_err),
            ExchangeError::JsonError(json_err) => Self::JsonError(json_err),
            ExchangeError::ApiError { code, message } => Self::ApiError { code, message },
            ExchangeError::AuthError(reason) => Self::AuthError { reason },
            ExchangeError::InvalidParameters(details) => Self::InvalidOrder { details },
            ExchangeError::NetworkError(msg) => Self::AuthError { reason: msg },
            _ => Self::AuthError {
                reason: err.to_string(),
            },
        }
    }
}

impl From<ExchangeError> for crate::exchanges::bybit_perp::BybitPerpError {
    fn from(err: ExchangeError) -> Self {
        match err {
            ExchangeError::HttpError(req_err) => Self::NetworkError(req_err),
            ExchangeError::JsonError(json_err) => Self::JsonError(json_err),
            ExchangeError::ApiError { code, message } => Self::ApiError { code, message },
            ExchangeError::AuthError(reason) => Self::AuthError { reason },
            ExchangeError::InvalidParameters(details) => Self::InvalidOrder { details },
            ExchangeError::NetworkError(msg) => Self::AuthError { reason: msg },
            _ => Self::AuthError {
                reason: err.to_string(),
            },
        }
    }
}

impl From<ExchangeError> for crate::exchanges::hyperliquid::HyperliquidError {
    fn from(err: ExchangeError) -> Self {
        match err {
            ExchangeError::HttpError(req_err) => Self::NetworkError(req_err),
            ExchangeError::JsonError(json_err) => Self::JsonError(json_err),
            ExchangeError::AuthError(reason) => Self::AuthError { reason },
            ExchangeError::InvalidParameters(details) => Self::InvalidOrder { details },
            ExchangeError::NetworkError(msg) => Self::AuthError { reason: msg },
            _ => Self::AuthError {
                reason: err.to_string(),
            },
        }
    }
}

// Helper trait to add context to Results
pub trait ResultExt<T> {
    fn with_exchange_context<F>(self, f: F) -> Result<T, ExchangeError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<ExchangeError>,
{
    fn with_exchange_context<F>(self, f: F) -> Result<T, ExchangeError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error: ExchangeError = e.into();
            let context = f();
            ExchangeError::ContextError(anyhow::Error::from(base_error).context(context))
        })
    }
}

/// Utility functions for common error handling patterns
///
/// These functions provide consistent error handling across all exchange implementations
/// based on patterns learned during the Binance refactoring.
impl ExchangeError {
    /// Create an authentication required error
    pub fn authentication_required() -> Self {
        Self::AuthenticationRequired
    }

    /// Create a rate limit exceeded error with context
    pub fn rate_limit_exceeded(endpoint: &str) -> Self {
        Self::RateLimitExceeded(format!("Rate limit exceeded for endpoint: {}", endpoint))
    }

    /// Create a server error with HTTP status code
    pub fn server_error(status_code: u16, message: &str) -> Self {
        Self::ServerError(format!("HTTP {}: {}", status_code, message))
    }

    /// Create an invalid response format error
    pub fn invalid_response_format(expected: &str, actual: &str) -> Self {
        Self::InvalidResponseFormat(format!("Expected {}, got {}", expected, actual))
    }

    /// Create a connection timeout error
    pub fn connection_timeout(operation: &str) -> Self {
        Self::ConnectionTimeout(format!("Timeout during {}", operation))
    }

    /// Create a WebSocket closed error
    pub fn websocket_closed(reason: &str) -> Self {
        Self::WebSocketClosed(reason.to_string())
    }

    /// Convert HTTP status codes to appropriate error types
    pub fn from_http_status(status_code: u16, response_body: &str) -> Self {
        match status_code {
            401 => Self::AuthError("Invalid API credentials".to_string()),
            403 => Self::AuthError("Access denied".to_string()),
            429 => Self::RateLimitExceeded("API rate limit exceeded".to_string()),
            500..=599 => Self::ServerError(format!("Server error: {}", response_body)),
            _ => Self::ApiError {
                code: status_code as i32,
                message: response_body.to_string(),
            },
        }
    }

    /// Check if the error is retryable (network issues, rate limits, server errors)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_)
                | Self::ConnectionTimeout(_)
                | Self::RateLimitExceeded(_)
                | Self::ServerError(_)
                | Self::WebSocketClosed(_)
        )
    }

    /// Check if the error is related to authentication
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::AuthError(_) | Self::AuthenticationRequired)
    }

    /// Check if the error is a client-side error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::AuthError(_)
                | Self::AuthenticationRequired
                | Self::InvalidParameters(_)
                | Self::ConfigurationError(_)
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> &str {
        match self {
            Self::AuthenticationRequired => "Please provide valid API credentials",
            Self::AuthError(_) => "Authentication failed - check your API key and secret",
            Self::RateLimitExceeded(_) => "Rate limit exceeded - please try again later",
            Self::ServerError(_) => "Server error - please try again later",
            Self::NetworkError(_) => "Network error - please check your connection",
            Self::ConnectionTimeout(_) => "Connection timeout - please try again",
            Self::WebSocketClosed(_) => "WebSocket connection closed - attempting to reconnect",
            Self::InvalidParameters(_) => "Invalid parameters provided",
            Self::ConfigurationError(_) => "Configuration error - please check your settings",
            _ => "An error occurred",
        }
    }
}

/// Helper trait for adding context to errors in a fluent manner
pub trait ExchangeErrorExt<T> {
    /// Add context about the exchange operation
    fn with_exchange_context(self, exchange: &str, operation: &str) -> Self;

    /// Add context about the symbol being processed
    fn with_symbol_context(self, symbol: &str) -> Self;

    /// Add context about the API endpoint
    fn with_endpoint_context(self, endpoint: &str) -> Self;
}

impl<T> ExchangeErrorExt<T> for Result<T, ExchangeError> {
    fn with_exchange_context(self, exchange: &str, operation: &str) -> Self {
        self.map_err(|e| {
            ExchangeError::ContextError(anyhow::anyhow!("{} {}: {}", exchange, operation, e))
        })
    }

    fn with_symbol_context(self, symbol: &str) -> Self {
        self.map_err(|e| ExchangeError::ContextError(anyhow::anyhow!("Symbol {}: {}", symbol, e)))
    }

    fn with_endpoint_context(self, endpoint: &str) -> Self {
        self.map_err(|e| {
            ExchangeError::ContextError(anyhow::anyhow!("Endpoint {}: {}", endpoint, e))
        })
    }
}
