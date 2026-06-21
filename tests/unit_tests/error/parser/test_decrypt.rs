use lyrix::error::parser::decrypt::DecryptError;

#[test]
fn base64_decode_display() {
    let e = DecryptError::Base64Decode {
        detail: "invalid padding".into(),
        len: 42,
    };
    let msg = e.to_string();
    assert!(msg.contains("base64 decode failed"));
    assert!(msg.contains("invalid padding"));
    assert!(msg.contains("42"));
}

#[test]
fn xor_decrypt_display() {
    let e = DecryptError::XorDecrypt {
        detail: "key mismatch".into(),
    };
    assert!(e.to_string().contains("XOR decrypt failed"));
    assert!(e.to_string().contains("key mismatch"));
}

#[test]
fn deflate_display() {
    let e = DecryptError::Deflate {
        detail: "corrupt data".into(),
    };
    assert!(e.to_string().contains("deflate decompress failed"));
    assert!(e.to_string().contains("corrupt data"));
}

#[test]
fn aes_decrypt_display() {
    let e = DecryptError::AesDecrypt {
        detail: "bad block".into(),
    };
    assert!(e.to_string().contains("AES decrypt failed"));
}

#[test]
fn triple_des_decrypt_display() {
    let e = DecryptError::TripleDesDecrypt {
        detail: "block not aligned".into(),
    };
    assert!(e.to_string().contains("3DES decrypt failed"));
}

#[test]
fn utf8_decode_display() {
    let e = DecryptError::Utf8Decode {
        detail: "invalid byte 0xFF".into(),
    };
    assert!(e.to_string().contains("decrypted data is not valid UTF-8"));
}

#[test]
fn invalid_key_length_display() {
    let e = DecryptError::InvalidKeyLength {
        expected: 16,
        actual: 8,
    };
    let msg = e.to_string();
    assert!(msg.contains("16"));
    assert!(msg.contains("8"));
}
