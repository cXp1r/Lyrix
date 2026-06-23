/// 代理配置错误
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// 代理 URL 格式不正确
    #[error("invalid proxy URL: {url} ({reason})")]
    InvalidUrl { url: String, reason: String },
}
