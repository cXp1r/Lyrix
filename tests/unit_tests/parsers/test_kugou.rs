use lyrix::error::parser::lyrics_parse::LyricsParseError;
use lyrix::error::LyrixError;
use lyrix::parsers::kugou::KugouParser;
use lyrix::parsers::IParsers;

// ── KugouParser::get_offset_time ──────────────────────────────
// Kugou 直接返回 t2 as u16，忽略 t1（因为 KRC 格式本身存的就是 offset）

#[test]
fn kugou_offset_direct_t2() {
    let parser = KugouParser;
    // t1 被忽略，直接返回 t2
    assert_eq!(parser.get_offset_time(99999, 500).unwrap(), 500);
}

#[test]
fn kugou_offset_zero() {
    let parser = KugouParser;
    assert_eq!(parser.get_offset_time(0, 0).unwrap(), 0);
}

#[test]
fn kugou_offset_max() {
    let parser = KugouParser;
    assert_eq!(parser.get_offset_time(0, 65535).unwrap(), 65535);
}

#[test]
fn kugou_offset_overflow() {
    let parser = KugouParser;
    let err = parser.get_offset_time(0, 65536).unwrap_err();
    assert!(matches!(
        err,
        LyrixError::Parser(lyrix::error::ParserError::LyricsParse(
            LyricsParseError::OffsetOverflow { .. }
        ))
    ));
}

// ── KugouParser::parse_without_st ─────────────────────────────
// 使用 IParsers 默认实现 ([s,d]text 格式 + <s,d> 音节)

#[test]
fn kugou_parse_single_line() {
    let parser = KugouParser;
    let lyrics = "[1000,500]Hello World".to_string();
    let result = parser.parse(lyrics).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].start_time, 1000);
    assert_eq!(result[0].duration, 500);
}

#[test]
fn kugou_parse_empty() {
    let parser = KugouParser;
    let result = parser.parse(String::new()).unwrap();
    assert!(result.is_empty());
}

// ── KugouParser::decrypt_and_parse ────────────────────────────

#[test]
#[ignore = "需要真实的 KRC 加密歌词数据"]
fn kugou_decrypt_and_parse_valid() {
    let parser = KugouParser;
    // 此测试需要从已知 API 响应中获取 KRC 密文
    let encrypted = ""; // TODO: 填入真实 KRC base64 加密字符串
    let result = parser.decrypt_and_parse(encrypted.into()).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn kugou_decrypt_invalid_base64() {
    let parser = KugouParser;
    let result = parser.decrypt_and_parse("!!!not-base64!!!".into());
    assert!(result.is_err());
}
