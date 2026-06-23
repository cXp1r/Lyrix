use lyrix::parsers::generate::spotify;

// ===== HOTP RFC 4226 测试向量 =====

#[test]
fn hotp_rfc4226_case0() {
    assert_eq!(spotify::hotp(b"12345678901234567890", 0, 6), "755224");
}

#[test]
fn hotp_rfc4226_case1() {
    assert_eq!(spotify::hotp(b"12345678901234567890", 1, 6), "287082");
}

#[test]
fn hotp_rfc4226_case2() {
    assert_eq!(spotify::hotp(b"12345678901234567890", 2, 6), "359152");
}

#[test]
fn hotp_rfc4226_case3() {
    assert_eq!(spotify::hotp(b"12345678901234567890", 3, 6), "969429");
}

#[test]
fn hotp_8_digits() {
    assert_eq!(spotify::hotp(b"12345678901234567890", 0, 8), "84755224");
}

#[test]
fn hotp_zero_length_secret() {
    // 空密钥应该也能计算（但结果取决于 HMAC 实现）
    let result = spotify::hotp(b"", 0, 6);
    assert_eq!(result.len(), 6);
}

// ===== TOTP RFC 6238 测试 =====

#[test]
fn totp_counter_30s_period() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 30, 6, 0);
    assert_eq!(ts.counter(59_000), 1);
    assert_eq!(ts.counter(60_000), 2);
    assert_eq!(ts.counter(89_000), 2);
    assert_eq!(ts.counter(90_000), 3);
}

#[test]
fn totp_counter_60s_period() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 60, 6, 0);
    assert_eq!(ts.counter(59_000), 0);
    assert_eq!(ts.counter(60_000), 1);
    assert_eq!(ts.counter(119_000), 1);
    assert_eq!(ts.counter(120_000), 2);
}

#[test]
fn totp_rfc6238_t59() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 30, 6, 0);
    assert_eq!(ts.generate(59_000), "287082");
}

#[test]
fn totp_generate_now_produces_6_digits() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 30, 6, 0);
    let code = ts.generate_now();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

// ===== SHA1 =====

#[test]
fn sha1_empty() {
    let mut s = spotify::Sha1::new();
    s.update(b"");
    let digest = s.digest();
    assert_eq!(
        hex::encode(digest),
        "da39a3ee5e6b4b0d3255bfef95601890afd80709"
    );
}

#[test]
fn sha1_abc() {
    let mut s = spotify::Sha1::new();
    s.update(b"abc");
    let digest = s.digest();
    assert_eq!(
        hex::encode(digest),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
}

#[test]
fn sha1_long_message() {
    let mut s = spotify::Sha1::new();
    s.update(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
    let digest = s.digest();
    assert_eq!(
        hex::encode(digest),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}

#[test]
fn sha1_multiple_updates() {
    let mut s = spotify::Sha1::new();
    s.update(b"ab");
    s.update(b"c");
    let digest = s.digest();
    assert_eq!(
        hex::encode(digest),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
}

#[test]
fn sha1_digest_into() {
    let mut s = spotify::Sha1::new();
    s.update(b"test");
    let mut out = [0u8; 20];
    s.digest_into(&mut out);
    assert_eq!(hex::encode(out), "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
}

// ===== HMAC-SHA1 =====

#[test]
fn hmac_sha1_rfc2202_case1() {
    let key = [0x0b; 20];
    let data = b"Hi There";
    let result = spotify::HmacSha1::oneshot(&key, data);
    assert_eq!(
        hex::encode(result),
        "b617318655057264e28bc0b6fb378c8ef146be00"
    );
}

#[test]
fn hmac_sha1_rfc2202_case2() {
    let key = b"Jefe";
    let data = b"what do ya want for nothing?";
    let result = spotify::HmacSha1::oneshot(key, data);
    assert_eq!(
        hex::encode(result),
        "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"
    );
}

#[test]
fn hmac_sha1_oneshot_eq_streaming() {
    let key = b"testkey";
    let data = b"testdata";
    let oneshot = spotify::HmacSha1::oneshot(key, data);
    let mut h = spotify::HmacSha1::new(key);
    h.update(data);
    let streaming = h.digest();
    assert_eq!(oneshot, streaming);
}

// ===== TOTP Payload =====

#[test]
fn tl_produces_valid_payload() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 30, 6, 61);
    let payload = spotify::tl(&ts, "init", "web-player", None);
    assert_eq!(payload.reason, "init");
    assert_eq!(payload.product_type, "web-player");
    assert_eq!(payload.totp.len(), 6);
    assert_eq!(payload.totp_server, "unavailable");
    assert_eq!(payload.totp_ver, "61");
}

#[test]
fn tl_with_server_ts() {
    let ts = spotify::Totp::new(b"12345678901234567890".to_vec(), 30, 6, 61);
    let payload = spotify::tl(&ts, "refresh", "web-player", Some(59));
    assert_eq!(payload.totp_server.len(), 6);
    assert_eq!(payload.totp_ver, "61");
}

// ===== build_totp =====

#[test]
fn build_totp_valid_indices() {
    for i in 0..=2 {
        let ts = spotify::build_totp(i).unwrap();
        let code = ts.generate_now();
        assert_eq!(code.len(), 6);
    }
}

#[test]
fn build_totp_invalid_index() {
    assert!(spotify::build_totp(3).is_err());
    assert!(spotify::build_totp(99).is_err());
}

#[test]
fn build_totp_versions() {
    let ts0 = spotify::build_totp(0).unwrap();
    assert_eq!(ts0.version, 61);
    assert_eq!(ts0.period, 30);
    assert_eq!(ts0.digits, 6);

    let ts1 = spotify::build_totp(1).unwrap();
    assert_eq!(ts1.version, 60);

    let ts2 = spotify::build_totp(2).unwrap();
    assert_eq!(ts2.version, 59);
}
