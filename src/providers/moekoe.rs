use crate::error::{GeneralError, LyrixResult};
use crate::ws_client::WsClient;
use futures_util::StreamExt;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

const MOEKOE_WS_URL: &str = "ws://127.0.0.1:6520";
const RECEIVE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
struct WsEnvelope {
    #[serde(rename = "type")]
    kind: String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct LyricsPayload {
    #[serde(rename = "lyricsData")]
    lyrics_data: Option<String>,
}

pub(crate) async fn fetch_lyrics(ws_client: &WsClient) -> LyrixResult<String> {
    let mut stream = ws_client.connect(MOEKOE_WS_URL).await?;
    let deadline = Instant::now() + RECEIVE_TIMEOUT;

    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| GeneralError::Internal {
                detail: "MoeKoe lyric receive timeout".to_string(),
            })?;

        let message = timeout(remaining, stream.next())
            .await
            .map_err(|_| GeneralError::Internal {
                detail: "MoeKoe lyric receive timeout".to_string(),
            })?
            .ok_or_else(|| GeneralError::Internal {
                detail: "MoeKoe websocket closed before lyrics arrived".to_string(),
            })?;

        let message = message.map_err(|e| GeneralError::Internal {
            detail: format!("MoeKoe websocket read failed: {e}"),
        })?;

        if let Some(content) = extract_lyrics(&message)? {
            return Ok(content);
        }
    }
}

fn extract_lyrics(message: &Message) -> LyrixResult<Option<String>> {
    let text = match message {
        Message::Text(text) => text.as_str(),
        Message::Binary(binary) => match std::str::from_utf8(binary) {
            Ok(text) => text,
            Err(_) => return Ok(None),
        },
        Message::Ping(_) | Message::Pong(_) => return Ok(None),
        Message::Close(_) => {
            return Err(GeneralError::Internal {
                detail: "MoeKoe websocket closed before lyrics arrived".to_string(),
            }
            .into())
        }
        _ => return Ok(None),
    };

    let envelope: WsEnvelope = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    if envelope.kind != "lyrics" {
        return Ok(None);
    }

    let payload: LyricsPayload =
        serde_json::from_value(envelope.data).map_err(|e| GeneralError::Internal {
            detail: format!("MoeKoe lyrics payload parse failed: {e}"),
        })?;

    let content = payload
        .lyrics_data
        .ok_or_else(|| GeneralError::MissingField {
            field: "MoeKoe lyricsData".to_string(),
        })?;

    if content.is_empty() {
        return Err(GeneralError::MissingField {
            field: "MoeKoe lyricsData".to_string(),
        }
        .into());
    }

    Ok(Some(content))
}
