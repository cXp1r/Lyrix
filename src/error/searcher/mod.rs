/// 搜索器层错误 —— 搜索/匹配歌曲过程中的失败
#[derive(Debug, thiserror::Error)]
pub enum SearcherError {
    /// 搜索未返回任何结果
    #[error("{label}: no search results (query={query})")]
    NoResults { label: String, query: String },

    /// 搜索结果全部低于分数线
    #[error("{label}: score too low (best={score}/{threshold}, query={query})")]
    LowScore {
        label: String,
        score: i8,
        threshold: i8,
        query: String,
    },

    /// 搜到了候选但没有与曲目元数据匹配的项
    #[error("{label}: no matching track found (title={title})")]
    NoMatch { label: String, title: String },

    /// 搜索结果缺少必要字段（如 id、hash 等）
    #[error("{label}: missing required field ({field})")]
    MissingField { label: String, field: String },
}
