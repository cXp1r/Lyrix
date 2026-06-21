use lyrix::parsers::decrypt;

// ===== QRC 解密单元测试 =====

#[test]
fn qrc_hex_to_bytes_valid() {
    // hex_string_to_byte_array is private, but qrc_decrypt calls it
    // We test qrc_decrypt with invalid input
    let result = decrypt::qrc::qrc_decrypt("not-hex");
    assert!(result.is_err());
}

#[test]
fn qrc_odd_length_hex() {
    let result = decrypt::qrc::qrc_decrypt("abc");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("3DES") || msg.contains("hex"));
}

#[test]
fn qrc_empty_input() {
    // 空字符串 hex decode 后为空字节 → 解密过程不会失败，返回空字符串
    let result = decrypt::qrc::qrc_decrypt("");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn qrc_wrong_block_alignment() {
    // hex pairs that aren't multiple of 8 bytes (16 hex chars)
    let result = decrypt::qrc::qrc_decrypt("aabb");
    assert!(result.is_err());
}

// ===== KRC 解密单元测试 =====

#[test]
fn krc_empty_input() {
    let result = decrypt::krc::krc_decrypt("");
    assert!(result.is_err());
}

#[test]
fn krc_invalid_base64() {
    let result = decrypt::krc::krc_decrypt("!!!invalid!!!");
    assert!(result.is_err());
}

#[test]
fn krc_too_short_after_decode() {
    // "AAAA" decodes to 3 bytes < 4
    let result = decrypt::krc::krc_decrypt("AAAA");
    assert!(result.is_err());
}

// ===== 网易云 EAPI 加密 =====

#[test]
fn eapi_encrypt_produces_hex() {
    let result = decrypt::netease::eapi_encrypt("/api/test", "body").unwrap();
    // 应该是大写 hex 字符串
    assert!(!result.is_empty());
    assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(result.len() % 2, 0);
}

#[test]
fn eapi_encrypt_consistent() {
    let a = decrypt::netease::eapi_encrypt("/api/x", "y").unwrap();
    let b = decrypt::netease::eapi_encrypt("/api/x", "y").unwrap();
    assert_eq!(a, b); // deterministic
}

#[test]
fn eapi_encrypt_different_input() {
    let a = decrypt::netease::eapi_encrypt("/a", "b").unwrap();
    let b = decrypt::netease::eapi_encrypt("/a", "c").unwrap();
    assert_ne!(a, b);
}

// ===== ENCRYPT/DECRYPT constants =====

#[test]
fn qrc_mode_constants() {
    assert_eq!(decrypt::qrc::ENCRYPT, 1);
    assert_eq!(decrypt::qrc::DECRYPT, 0);
}
