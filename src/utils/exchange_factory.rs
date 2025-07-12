use crate::core::{config::ExchangeConfig, traits::MarketDataSource};
use crate::exchanges::backpack;
use crate::exchanges::{bybit::BybitConnector, hyperliquid, paradex};

/// Configuration for an exchange in the latency test
#[derive(Debug, Clone)]
pub struct ExchangeTestConfig {
    pub name: String,
    pub exchange_type: ExchangeType,
    pub testnet: bool,
    pub base_url: Option<String>,
    pub requires_auth: bool,
    pub symbols: Vec<String>,
}

/// Supported exchange types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExchangeType {
    Binance,
    BinancePerp,
    Bybit,
    BybitPerp,
    Backpack,
    Hyperliquid,
    Paradex,
}

impl std::fmt::Display for ExchangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binance => write!(f, "Binance"),
            Self::BinancePerp => write!(f, "Binance Perp"),
            Self::Bybit => write!(f, "Bybit"),
            Self::BybitPerp => write!(f, "Bybit Perp"),
            Self::Backpack => write!(f, "Backpack"),
            Self::Hyperliquid => write!(f, "Hyperliquid"),
            Self::Paradex => write!(f, "Paradex"),
        }
    }
}

/// Factory for creating exchange connectors
pub struct ExchangeFactory;

impl ExchangeFactory {
    /// Create a connector for the given exchange type
    pub fn create_connector(
        exchange_type: &ExchangeType,
        config: Option<ExchangeConfig>,
        testnet: bool,
    ) -> Result<Box<dyn MarketDataSource>, Box<dyn std::error::Error>> {
        match exchange_type {
            ExchangeType::Binance => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                Ok(Box::new(
                    crate::exchanges::binance::create_binance_connector(cfg)?,
                ))
            }
            ExchangeType::BinancePerp => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                Ok(Box::new(
                    crate::exchanges::binance_perp::create_binance_perp_connector(cfg)?,
                ))
            }
            ExchangeType::Bybit => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                Ok(Box::new(BybitConnector::for_factory(cfg)))
            }
            ExchangeType::BybitPerp => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                Ok(Box::new(crate::exchanges::bybit_perp::build_connector(
                    cfg,
                )?))
            }
            ExchangeType::Backpack => {
                // Backpack requires credentials, so use placeholder values for testing
                let cfg = config.unwrap_or_else(|| {
                    ExchangeConfig::new("placeholder".to_string(), "placeholder".to_string())
                        .testnet(testnet)
                });
                match backpack::create_backpack_connector(cfg, false) {
                    Ok(connector) => Ok(Box::new(connector)),
                    Err(e) => Err(Box::new(e)),
                }
            }
            ExchangeType::Hyperliquid => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                Ok(Box::new(hyperliquid::build_hyperliquid_connector(cfg)?))
            }
            ExchangeType::Paradex => {
                let cfg = config.unwrap_or_else(|| ExchangeConfig::read_only().testnet(testnet));
                match paradex::build_connector(cfg) {
                    Ok(connector) => Ok(Box::new(connector)),
                    Err(e) => Err(Box::new(e)),
                }
            }
        }
    }

    /// Get default test configuration for all exchanges
    pub fn get_default_test_configs() -> Vec<ExchangeTestConfig> {
        vec![
            ExchangeTestConfig {
                name: "Binance Spot".to_string(),
                exchange_type: ExchangeType::Binance,
                testnet: false,
                base_url: None,
                requires_auth: false,
                symbols: vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "ADAUSDT".to_string(),
                ],
            },
            ExchangeTestConfig {
                name: "Binance Perp".to_string(),
                exchange_type: ExchangeType::BinancePerp,
                testnet: false,
                base_url: None,
                requires_auth: false,
                symbols: vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "ADAUSDT".to_string(),
                ],
            },
            ExchangeTestConfig {
                name: "Bybit Spot".to_string(),
                exchange_type: ExchangeType::Bybit,
                testnet: false,
                base_url: None,
                requires_auth: false,
                symbols: vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "ADAUSDT".to_string(),
                ],
            },
            ExchangeTestConfig {
                name: "Bybit Perp".to_string(),
                exchange_type: ExchangeType::BybitPerp,
                testnet: false,
                base_url: None,
                requires_auth: false,
                symbols: vec![
                    "BTCUSDT".to_string(),
                    "ETHUSDT".to_string(),
                    "ADAUSDT".to_string(),
                ],
            },
            ExchangeTestConfig {
                name: "Hyperliquid".to_string(),
                exchange_type: ExchangeType::Hyperliquid,
                testnet: false,
                base_url: None,
                requires_auth: false,
                symbols: vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()],
            },
            ExchangeTestConfig {
                name: "Paradex".to_string(),
                exchange_type: ExchangeType::Paradex,
                testnet: true, // Default to testnet for Paradex
                base_url: None,
                requires_auth: false,
                symbols: vec![
                    "BTC-USD".to_string(),
                    "ETH-USD".to_string(),
                    "SOL-USD".to_string(),
                ],
            },
            // Note: Backpack excluded from default config as it requires valid credentials
        ]
    }

    /// Get test configuration from environment variables
    pub fn get_test_configs_from_env() -> Vec<ExchangeTestConfig> {
        let mut configs = Self::get_default_test_configs();

        // Add Backpack if credentials are available
        if ExchangeConfig::from_env("BACKPACK").is_ok() {
            configs.push(ExchangeTestConfig {
                name: "Backpack".to_string(),
                exchange_type: ExchangeType::Backpack,
                testnet: false,
                base_url: None,
                requires_auth: true,
                symbols: vec!["SOL_USDC".to_string(), "BTC_USDC".to_string()],
            });
        }

        // Add Paradex if credentials are available
        if ExchangeConfig::from_env("PARADEX").is_ok() {
            configs.push(ExchangeTestConfig {
                name: "Paradex (Auth)".to_string(),
                exchange_type: ExchangeType::Paradex,
                testnet: true,
                base_url: None,
                requires_auth: true,
                symbols: vec!["BTC-USD".to_string(), "ETH-USD".to_string()],
            });
        }

        configs
    }

    /// Get available exchange types
    pub fn get_available_exchanges() -> Vec<ExchangeType> {
        vec![
            ExchangeType::Binance,
            ExchangeType::BinancePerp,
            ExchangeType::Bybit,
            ExchangeType::BybitPerp,
            ExchangeType::Backpack,
            ExchangeType::Hyperliquid,
            ExchangeType::Paradex,
        ]
    }
}

/// Builder for custom exchange test configurations
pub struct ExchangeTestConfigBuilder {
    configs: Vec<ExchangeTestConfig>,
}

impl Default for ExchangeTestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExchangeTestConfigBuilder {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    pub fn add_exchange(
        mut self,
        name: String,
        exchange_type: ExchangeType,
        testnet: bool,
    ) -> Self {
        let symbols = match exchange_type {
            ExchangeType::Hyperliquid => vec!["BTC".to_string(), "ETH".to_string()],
            ExchangeType::Backpack => vec!["SOL_USDC".to_string(), "BTC_USDC".to_string()],
            ExchangeType::Paradex => vec!["BTC-USD".to_string(), "ETH-USD".to_string()],
            _ => vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        };

        self.configs.push(ExchangeTestConfig {
            name,
            exchange_type,
            testnet,
            base_url: None,
            requires_auth: matches!(exchange_type, ExchangeType::Backpack),
            symbols,
        });
        self
    }

    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        if let Some(last_config) = self.configs.last_mut() {
            last_config.symbols = symbols;
        }
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        if let Some(last_config) = self.configs.last_mut() {
            last_config.base_url = Some(base_url);
        }
        self
    }

    pub fn build(self) -> Vec<ExchangeTestConfig> {
        self.configs
    }
}
