pub mod parser;
pub mod provider;
pub mod searcher;
pub mod general;

pub use parser::ParserError;
pub use provider::ProviderError;
pub use searcher::SearcherError;
pub use general::GeneralError;

/// 库级别 Result 别名
pub type LyrixResult<T> = Result<T, LyrixError>;

/// Lyrix 库的统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum LyrixError {
    /// 解析器层错误
    #[error("{0}")]
    Parser(#[from] ParserError),

    /// 提供器层错误
    #[error("{0}")]
    Provider(#[from] ProviderError),

    /// 搜索器层错误
    #[error("{0}")]
    Searcher(#[from] SearcherError),

    /// 通用/杂项错误
    #[error("{0}")]
    General(#[from] GeneralError),
}
// ===== From 转换：叶子错误 → LyrixError =====

impl From<parser::lyrics_parse::LyricsParseError> for LyrixError {
    fn from(e: parser::lyrics_parse::LyricsParseError) -> Self {
        LyrixError::Parser(ParserError::LyricsParse(e))
    }
}

impl From<parser::decrypt::DecryptError> for LyrixError {
    fn from(e: parser::decrypt::DecryptError) -> Self {
        LyrixError::Parser(ParserError::Decrypt(e))
    }
}

impl From<parser::totp_gen::TotpGenError> for LyrixError {
    fn from(e: parser::totp_gen::TotpGenError) -> Self {
        LyrixError::Parser(ParserError::TotpGenerate(e))
    }
}

impl From<provider::http::HttpError> for LyrixError {
    fn from(e: provider::http::HttpError) -> Self {
        LyrixError::Provider(ProviderError::Http(e))
    }
}

impl From<provider::json::JsonError> for LyrixError {
    fn from(e: provider::json::JsonError) -> Self {
        LyrixError::Provider(ProviderError::Json(e))
    }
}

impl From<provider::auth::AuthError> for LyrixError {
    fn from(e: provider::auth::AuthError) -> Self {
        LyrixError::Provider(ProviderError::Auth(e))
    }
}

impl From<provider::proxy::ProxyError> for LyrixError {
    fn from(e: provider::proxy::ProxyError) -> Self {
        LyrixError::Provider(ProviderError::Proxy(e))
    }
}

