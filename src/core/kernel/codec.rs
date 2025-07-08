use crate::core::errors::ExchangeError;
use tokio_tungstenite::tungstenite::Message;

/// Codec trait for handling exchange-specific WebSocket message encoding/decoding
///
/// This trait defines the contract for converting between raw WebSocket messages
/// and exchange-specific typed messages. Each exchange should implement this trait
/// to handle their specific message formats.
pub trait WsCodec: Send + Sync + 'static {
    /// The type representing parsed messages from this exchange
    type Message: Send + Sync;

    /// Encode a subscription request into a WebSocket message
    ///
    /// # Arguments
    /// * `streams` - The stream identifiers to subscribe to
    ///
    /// # Returns
    /// A WebSocket message ready to be sent to the exchange
    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError>;

    /// Encode an unsubscription request into a WebSocket message  
    ///
    /// # Arguments
    /// * `streams` - The stream identifiers to unsubscribe from
    ///
    /// # Returns
    /// A WebSocket message ready to be sent to the exchange
    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError>;

    /// Decode a raw WebSocket message into a typed message
    ///
    /// This method should only handle data messages. Control messages (ping, pong, close)
    /// are handled at the transport level.
    ///
    /// # Arguments
    /// * `message` - The raw WebSocket message to decode
    ///
    /// # Returns
    /// - `Ok(Some(message))` - Successfully decoded message
    /// - `Ok(None)` - Message was ignored/filtered by codec
    /// - `Err(error)` - Failed to decode message
    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError>;
}
