/// 歌词文本解析失败
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// 碰到错误的歌词结构 —— 缺少 '[' / ']' / ',' 或 tag 格式不符合预期
    #[error("invalid lyrics structure: {detail}")]
    InvalidStructure { detail: String },

    /// 时间戳解析错误 —— 开始时间(s)或持续时间(d)无法解析为整数
    #[error("timestamp parse error: field={field}, raw={raw}")]
    TimestampParse { field: String, raw: String },

    /// offset 计算溢出 —— t2 - t1 下溢、或差值超过 u16 范围
    #[error("offset overflow: t1={t1}, t2={t2}")]
    OffsetOverflow { t1: u32, t2: u32 },

    /// 音节标签解析错误 —— <s,d> 或 <s,d,x> 格式不正确
    #[error("syllable parse error: {detail}")]
    SyllableParse { detail: String },

    /// 歌词内容为空 —— 解析成功但没有任何行
    #[error("empty lyrics content")]
    EmptyContent,

    /// LRC 格式错误（仅标准 LRC，不含逐字格式）
    #[error("invalid LRC format: {detail}")]
    InvalidLrcFormat { detail: String },

    /// 遇到无法识别的同步类型
    #[error("unknown lyrics sync type")]
    UnknownSyncType,
}
