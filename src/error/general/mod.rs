/// 通用/杂项错误 —— 跨模块、不属于解析/提供/搜索三层的错误
#[derive(Debug, thiserror::Error)]
pub enum GeneralError {
    /// 不支持的播放器 / app_id
    #[error("unsupported player: {name}")]
    UnsupportedPlayer { name: String },

    /// 缺少必要字段（trial info、track_metadata 等）
    #[error("missing required field: {field}")]
    MissingField { field: String },

    /// I/O 错误（日志文件写入等）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 内部错误 —— 理论上不应发生，防御性编程用
    #[error("internal error: {detail}")]
    Internal { detail: String },
}
