use crate::error::{
    GeneralError, LyrixResult,
};
use reqwest::Client;
use std::sync::{Arc, Mutex};
use crate::models::{LineInfo, LyricsData, TrackMetadata};
use crate::providers::lyrics_provider::fetch_lyrics_from_player;
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
