mod applemusic;
mod kugou;
mod moekoe;
mod netease;
mod qqmusic;
mod soda_music;
mod spotify;

use crate::error::fetcher::auth::AuthError;
use crate::error::{GeneralError, LyrixResult, SearcherError};
use crate::models::{ITrackMetadata, LineInfo, LyricsData, MusicPlayer, Session, TrackMetadata};
use crate::parsers::lrc::LrcParser;
use crate::parsers::IParsers;
use crate::searchers::{ISearchResult, ISearcher};
use crate::ws_client::WsClient;
use async_trait::async_trait;
use reqwest::Client;

use applemusic::AppleMusicProvider;
use kugou::KugouProvider;
use netease::NeteaseProvider;
use qqmusic::QQMusicProvider;
use soda_music::SodaMusicProvider;
use spotify::SpotifyProvider;

#[derive(Debug, Clone)]
pub(crate) struct RawLyrics {
    pub(crate) content: String,
    pub(crate) track_metadata: Option<TrackMetadata>,
}

/// Provider only searches and fetches raw lyrics. Parsing is selected later by player.
#[async_trait]
pub(crate) trait LyrixProvider {
    type Searcher: ISearcher;
    type Api: Send + Sync;
    type SearchResult: ISearchResult + 'static;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher>;
    async fn create_api(&self) -> LyrixResult<Self::Api>;
    fn label() -> &'static str;
    async fn fetch(api: &Self::Api, best: &Self::SearchResult) -> LyrixResult<String>;
}

async fn fetch_raw_lyrics<P: LyrixProvider>(
    provider: &P,
    track: &dyn ITrackMetadata,
) -> LyrixResult<RawLyrics> {
    let label = P::label();

    let searcher = provider.create_searcher().await?;
    let result =
        searcher
            .search_for_result(track)
            .await?
            .ok_or_else(|| SearcherError::NoMatch {
                label: label.to_string(),
                title: track.title().unwrap_or_default().to_string(),
            })?;

    let best = result
        .as_any()
        .downcast_ref::<P::SearchResult>()
        .ok_or_else(|| GeneralError::Internal {
            detail: format!("{}: search result type mismatch", label),
        })?;

    let api = provider.create_api().await?;
    let content = P::fetch(&api, best).await?;

    Ok(RawLyrics {
        content,
        track_metadata: Some(TrackMetadata {
            title: Some(best.title().to_string()),
            artist: Some(best.artists().join(", ")),
            album: Some(best.album().to_string()),
            duration_ms: best.duration_ms(),
            score: best.match_score(),
            is_trial: best.is_trial(),
            trial: best.trial(),
            ..Default::default()
        }),
    })
}

pub(crate) fn parse_lyrics_for_player(
    player: &MusicPlayer,
    raw: RawLyrics,
) -> LyrixResult<LyricsData> {
    let content = raw.content;
    let lines: Vec<LineInfo> = match player {
        MusicPlayer::Netease => parse_netease_lyrics(content)?,
        MusicPlayer::QQMusic => parse_qqmusic_lyrics(content)?,
        MusicPlayer::Kugou => crate::parsers::kugou::KugouParser {}.decrypt_and_parse(content)?,
        MusicPlayer::SodaMusic => crate::parsers::soda_music::SodaParser {}.parse(content)?,
        MusicPlayer::Spotify => crate::parsers::spotify::SpotifyParser {}.parse(content)?,
        MusicPlayer::AppleMusic => {
            crate::parsers::applemusic::AppleMusicParser {}.parse(content)?
        }
        MusicPlayer::MoeKoe => crate::parsers::kugou::KugouParser {}.parse(content)?,
        MusicPlayer::LXMusic | MusicPlayer::AnyListen => {
            return Err(GeneralError::UnsupportedPlayer {
                name: player.display_name().to_string(),
            }
            .into())
        }
    };

    if lines.is_empty() {
        return Err(GeneralError::MissingField {
            field: format!("{}: no lyrics content", player.display_name()),
        }
        .into());
    }

    Ok(LyricsData {
        file: None,
        lines,
        track_metadata: raw.track_metadata,
    })
}

fn parse_netease_lyrics(content: String) -> LyrixResult<Vec<LineInfo>> {
    if let Ok(lines) = (crate::parsers::netease::NeteaseParser {}).parse(content.clone()) {
        if !lines.is_empty() {
            return Ok(lines);
        }
    }

    crate::parsers::netease::NeteaseLrcParser { version: 3 }.parse(content)
}

fn parse_qqmusic_lyrics(content: String) -> LyrixResult<Vec<LineInfo>> {
    if let Ok(lines) =
        (crate::parsers::qqmusic::QQMusicParser {}).decrypt_and_parse(content.clone())
    {
        if !lines.is_empty() {
            return Ok(lines);
        }
    }

    crate::parsers::qqmusic::QQMusicLrcParser {}.parse(content)
}

pub(crate) async fn fetch_raw_lyrics_from_player(
    player: &MusicPlayer,
    track: &dyn ITrackMetadata,
    session: &Session,
    client: &Client,
    ws_client: &WsClient,
) -> LyrixResult<RawLyrics> {
    match player {
        MusicPlayer::Netease => {
            fetch_raw_lyrics(
                &NeteaseProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::QQMusic => {
            fetch_raw_lyrics(
                &QQMusicProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::Kugou => {
            fetch_raw_lyrics(
                &KugouProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::SodaMusic => {
            fetch_raw_lyrics(
                &SodaMusicProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::Spotify => {
            let cookie = session
                .spotify_cookie
                .as_ref()
                .ok_or_else(|| AuthError::MissingCredential {
                    provider: "Spotify".to_string(),
                    field: "spotify_cookie".to_string(),
                })?
                .clone();
            fetch_raw_lyrics(
                &SpotifyProvider {
                    cookie,
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::AppleMusic => {
            let token = session
                .applemusic_token
                .as_ref()
                .ok_or_else(|| AuthError::MissingCredential {
                    provider: "AppleMusic".to_string(),
                    field: "applemusic_token".to_string(),
                })?
                .clone();
            fetch_raw_lyrics(
                &AppleMusicProvider {
                    token,
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::MoeKoe => {
            let content = moekoe::fetch_lyrics(ws_client).await?;
            Ok(RawLyrics {
                content,
                track_metadata: None,
            })
        }
        MusicPlayer::LXMusic | MusicPlayer::AnyListen => Err(GeneralError::UnsupportedPlayer {
            name: player.display_name().to_string(),
        }
        .into()),
    }
}

pub(crate) async fn fetch_lyrics_from_player(
    player: &MusicPlayer,
    track: &dyn ITrackMetadata,
    session: &Session,
    client: &Client,
    ws_client: &WsClient,
) -> LyrixResult<LyricsData> {
    let raw = fetch_raw_lyrics_from_player(player, track, session, client, ws_client).await?;
    parse_lyrics_for_player(player, raw)
}
