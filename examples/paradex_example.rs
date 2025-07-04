use lotusx::{
    core::{config::ExchangeConfig, traits::MarketDataSource},
    exchanges::paradex::ParadexConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("PARADEX")?;
    let connector = ParadexConnector::new(config);

    // Test basic functionality
    let markets = connector.get_markets().await?;
    println!("Found {} markets", markets.len());

    Ok(())
}
