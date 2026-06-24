pub mod decrypt;
pub mod parse;
pub mod totp_gen;

pub use decrypt::DecryptError;
pub use parse::ParseError;
pub use totp_gen::TotpGenError;

/// 解析器层错误
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    /// 歌词文本解析失败
    #[error("{0}")]
    Parse(#[from] ParseError),

    /// 歌词解密失败
    #[error("{0}")]
    Decrypt(#[from] DecryptError),

    /// Spotify TOTP 验证码生成失败
    #[error("{0}")]
    TotpGenerate(#[from] TotpGenError),
}
