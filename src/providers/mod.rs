mod applemusic;
mod kugou;
mod moekoe;
mod netease;
mod qqmusic;
mod soda_music;
mod spotify;

use crate::error::fetcher::auth::AuthError;
use crate::error::{GeneralError, LyrixResult, SearcherError};
use crate::logger;
use crate::models::{ITrackMetadata, LineInfo, LyricsData, MusicPlayer, Session, TrackMetadata};
use crate::parsers::lrc::LrcParser;
use crate::parsers::IParsers;
use crate::searchers::{ISearchResult, ISearcher};
use crate::ws_client::WsClient;
use async_trait::async_trait;
use reqwest::Client;
use tokio::task::JoinSet;

use applemusic::AppleMusicProvider;
use kugou::KugouProvider;
use netease::NeteaseProvider;
use qqmusic::QQMusicProvider;
use soda_music::SodaMusicProvider;
use spotify::SpotifyProvider;

use crate::readers::qqmusic::QQMusicReaders;
use crate::readers::readers::LyrixReader;

#[derive(Debug, Clone)]
pub(crate) struct RawLyrics {
    pub(crate) content: String,
    pub(crate) track_metadata: Option<TrackMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawLyricsSource {
    Local,
    Network,
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

fn track_metadata_snapshot(track: &dyn ITrackMetadata) -> TrackMetadata {
    TrackMetadata {
        title: track.title().map(|s| s.to_string()),
        artist: track.artist().map(|s| s.to_string()),
        album: track.album().map(|s| s.to_string()),
        album_artist: track.album_artist().map(|s| s.to_string()),
        duration_ms: track.duration_ms(),
        ..Default::default()
    }
}

fn log_lyrics_source(provider_label: &str, source: &str) {
    logger::info(
        provider_label,
        format_args!("lyrics selected | source={}", source,),
    );
}

async fn fetch_raw_lyrics_with_reader<P, R>(
    provider: P,
    _reader: R,
    track: &dyn ITrackMetadata,
) -> LyrixResult<RawLyrics>
where
    P: LyrixProvider + Send + Sync + 'static,
    R: LyrixReader + Send + Sync + 'static,
{
    let local_track = track_metadata_snapshot(track);
    let network_track = local_track.clone();
    let provider_label = P::label().to_string();
    let reader_label = R::label().to_string();

    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        let result = R::read_raw(&local_track).await.map(|content| RawLyrics {
            content,
            track_metadata: Some(local_track),
        });
        (RawLyricsSource::Local, result)
    });
    tasks.spawn(async move {
        let result = fetch_raw_lyrics(&provider, &network_track).await;
        (RawLyricsSource::Network, result)
    });

    let mut local_finished = false;
    let mut network_lyrics = None;
    let mut local_error = None;
    let mut network_error = None;
    let mut join_error = None;

    // Local lyrics are authoritative; hold a network hit until the local read fails.
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok((RawLyricsSource::Local, Ok(lyrics))) => {
                tasks.abort_all();
                log_lyrics_source(&provider_label, &format!("reader:{}", reader_label));
                return Ok(lyrics);
            }
            Ok((RawLyricsSource::Local, Err(err))) => {
                local_finished = true;
                local_error = Some(err.to_string());
                if let Some(lyrics) = network_lyrics.take() {
                    tasks.abort_all();
                    log_lyrics_source(&provider_label, "network");
                    return Ok(lyrics);
                }
                if network_error.is_some() {
                    break;
                }
            }
            Ok((RawLyricsSource::Network, Ok(lyrics))) => {
                if local_finished {
                    tasks.abort_all();
                    log_lyrics_source(&provider_label, "network");
                    return Ok(lyrics);
                }
                network_lyrics = Some(lyrics);
            }
            Ok((RawLyricsSource::Network, Err(err))) => {
                network_error = Some(err.to_string());
                if local_finished {
                    break;
                }
            }
            Err(err) => {
                join_error = Some(err.to_string());
            }
        }
    }

    if let Some(lyrics) = network_lyrics {
        log_lyrics_source(&provider_label, "network");
        return Ok(lyrics);
    }

    let mut field = format!("{}: no lyrics content", provider_label);
    if let Some(err) = local_error {
        field.push_str(&format!("; local: {err}"));
    }
    if let Some(err) = network_error {
        field.push_str(&format!("; network: {err}"));
    }
    if let Some(err) = join_error {
        field.push_str(&format!("; join: {err}"));
    }

    Err(GeneralError::MissingField { field }.into())
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
    let raw = if matches!(player, MusicPlayer::QQMusic) {
        fetch_raw_lyrics_with_reader(
            QQMusicProvider {
                client: client.clone(),
            },
            QQMusicReaders,
            track,
        )
        .await?
    } else {
        fetch_raw_lyrics_from_player(player, track, session, client, ws_client).await?
    };

    parse_lyrics_for_player(player, raw)
}
