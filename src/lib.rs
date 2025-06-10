pub mod core;
pub mod exchanges;
pub mod utils;

pub use core::{errors::ExchangeError, traits::ExchangeConnector, types::*};
pub use exchanges::binance::BinanceConnector;
