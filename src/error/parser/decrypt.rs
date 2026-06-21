/// 歌词解密失败
#[derive(Debug, thiserror::Error)]
pub enum DecryptError {
    /// Base64 解码失败（KRC、QRC 的第一步）
    #[error("base64 decode failed: {detail} (len={len})")]
    Base64Decode { detail: String, len: usize },

    /// XOR 解密失败（KRC 第二步）
    #[error("XOR decrypt failed: {detail}")]
    XorDecrypt { detail: String },

    /// zlib/deflate 解压失败（KRC/QRC 最后一步）
    #[error("deflate decompress failed: {detail}")]
    Deflate { detail: String },

    /// AES-128-ECB 解密失败（网易云 EAPI）
    #[error("AES decrypt failed: {detail}")]
    AesDecrypt { detail: String },

    /// Triple-DES 解密失败（QRC）
    #[error("3DES decrypt failed: {detail}")]
    TripleDesDecrypt { detail: String },

    /// 解密后的字节不是合法 UTF-8
    #[error("decrypted data is not valid UTF-8: {detail}")]
    Utf8Decode { detail: String },

    /// 密钥材料长度不正确
    #[error("invalid key length: expected={expected}, actual={actual}")]
    InvalidKeyLength { expected: usize, actual: usize },
}
