use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::sync::OnceLock;

/// HFT-optimized configuration with caching
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub api_key: Secret<String>,
    pub secret_key: Secret<String>,
    pub testnet: bool,
    pub base_url: Option<String>,
    // HFT optimization: cache expensive operations
    has_credentials_cache: OnceLock<bool>,
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
            has_credentials_cache: OnceLock::new(),
        })
    }
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            api_key: Secret::new(String::new()),
            secret_key: Secret::new(String::new()),
            testnet: false,
            base_url: None,
            has_credentials_cache: OnceLock::new(),
        }
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
            has_credentials_cache: OnceLock::new(),
        }
    }

    /// Create configuration from environment variables - HFT optimized
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
            has_credentials_cache: OnceLock::new(),
        })
    }

    /// Create configuration from environment file - Replaces dotenv with safer implementation
    ///
    /// This method loads environment variables from a file and reads configuration.
    /// This is a secure replacement for the unmaintained dotenv crate.
    pub fn from_env_file(exchange_prefix: &str) -> Result<Self, ConfigError> {
        Self::from_env_file_with_path(exchange_prefix, ".env")
    }

    /// Create configuration from a specific environment file path
    ///
    /// This allows you to specify a custom path for your environment file.
    /// Useful for different environments (e.g., .env.development, .env.production)
    pub fn from_env_file_with_path(
        exchange_prefix: &str,
        env_file_path: &str,
    ) -> Result<Self, ConfigError> {
        // Load environment file if it exists
        if let Err(e) = Self::load_env_file(env_file_path) {
            if !matches!(e, ConfigError::FileNotFound(_)) {
                return Err(e);
            }
            // File doesn't exist, that's okay - continue with system env vars
        }

        // Now load from environment variables (which may include those from .env)
        Self::from_env(exchange_prefix)
    }

    /// Load configuration with automatic environment file detection
    ///
    /// This method tries multiple common environment file names in order:
    /// 1. .env.local (highest priority)
    /// 2. .env.{environment} (if ENVIRONMENT is set)
    /// 3. .env (default)
    ///
    /// Falls back to system environment variables if no env files are found.
    pub fn from_env_auto(exchange_prefix: &str) -> Result<Self, ConfigError> {
        let env_files = [
            ".env.local",
            &format!(
                ".env.{}",
                env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            ),
            ".env",
        ];

        for env_file in &env_files {
            match Self::load_env_file(env_file) {
                Ok(()) => break,                        // Load only the first file found
                Err(ConfigError::FileNotFound(_)) => {} // File doesn't exist, try next
                Err(e) => return Err(e),                // Real error, propagate
            }
        }

        // Load from environment variables
        Self::from_env(exchange_prefix)
    }

    /// Safe environment file loader - replacement for dotenv
    fn load_env_file(file_path: &str) -> Result<(), ConfigError> {
        use std::fs;
        use std::io::ErrorKind;

        let contents = match fs::read_to_string(file_path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(ConfigError::FileNotFound(file_path.to_string()));
            }
            Err(e) => {
                return Err(ConfigError::InvalidConfiguration(format!(
                    "Failed to read env file '{}': {}",
                    file_path, e
                )));
            }
        };

        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE format
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                // Only set if not already set (system env vars take precedence)
                if env::var(key).is_err() {
                    env::set_var(key, value);
                }
            } else {
                return Err(ConfigError::InvalidConfiguration(format!(
                    "Invalid line format in '{}' at line {}: '{}'",
                    file_path,
                    line_num + 1,
                    line
                )));
            }
        }

        Ok(())
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
            has_credentials_cache: OnceLock::new(),
        }
    }

    /// Check if this configuration has valid credentials for authenticated operations
    /// HFT optimized with caching
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        *self.has_credentials_cache.get_or_init(|| {
            !self.api_key.expose_secret().is_empty() && !self.secret_key.expose_secret().is_empty()
        })
    }

    /// Set testnet mode
    #[must_use]
    pub fn testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        // Clear cache since configuration changed
        self.has_credentials_cache = OnceLock::new();
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

    #[error("File not found: {0}")]
    FileNotFound(String),
}
