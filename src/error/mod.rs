pub mod fetcher;
pub mod general;
pub mod parser;
pub mod searcher;

pub use fetcher::FetcherError;
pub use general::GeneralError;
pub use parser::ParserError;
pub use searcher::SearcherError;

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
    Fetcher(#[from] FetcherError),

    /// 搜索器层错误
    #[error("{0}")]
    Searcher(#[from] SearcherError),

    /// 通用/杂项错误
    #[error("{0}")]
    General(#[from] GeneralError),
}
// ===== From 转换：叶子错误 → LyrixError =====

impl From<parser::parse::ParseError> for LyrixError {
    fn from(e: parser::parse::ParseError) -> Self {
        LyrixError::Parser(ParserError::Parse(e))
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

impl From<fetcher::http::HttpError> for LyrixError {
    fn from(e: fetcher::http::HttpError) -> Self {
        LyrixError::Fetcher(FetcherError::Http(e))
    }
}

impl From<fetcher::json::JsonError> for LyrixError {
    fn from(e: fetcher::json::JsonError) -> Self {
        LyrixError::Fetcher(FetcherError::Json(e))
    }
}

impl From<fetcher::auth::AuthError> for LyrixError {
    fn from(e: fetcher::auth::AuthError) -> Self {
        LyrixError::Fetcher(FetcherError::Auth(e))
    }
}

impl From<fetcher::proxy::ProxyError> for LyrixError {
    fn from(e: fetcher::proxy::ProxyError) -> Self {
        LyrixError::Fetcher(FetcherError::Proxy(e))
    }
}
