pub mod auth;
pub mod http;
pub mod json;
pub mod proxy;

pub use auth::AuthError;
pub use http::HttpError;
pub use json::JsonError;
pub use proxy::ProxyError;

/// 提供器层错误
#[derive(Debug, thiserror::Error)]
pub enum FetcherError {
    /// HTTP 请求错误
    #[error("{0}")]
    Http(#[from] HttpError),

    /// JSON 反序列化错误
    #[error("{0}")]
    Json(#[from] JsonError),

    /// 鉴权错误
    #[error("{0}")]
    Auth(#[from] AuthError),

    /// 代理配置错误
    #[error("{0}")]
    Proxy(#[from] ProxyError),
}
