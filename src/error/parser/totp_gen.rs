/// Spotify TOTP 验证码生成失败
#[derive(Debug, thiserror::Error)]
pub enum TotpGenError {
    /// build_totp() 传入的 index 不在 0..=2 范围内
    #[error("invalid TOTP index: {index} (valid: 0/1/2)")]
    InvalidIndex { index: usize },

    /// 系统时间早于 Unix 纪元（1970-01-01）
    #[error("system clock error: time is before Unix epoch")]
    ClockError,
}
