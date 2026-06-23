/// HTTP 请求错误 —— 不直接暴露 reqwest::Error
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    /// 400 Bad Request
    #[error("bad request (400): {url}")]
    BadRequest { url: String },

    /// 401 Unauthorized
    #[error("unauthorized (401): {url}")]
    Unauthorized { url: String },

    /// 403 Forbidden
    #[error("forbidden (403): {url}")]
    Forbidden { url: String },

    /// 404 Not Found
    #[error("not found (404): {url}")]
    NotFound { url: String },

    /// 429 Too Many Requests
    #[error("too many requests (429): {url}")]
    TooManyRequests { url: String },

    /// 500 Internal Server Error
    #[error("server error (500): {url}")]
    ServerError { url: String },

    /// 502 Bad Gateway
    #[error("bad gateway (502): {url}")]
    BadGateway { url: String },

    /// 503 Service Unavailable
    #[error("service unavailable (503): {url}")]
    ServiceUnavailable { url: String },

    /// 意外的重定向 (301/302)
    #[error("unexpected redirect ({status}): {url}")]
    Redirect { status: u16, url: String },

    /// 其他 HTTP 状态码
    #[error("HTTP {status}: {url}")]
    OtherStatus { status: u16, url: String },

    /// 连接失败 —— DNS 解析失败 / TCP 连接被拒绝
    #[error("connection failed: {detail} (url={url})")]
    ConnectionFailed { detail: String, url: String },

    /// 请求超时
    #[error("request timeout: {url}")]
    Timeout { url: String },

    /// TLS/SSL 握手错误
    #[error("TLS error: {detail} (url={url})")]
    TlsError { detail: String, url: String },
}
