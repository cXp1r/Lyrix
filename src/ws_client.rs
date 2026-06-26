use crate::error::{GeneralError, LyrixResult};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, Default)]
pub struct WsClient;

impl WsClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect(&self, url: &str) -> LyrixResult<WsStream> {
        let (stream, _) = connect_async(url)
            .await
            .map_err(|e| GeneralError::Internal {
                detail: format!("websocket connect failed: {e}"),
            })?;
        Ok(stream)
    }
}
