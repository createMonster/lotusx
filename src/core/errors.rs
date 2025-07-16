use thiserror::Error;

/// Core exchange error type - simplified and focused
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

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("WebSocket connection closed: {0}")]
    WebSocketClosed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Feature not supported: {0}")]
    NotSupported(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl ExchangeError {
    /// Create common error types - simple constructors
    pub fn api_error(code: i32, message: String) -> Self {
        Self::ApiError { code, message }
    }

    pub fn auth_error(message: String) -> Self {
        Self::AuthError(message)
    }

    pub fn network_error(message: String) -> Self {
        Self::NetworkError(message)
    }

    pub fn rate_limit_exceeded(message: String) -> Self {
        Self::RateLimitExceeded(message)
    }

    /// Convert HTTP status codes to appropriate error types
    pub fn from_http_status(status_code: u16, response_body: &str) -> Self {
        match status_code {
            401 | 403 => Self::AuthError("Authentication failed".to_string()),
            429 => Self::RateLimitExceeded("Rate limit exceeded".to_string()),
            500..=599 => Self::ServerError(format!("Server error: {}", response_body)),
            _ => Self::ApiError {
                code: status_code as i32,
                message: response_body.to_string(),
            },
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_)
                | Self::ConnectionTimeout(_)
                | Self::RateLimitExceeded(_)
                | Self::ServerError(_)
                | Self::WebSocketClosed(_)
                | Self::HttpError(_)
        )
    }

    /// Check if the error is auth-related
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::AuthError(_) | Self::AuthenticationRequired)
    }

    /// Get a user-friendly message
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::AuthenticationRequired | Self::AuthError(_) => {
                "Authentication failed - check credentials"
            }
            Self::RateLimitExceeded(_) => "Rate limit exceeded - please wait",
            Self::ServerError(_) => "Server error - try again later",
            Self::NetworkError(_) | Self::HttpError(_) => "Network error - check connection",
            Self::ConnectionTimeout(_) => "Connection timeout - try again",
            Self::WebSocketClosed(_) => "Connection closed - reconnecting",
            Self::InvalidParameters(_) => "Invalid parameters",
            Self::ConfigError(_) | Self::ConfigurationError(_) => "Configuration error",
            Self::JsonError(_) | Self::SerializationError(_) | Self::DeserializationError(_) => {
                "Data parsing error"
            }
            Self::WebSocketError(_) => "WebSocket error",
            Self::InvalidResponseFormat(_) => "Invalid response format",
            Self::ApiError { .. } => "API error",
            Self::Other(_) => "An error occurred",
        }
    }
}

/// Simple extension trait for adding context to errors
pub trait ExchangeErrorExt<T> {
    fn with_context(self, context: &str) -> Result<T, ExchangeError>;
}

impl<T, E> ExchangeErrorExt<T> for Result<T, E>
where
    E: Into<ExchangeError>,
{
    fn with_context(self, context: &str) -> Result<T, ExchangeError> {
        self.map_err(|e| {
            let base_error: ExchangeError = e.into();
            ExchangeError::Other(format!("{}: {}", context, base_error))
        })
    }
}

// Basic conversion for common exchange errors
// Note: ExchangeError already implements std::error::Error so automatic conversion exists

// Add basic From implementations for exchange-specific errors
impl From<ExchangeError> for String {
    fn from(err: ExchangeError) -> Self {
        err.to_string()
    }
}
