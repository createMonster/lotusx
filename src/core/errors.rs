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
