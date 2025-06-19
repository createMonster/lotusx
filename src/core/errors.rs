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

    #[error("Other error: {0}")]
    Other(String),

    #[error("Context error: {0}")]
    ContextError(#[from] anyhow::Error),
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
