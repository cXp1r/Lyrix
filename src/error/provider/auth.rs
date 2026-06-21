/// 鉴权错误
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// 缺少必要的 token/cookie
    #[error("{provider}: missing {field}, call set_session first")]
    MissingCredential { provider: String, field: String },

    /// 凭证过期
    #[error("{provider}: {field} expired")]
    CredentialExpired { provider: String, field: String },
}
