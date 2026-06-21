pub mod lyrics_parse;
pub mod decrypt;
pub mod totp_gen;

pub use lyrics_parse::LyricsParseError;
pub use decrypt::DecryptError;
pub use totp_gen::TotpGenError;

/// 解析器层错误
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    /// 歌词文本解析失败
    #[error("{0}")]
    LyricsParse(#[from] LyricsParseError),

    /// 歌词解密失败
    #[error("{0}")]
    Decrypt(#[from] DecryptError),

    /// Spotify TOTP 验证码生成失败
    #[error("{0}")]
    TotpGenerate(#[from] TotpGenError),
}
