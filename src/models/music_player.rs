use crate::error::{GeneralError, LyrixResult};

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
        _ => {
            return Err(GeneralError::UnsupportedPlayer {
                name: format!("Unsupported appid: {}", app_id),
            }
            .into())
        }
    })
}
