use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;

#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub api_key: Secret<String>,
    pub secret_key: Secret<String>,
    pub testnet: bool,
    pub base_url: Option<String>,
}

// Custom Serialize implementation - never expose secrets in serialization
impl Serialize for ExchangeConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ExchangeConfig", 4)?;
        state.serialize_field("api_key", "[REDACTED]")?;
        state.serialize_field("secret_key", "[REDACTED]")?;
        state.serialize_field("testnet", &self.testnet)?;
        state.serialize_field("base_url", &self.base_url)?;
        state.end()
    }
}

// Custom Deserialize implementation
impl<'de> Deserialize<'de> for ExchangeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ExchangeConfigHelper {
            api_key: String,
            secret_key: String,
            testnet: bool,
            base_url: Option<String>,
        }

        let helper = ExchangeConfigHelper::deserialize(deserializer)?;
        Ok(Self {
            api_key: Secret::new(helper.api_key),
            secret_key: Secret::new(helper.secret_key),
            testnet: helper.testnet,
            base_url: helper.base_url,
        })
    }
}

impl ExchangeConfig {
    /// Create a new configuration with API credentials
    #[must_use]
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key: Secret::new(api_key),
            secret_key: Secret::new(secret_key),
            testnet: false,
            base_url: None,
        }
    }

    /// Create configuration from environment variables
    ///
    /// Expected environment variables:
    /// - `{EXCHANGE}_API_KEY` (e.g., `BINANCE_API_KEY`)
    /// - `{EXCHANGE}_SECRET_KEY` (e.g., `BINANCE_SECRET_KEY`)
    /// - `{EXCHANGE}_TESTNET` (optional, defaults to false)
    /// - `{EXCHANGE}_BASE_URL` (optional)
    pub fn from_env(exchange_prefix: &str) -> Result<Self, ConfigError> {
        let api_key_var = format!("{}_API_KEY", exchange_prefix.to_uppercase());
        let secret_key_var = format!("{}_SECRET_KEY", exchange_prefix.to_uppercase());
        let testnet_var = format!("{}_TESTNET", exchange_prefix.to_uppercase());
        let base_url_var = format!("{}_BASE_URL", exchange_prefix.to_uppercase());

        let api_key = env::var(&api_key_var)
            .map_err(|_| ConfigError::MissingEnvironmentVariable(api_key_var))?;

        let secret_key = env::var(&secret_key_var)
            .map_err(|_| ConfigError::MissingEnvironmentVariable(secret_key_var))?;

        let testnet = env::var(&testnet_var)
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let base_url = env::var(&base_url_var).ok();

        Ok(Self {
            api_key: Secret::new(api_key),
            secret_key: Secret::new(secret_key),
            testnet,
            base_url,
        })
    }

    /// Create configuration from .env file and environment variables
    ///
    /// This method first loads environment variables from a .env file (if it exists),
    /// then reads the configuration using the standard environment variable names.
    ///
    ///
    /// **Security Warning**: Never commit .env files to version control!
    /// Add .env to your .gitignore file.
    #[cfg(feature = "env-file")]
    pub fn from_env_file(exchange_prefix: &str) -> Result<Self, ConfigError> {
        Self::from_env_file_with_path(exchange_prefix, ".env")
    }

    /// Create configuration from a specific .env file path
    ///
    /// This allows you to specify a custom path for your environment file.
    /// Useful for different environments (e.g., .env.development, .env.production)
    #[cfg(feature = "env-file")]
    pub fn from_env_file_with_path(
        exchange_prefix: &str,
        env_file_path: &str,
    ) -> Result<Self, ConfigError> {
        // Load .env file if it exists
        match dotenv::from_path(env_file_path) {
            Ok(_) => {
                // .env file loaded successfully
            }
            Err(dotenv::Error::Io(io_err)) if io_err.kind() == std::io::ErrorKind::NotFound => {
                // .env file doesn't exist, that's okay - continue with system env vars
            }
            Err(e) => {
                return Err(ConfigError::InvalidConfiguration(format!(
                    "Failed to load .env file '{}': {}",
                    env_file_path, e
                )));
            }
        }

        // Now load from environment variables (which may include those from .env)
        Self::from_env(exchange_prefix)
    }

    /// Load configuration with automatic .env file detection
    ///
    /// This method tries multiple common .env file names in order:
    /// 1. .env.local (highest priority)
    /// 2. .env.{environment} (if ENVIRONMENT is set)
    /// 3. .env (default)
    ///
    /// Falls back to system environment variables if no .env files are found.
    #[cfg(feature = "env-file")]
    pub fn from_env_auto(exchange_prefix: &str) -> Result<Self, ConfigError> {
        let env_files = [
            ".env.local",
            &format!(
                ".env.{}",
                env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            ),
            ".env",
        ];

        let mut loaded_any = false;
        for env_file in &env_files {
            match dotenv::from_path(env_file) {
                Ok(_) => {
                    loaded_any = true;
                    break; // Load only the first file found
                }
                Err(dotenv::Error::Io(io_err)) if io_err.kind() == std::io::ErrorKind::NotFound => {
                    // File doesn't exist, try next
                }
                Err(e) => {
                    return Err(ConfigError::InvalidConfiguration(format!(
                        "Failed to load .env file '{}': {}",
                        env_file, e
                    )));
                }
            }
        }

        if !loaded_any {
            // No .env files found, that's okay - will use system environment variables
        }

        // Load from environment variables
        Self::from_env(exchange_prefix)
    }

    /// Create configuration for read-only operations (market data only)
    /// This doesn't require API credentials for public endpoints
    #[must_use]
    pub fn read_only() -> Self {
        Self {
            api_key: Secret::new(String::new()),
            secret_key: Secret::new(String::new()),
            testnet: false,
            base_url: None,
        }
    }

    /// Check if this configuration has valid credentials for authenticated operations
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        !self.api_key.expose_secret().is_empty() && !self.secret_key.expose_secret().is_empty()
    }

    /// Set testnet mode
    #[must_use]
    pub const fn testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    /// Set custom base URL
    #[must_use]
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    /// Get API key (use carefully - exposes secret)
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret()
    }

    /// Get secret key (use carefully - exposes secret)
    pub fn secret_key(&self) -> &str {
        self.secret_key.expose_secret()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvironmentVariable(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}
