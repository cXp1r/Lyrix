use std::fmt;

/// JSON 反序列化错误（包装 serde_json::Error + 上下文）
#[derive(Debug)]
pub struct JsonError {
    /// 是哪个 API 的响应解析失败
    pub api: String,
    /// serde_json 的原始错误
    pub source: serde_json::Error,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} API response parse failed: {}", self.api, self.source)
    }
}

impl std::error::Error for JsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
