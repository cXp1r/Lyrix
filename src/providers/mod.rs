mod applemusic;
mod kugou;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawLyricsFormat {
    NeteaseYrc,
    NeteaseLrc { version: u8 },
    QQMusicQrc,
    QQMusicLrc,
    KugouKrc,
    SodaMusic,
    SpotifyJson,
    AppleMusicTtml,
}

#[derive(Debug, Clone)]
pub(crate) struct RawLyricsContent {
    pub(crate) content: String,
    pub(crate) format: RawLyricsFormat,
}

#[derive(Debug, Clone)]
pub(crate) struct RawLyrics {
    pub(crate) content: String,
    pub(crate) format: RawLyricsFormat,
    pub(crate) track_metadata: Option<TrackMetadata>,
}

/// Provider only searches and fetches raw lyrics. Parsing is selected later by player and format.
#[async_trait]
pub(crate) trait LyrixProvider {
    type Searcher: ISearcher;
    type Api: Send + Sync;
    type SearchResult: ISearchResult + 'static;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher>;
    async fn create_api(&self) -> LyrixResult<Self::Api>;
    fn label() -> &'static str;
    async fn fetch(api: &Self::Api, best: &Self::SearchResult) -> LyrixResult<RawLyricsContent>;
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
    let raw = P::fetch(&api, best).await?;

    Ok(RawLyrics {
        content: raw.content,
        format: raw.format,
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
    let lines: Vec<LineInfo> = match raw.format {
        RawLyricsFormat::NeteaseYrc => {
            crate::parsers::netease::NeteaseParser {}.parse(raw.content)?
        }
        RawLyricsFormat::NeteaseLrc { version } => {
            crate::parsers::netease::NeteaseLrcParser { version }.parse(raw.content)?
        }
        RawLyricsFormat::QQMusicQrc => {
            crate::parsers::qqmusic::QQMusicParser {}.decrypt_and_parse(raw.content)?
        }
        RawLyricsFormat::QQMusicLrc => {
            crate::parsers::qqmusic::QQMusicLrcParser {}.parse(raw.content)?
        }
        RawLyricsFormat::KugouKrc => {
            crate::parsers::kugou::KugouParser {}.decrypt_and_parse(raw.content)?
        }
        RawLyricsFormat::SodaMusic => {
            crate::parsers::soda_music::SodaParser {}.parse(raw.content)?
        }
        RawLyricsFormat::SpotifyJson => {
            crate::parsers::spotify::SpotifyParser {}.parse(raw.content)?
        }
        RawLyricsFormat::AppleMusicTtml => {
            crate::parsers::applemusic::AppleMusicParser {}.parse(raw.content)?
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

async fn fetch_third_party_raw_lyrics(
    player: &MusicPlayer,
    _track: &dyn ITrackMetadata,
    _session: &Session,
    _ws_client: &WsClient,
) -> LyrixResult<RawLyrics> {
    Err(GeneralError::UnsupportedPlayer {
        name: format!("{} third-party fetch placeholder", player.display_name()),
    }
    .into())
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
        MusicPlayer::LXMusic | MusicPlayer::AnyListen => {
            fetch_third_party_raw_lyrics(player, track, session, ws_client).await
        }
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
