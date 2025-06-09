pub mod core;
pub mod exchanges;
pub mod utils;

pub use core::{traits::ExchangeConnector, types::*, errors::ExchangeError};
pub use exchanges::binance::BinanceConnector; 