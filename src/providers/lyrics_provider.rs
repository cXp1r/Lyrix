use crate::error::fetcher::auth::AuthError;
use crate::error::{GeneralError, LyrixResult, SearcherError};
use crate::models::{ITrackMetadata, LineInfo, LyricsData, TrackMetadata, MusicPlayer, Session};
use crate::searchers::{ISearchResult, ISearcher};
use async_trait::async_trait;
use reqwest::Client;

use super::applemusic::AppleMusicProvider;
use super::kugou::KugouProvider;
use super::netease::NeteaseProvider;
use super::qqmusic::QQMusicProvider;
use super::soda_music::SodaMusicProvider;
use super::spotify::SpotifyProvider;

/// LyricsProvider trait —— 定义「搜索 → API → 解析」一条龙服务接口
#[async_trait]
pub(crate) trait LyricsProvider {
    type Searcher: ISearcher;
    type Api: Send + Sync;
    type SearchResult: ISearchResult + 'static;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher>;
    async fn create_api(&self) -> LyrixResult<Self::Api>;
    fn label() -> &'static str;
    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>>;
}

/// 通用编排：搜索 → 类型转换 → API 请求 → 解析 → 组装 LyricsData
async fn fetch_lyrics<P: LyricsProvider>(
    provider: &P,
    track: &dyn ITrackMetadata,
) -> LyrixResult<LyricsData> {
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
            detail: format!("{}: 搜索结果类型不匹配", label),
        })?;

    let api = provider.create_api().await?;
    let lines = P::fetch_and_parse(&api, best).await?;

    if lines.is_empty() {
        return Err(GeneralError::MissingField {
            field: format!("{}: 未获取到歌词内容", label),
        }
        .into());
    }
    Ok(LyricsData {
        file: None,
        lines,
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

/// 按播放器类型分发到具体 Provider
pub(crate) async fn fetch_lyrics_from_player(
    player: &MusicPlayer,
    track: &dyn ITrackMetadata,
    session: &Session,
    client: &Client,
) -> LyrixResult<LyricsData> {
    match player {
        MusicPlayer::Netease => {
            fetch_lyrics(
                &NeteaseProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::QQMusic => {
            fetch_lyrics(
                &QQMusicProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::Kugou => {
            fetch_lyrics(
                &KugouProvider {
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::SodaMusic => {
            fetch_lyrics(
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
            fetch_lyrics(
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
            fetch_lyrics(
                &AppleMusicProvider {
                    token,
                    client: client.clone(),
                },
                track,
            )
            .await
        }
        MusicPlayer::LXMusic => Err(GeneralError::UnsupportedPlayer {
            name: "落雪音乐".to_string(),
        }
        .into()),
        MusicPlayer::AnyListen => Err(GeneralError::UnsupportedPlayer {
            name: "Any Listen".to_string(),
        }
        .into()),
    }
}
