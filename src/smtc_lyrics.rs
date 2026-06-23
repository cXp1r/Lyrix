use crate::error::provider::auth::AuthError;
use crate::error::{
    GeneralError, LyrixResult, SearcherError,
};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use crate::models::{LineInfo, LyricsData, TrackMetadata, ITrackMetadata};
use crate::searchers::{ISearcher, ISearchResult};
#[derive(Debug, Clone, Default)]
pub struct Session {
    pub applemusic_token: Option<String>,
    pub spotify_cookie: Option<String>,
}

pub struct Lyrix {
    pub session: Arc<Mutex<Option<Session>>>,
    client: Client,
}

impl Lyrix {
    pub fn new(session: Option<Session>) -> Self {
        Self { session: Arc::new(Mutex::new(session)), client: Client::new() }
    }

    pub fn set_session(
        &self,
        session: Option<Session>,
    ) -> LyrixResult<()> {
        let mut guard = self
            .session
            .lock()
            .map_err(|e| GeneralError::Internal {
                detail: format!("failed to set session: {e}"),
            })?;
        *guard = session;
        Ok(())
    }

    pub async fn get_lyrics_with_player(
        &self,
        player: &MusicPlayer,
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
        album_artist: Option<&str>,
        duration_ms: u32,
    ) -> LyrixResult<LyricsData> {
        let track = TrackMetadata {
            title: Some(title.to_string()),
            artist: artist.map(|s| s.to_string()),
            album: album.map(|s| s.to_string()),
            album_artist: album_artist.map(|s| s.to_string()),
            duration_ms: Some(duration_ms),
            ..Default::default()
        };
        let session = self
            .session
            .lock()
            .map_err(|e| GeneralError::Internal {
                detail: format!("failed to get session: {e}"),
            })?
            .clone()
            .unwrap_or_default();
        fetch_lyrics_from_player(
            player,
            &track,
            &session,
            &self.client
        ).await
    }

    pub async fn get_lyrics_with_appid(
        &self,
        app_id: &str,
        title: &str,
        artist: Option<&str>,
        album: Option<&str>,
        album_artist: Option<&str>,
        duration_ms: u32,
    ) -> LyrixResult<LyricsData> {
        let player = id2player(app_id)?;
        let metadata = TrackMetadata {
            title: Some(title.to_string()),
            artist: artist.map(|s| s.to_string()),
            album: album.map(|s| s.to_string()),
            album_artist: album_artist.map(|s| s.to_string()),
            duration_ms: Some(duration_ms),
            ..Default::default()
        };
        let session = self
            .session
            .lock()
            .map_err(|e| GeneralError::Internal {
                detail: format!("failed to get session: {e}"),
            })?
            .clone()
            .unwrap_or_default();
        fetch_lyrics_from_player(&player, &metadata, &session, &self.client).await
    }

    pub fn get_trial_part(&self, raw: LyricsData) -> LyrixResult<LyricsData> {
        let (st, du) = match &raw.track_metadata {
            Some(op) => match &op.trial {
                Some(trial) => (trial[0], trial[1]),
                None => return Err(GeneralError::MissingField {
                    field: "trial info".to_string(),
                }.into()),
            },
            None => return Err(GeneralError::MissingField {
                field: "track_metadata".to_string(),
            }.into()),
        };
        let raw_lines= raw.lines;
        let mut new_lines: Vec<LineInfo> = Vec::new();
        for x in raw_lines {
            if x.start_time < st {
                continue;
            }
            if x.start_time > st + du {
                break;
            }
            new_lines.push(LineInfo { start_time: x.start_time - st, ..x });
        }
        Ok(LyricsData { lines: new_lines, ..raw })
    }
}
// ===== MusicPlayer =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MusicPlayer {
    Kugou,
    Netease,
    QQMusic,
    SodaMusic,
    Spotify,
    AppleMusic,
    LXMusic,
    AnyListen,
}

impl MusicPlayer {
    pub fn display_name(&self) -> &str {
        match self {
            MusicPlayer::Kugou => "酷狗音乐",
            MusicPlayer::Netease => "网易云音乐",
            MusicPlayer::QQMusic => "QQ音乐",
            MusicPlayer::SodaMusic => "汽水音乐",
            MusicPlayer::Spotify => "Spotify",
            MusicPlayer::AppleMusic => "AppleMusic",
            MusicPlayer::LXMusic => "落雪音乐",
            MusicPlayer::AnyListen => "Any Listen",
        }
    }
}

pub fn id2player(app_id: &str) -> LyrixResult<MusicPlayer> {
    Ok(match app_id {
        "cloudmusic.exe" => MusicPlayer::Netease,
        "qqmusic.exe" => MusicPlayer::QQMusic,
        "kugou" => MusicPlayer::Kugou,
        "\u{6c7d}\u{6c34}\u{97f3}\u{4e50}" => MusicPlayer::SodaMusic,
        "AppleInc.AppleMusicWin_nzyj5cx40ttqa!App" => MusicPlayer::AppleMusic,
        "Spotify.exe" => MusicPlayer::Spotify,
        "cn.toside.music.desktop" => MusicPlayer::LXMusic,
        "cn.toside.anylisten.desktop" => MusicPlayer::AnyListen,
        _ => return Err(GeneralError::UnsupportedPlayer {
            name: format!("Unsupported appid: {}", app_id),
        }.into()),
    })
}

///严肃采用trait
#[async_trait]
trait LyricsProvider {
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

///通用函数
async fn fetch_lyrics<P: LyricsProvider>(
    provider: &P,
    track: &dyn ITrackMetadata,
) -> LyrixResult<LyricsData> {
    let label = P::label();

    let searcher = provider.create_searcher().await?;
    let result = searcher
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
        }.into());
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

//本质分发
async fn fetch_lyrics_from_player(
    player: &MusicPlayer,
    track: &dyn ITrackMetadata,
    session: &Session,
    client: &Client,
) -> LyrixResult<LyricsData> {
    match player {
        MusicPlayer::Netease => fetch_lyrics(&NeteaseProvider { client: client.clone() }, track).await,
        MusicPlayer::QQMusic => fetch_lyrics(&QQMusicProvider { client: client.clone() }, track).await,
        MusicPlayer::Kugou => fetch_lyrics(&KugouProvider { client: client.clone() }, track).await,
        MusicPlayer::SodaMusic => fetch_lyrics(&SodaMusicProvider { client: client.clone() }, track).await,
        MusicPlayer::Spotify => {
            let cookie = session
                .spotify_cookie
                .as_ref()
                .ok_or_else(|| AuthError::MissingCredential {
                    provider: "Spotify".to_string(),
                    field: "spotify_cookie".to_string(),
                })?
                .clone();
            fetch_lyrics(&SpotifyProvider { cookie, client: client.clone() }, track).await
        },
        MusicPlayer::AppleMusic => {
            let token = session
                .applemusic_token
                .as_ref()
                .ok_or_else(|| AuthError::MissingCredential {
                    provider: "AppleMusic".to_string(),
                    field: "applemusic_token".to_string(),
                })?
                .clone();
            fetch_lyrics(&AppleMusicProvider { token, client: client.clone() }, track).await
        },
        MusicPlayer::LXMusic => {
            Err(GeneralError::UnsupportedPlayer {
                name: "落雪音乐".to_string(),
            }.into())
        }
        MusicPlayer::AnyListen => {
            Err(GeneralError::UnsupportedPlayer {
                name: "Any Listen".to_string(),
            }.into())
        }

    }
}



struct NeteaseProvider {
    client: Client,
}
struct QQMusicProvider {
    client: Client,
}
struct KugouProvider {
    client: Client,
}
struct SodaMusicProvider {
    client: Client,
}
struct AppleMusicProvider {
    token: String,
    client: Client,
}
struct SpotifyProvider {
    cookie: String,
    client: Client,
}

#[async_trait]
impl LyricsProvider for NeteaseProvider {
    type Searcher = crate::searchers::netease::NeteaseSearcher;
    type Api = crate::fetchers::netease::NeteaseApi;
    type SearchResult = crate::searchers::netease::NeteaseSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::netease::NeteaseSearcher::with_client(self.client.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::netease::NeteaseApi::with_client(self.client.clone()))
    }
    fn label() -> &'static str {
        "网易云"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::netease::{NeteaseParser, NeteaseLrcParser};
        use crate::parsers::IParsers;
        use crate::parsers::lrc::LrcParser;
        let lyric_result = api.get_lyric(&best.id).await?;
        if let Some(yrc) = lyric_result.yrc.and_then(|y| y.lyric) {
            if !yrc.is_empty() {
                return Ok(NeteaseParser {}.parse(yrc)?);
            }
        }
        let lrc = lyric_result.lrc.ok_or_else(|| GeneralError::MissingField {
            field: "网易云: LRC也没有哟".to_string(),
        })?;
        let parser = NeteaseLrcParser { version: lrc.version.unwrap_or(3) as u8 };
        Ok(parser.parse(lrc.lyric.ok_or_else(|| GeneralError::MissingField {
            field: "网易云: LRC也没有哟".to_string(),
        })?)?)
    }
}

#[async_trait]
impl LyricsProvider for QQMusicProvider {
    type Searcher = crate::searchers::qqmusic::QQMusicSearcher;
    type Api = crate::fetchers::qqmusic::QQMusicApi;
    type SearchResult = crate::searchers::qqmusic::QQMusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::qqmusic::QQMusicSearcher::with_client(self.client.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::qqmusic::QQMusicApi::with_client(self.client.clone()))
    }
    fn label() -> &'static str {
        "QQ音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::qqmusic::{QQMusicParser, QQMusicLrcParser};
        use crate::parsers::lrc::LrcParser;
        if let Ok(qrc) = api.get_lyrics_qrc(&best.id.to_string()).await {
            return Ok(QQMusicParser {}.decrypt_and_parse(qrc)?);
        }
        let lyric_result = api
            .get_lyric(&best.mid)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "QQ音乐: 获取歌词失败".to_string(),
            })?;
        if let Some(lrc) = lyric_result.lyric {
            if !lrc.is_empty() {
                return Ok(QQMusicLrcParser {}.parse(lrc)?);
            }
        }
        Err(GeneralError::MissingField {
            field: "QQ音乐: 未获取到歌词内容".to_string(),
        }.into())
    }
}

#[async_trait]
impl LyricsProvider for KugouProvider {
    type Searcher = crate::searchers::kugou::KugouSearcher;
    type Api = crate::fetchers::kugou::KugouApi;
    type SearchResult = crate::searchers::kugou::KugouSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::kugou::KugouSearcher::with_client(self.client.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::kugou::KugouApi::with_client(self.client.clone()))
    }
    fn label() -> &'static str {
        "酷狗"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::kugou::KugouParser;
        let keyword = format!("{} {}", best.title, best.artists.join(", "));
        let lyrics_resp = api
            .get_search_lyrics(Some(&keyword), Some(&best.hash))
            .await?
            .ok_or_else(|| SearcherError::NoResults {
                label: "酷狗".to_string(),
                query: keyword.clone(),
            })?;
        let candidates = lyrics_resp.candidates.unwrap_or_default();
        let candidate = candidates.first().ok_or_else(|| SearcherError::MissingField {
            label: "酷狗".to_string(),
            field: "candidate".to_string(),
        })?;
        let id = candidate.id.as_deref().ok_or_else(|| SearcherError::MissingField {
            label: "酷狗".to_string(),
            field: "id".to_string(),
        })?;
        let access_key = candidate.access_key.as_deref().ok_or_else(|| SearcherError::MissingField {
            label: "酷狗".to_string(),
            field: "accesskey".to_string(),
        })?;
        let dl_resp = api
            .get_download_krc(id, access_key)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "酷狗: 下载 KRC 失败".to_string(),
            })?;
        let krc = dl_resp.content.ok_or_else(|| GeneralError::MissingField {
            field: "酷狗: KRC 内容为空".to_string(),
        })?;
        if krc.is_empty() {
            return Err(GeneralError::MissingField {
                field: "酷狗: KRC 内容为空".to_string(),
            }.into());
        }
        Ok(KugouParser {}.decrypt_and_parse(krc)?)
    }
}

#[async_trait]
impl LyricsProvider for SpotifyProvider {
    type Searcher = crate::searchers::spotify::SpotifySearcher;
    type Api = crate::fetchers::spotify::SpotifyApi;
    type SearchResult = crate::searchers::spotify::SpotifySearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::spotify::SpotifySearcher::with_client(self.client.clone(), self.cookie.clone()).await?)
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::spotify::SpotifyApi::with_client(self.client.clone(), self.cookie.clone()).await?)
    }
    fn label() -> &'static str {
        "Spotify"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::spotify::SpotifyParser;
        let lryics = api
            .get_lyrics(&best.id)
            .await?;
        if lryics.is_empty() {
            return Err(GeneralError::MissingField {
                field: "Spotify: 歌词内容为空".to_string(),
            }.into());
        }
        Ok(SpotifyParser {}.parse(lryics)?)
    }
}
#[async_trait]
impl LyricsProvider for SodaMusicProvider {
    type Searcher = crate::searchers::soda_music::SodaMusicSearcher;
    type Api = crate::fetchers::soda_music::SodaMusicApi;
    type SearchResult = crate::searchers::soda_music::SodaMusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::soda_music::SodaMusicSearcher::with_client(self.client.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::soda_music::SodaMusicApi::with_client(self.client.clone()))
    }
    fn label() -> &'static str {
        "汽水音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::soda_music::SodaParser;
        use crate::parsers::IParsers;
        let detail = api
            .get_detail(&best.id)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "汽水音乐: 获取歌曲详情失败".to_string(),
            })?;
        let lyric_info = detail.lyric.ok_or_else(|| GeneralError::MissingField {
            field: "汽水音乐: 歌曲没有歌词".to_string(),
        })?;
        let content = lyric_info.content.ok_or_else(|| GeneralError::MissingField {
            field: "汽水音乐: 无歌曲详细信息".to_string(),
        })?;
        if content.is_empty() {
            return Err(GeneralError::MissingField {
                field: "汽水音乐: 歌词内容为空".to_string(),
            }.into());
        }
        Ok(SodaParser {}.parse(content)?)
    }
}

#[async_trait]
impl LyricsProvider for AppleMusicProvider {
    type Searcher = crate::searchers::applemusic::ApplemusicSearcher;
    type Api = crate::fetchers::applemusic::ApplemusicApi;
    type SearchResult = crate::searchers::applemusic::ApplemusicSearchResult;

    async fn create_searcher(&self) -> LyrixResult<Self::Searcher> {
        Ok(crate::searchers::applemusic::ApplemusicSearcher::with_client(self.client.clone(), self.token.clone()))
    }
    async fn create_api(&self) -> LyrixResult<Self::Api> {
        Ok(crate::fetchers::applemusic::ApplemusicApi::with_client(self.client.clone(), self.token.clone()))
    }
    fn label() -> &'static str {
        "applemusic"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> LyrixResult<Vec<LineInfo>> {
        use crate::parsers::applemusic::AppleMusicParser;
        let detail = api
            .get_lyric(&best.id)
            .await?
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 获取歌曲详情失败".to_string(),
            })?;
        //虽然是vec 但是实际上只有一项(目前来看是的)
        let lyric_data = detail.data.ok_or_else(|| GeneralError::MissingField {
            field: "applemusic: 歌曲没有歌词".to_string(),
        })?;
        let u = lyric_data.get(0).ok_or_else(|| GeneralError::MissingField {
            field: "applemusic: 歌曲没有歌词".to_string(),
        })?;
        let att = u.attributes.as_ref().ok_or_else(|| GeneralError::MissingField {
            field: "applemusic: 无歌曲详细信息".to_string(),
        })?;
        let lyrics = att
            .ttml_localizations
            .as_ref()
            .ok_or_else(|| GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            })?;
        if lyrics.is_empty() {
            return Err(GeneralError::MissingField {
                field: "applemusic: 歌词内容为空".to_string(),
            }.into());
        }
        Ok(AppleMusicParser {}.parse(lyrics.to_string())?)
    }
}
